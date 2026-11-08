//! Legacy formulas: Excel's implicit intersection, spelled out with `@` on import.
//!
//! A formula a file stores without dynamic-array metadata (`cm`) and outside a legacy array range
//! is a legacy formula. Where one gives a multi-cell reference to an operator or to a parameter
//! that takes one value, Excel intersects the reference with the formula's row or column (and a
//! function that may return several values gives its first). GridCraft evaluates every formula
//! as a dynamic array, so it would spill there (or show `#SPILL!`) instead. On import those
//! places get the `@` operator, as Excel 365 shows them; on export, a formula whose every `@` is
//! one of those is written back as the legacy formula without them.

use std::sync::Arc;

use gridcraft_core::CellRef;
use gridcraft_formula::{BinOp, Expr, UnOp};
use gridcraft_model::{Formula, Workbook};

/// Functions only dynamic-array Excel has (or that only make sense there): a formula calling one
/// is left as it is.
const DYNAMIC_ONLY: &[&str] = &[
    "LET",
    "LAMBDA",
    "FILTER",
    "SORT",
    "SORTBY",
    "UNIQUE",
    "SEQUENCE",
    "RANDARRAY",
    "VSTACK",
    "HSTACK",
    "TOROW",
    "TOCOL",
    "WRAPROWS",
    "WRAPCOLS",
    "TAKE",
    "DROP",
    "CHOOSEROWS",
    "CHOOSECOLS",
    "EXPAND",
    "GROUPBY",
    "PIVOTBY",
    "MAP",
    "REDUCE",
    "SCAN",
    "BYROW",
    "BYCOL",
    "MAKEARRAY",
    "ANCHORARRAY",
    "SINGLE",
    "TEXTSPLIT",
    "TRIMRANGE",
    "REGEXEXTRACT",
    "ISOMITTED",
];

/// Functions that evaluate every argument as an array in a legacy formula.
const ARRAY_ARGS: &[&str] = &["SUMPRODUCT", "MMULT", "MINVERSE", "MDETERM", "TRANSPOSE", "FREQUENCY", "LINEST", "LOGEST", "TREND", "GROWTH"];

/// What a position in a legacy formula takes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Takes {
    /// One value: a multi-cell reference there is intersected.
    Value,
    /// A reference or an array as it is; operators inside still take values.
    Reference,
}

/// Whether an argument evaluates as an array in a legacy formula (no `@` anywhere inside).
fn array_arg(name: &str, i: usize) -> bool {
    ARRAY_ARGS.contains(&name) || (name == "INDEX" && i == 0) || (name == "LOOKUP" && i > 0) || (name == "AGGREGATE" && i > 1)
}

/// Whether a parameter takes one value. [`crate::array_formula::scalar_arg`] errs towards
/// "one value", which is safe for export; here a wrong `@` would change a result, so parameters
/// that take ranges and that list misses are excluded.
fn value_arg(name: &str, i: usize) -> bool {
    let takes_range = match name {
        "STDEVA" | "STDEVPA" | "VARA" | "VARPA" | "MODE.MULT" | "GCD" | "LCM" | "MULTINOMIAL" | "IMSUM" | "IMPRODUCT" | "N" | "T" => true,
        "TRIMMEAN" | "PERCENTRANK" | "PERCENTRANK.INC" | "PERCENTRANK.EXC" | "ZTEST" | "Z.TEST" | "FORMULATEXT" | "ISFORMULA" | "SHEET"
        | "SHEETS" | "ARRAYTOTEXT" => i == 0,
        "CHITEST" | "CHISQ.TEST" | "TTEST" | "T.TEST" | "FTEST" | "F.TEST" | "PROB" => i <= 1,
        "FVSCHEDULE" | "CELL" | "GETPIVOTDATA" => i == 1,
        "NETWORKDAYS" | "WORKDAY" => i == 2,
        "NETWORKDAYS.INTL" | "WORKDAY.INTL" | "SERIESSUM" => i == 3,
        "DAVERAGE" | "DCOUNT" | "DCOUNTA" | "DGET" | "DMAX" | "DMIN" | "DPRODUCT" | "DSTDEV" | "DSTDEVP" | "DSUM" | "DVAR" | "DVARP" => i != 1,
        _ => false,
    };
    !takes_range && crate::array_formula::scalar_arg(name, i)
}

fn is_reference_op(op: BinOp) -> bool {
    matches!(op, BinOp::Range | BinOp::Union | BinOp::Intersect)
}

/// Whether the formula is left as it is: it uses something only dynamic-array Excel has, or it
/// already says where it intersects.
fn left_alone(e: &Expr) -> bool {
    let mut skip = false;
    e.walk(&mut |x| {
        skip |= match x {
            Expr::Call(name, _) => DYNAMIC_ONLY.contains(&name.as_str()),
            Expr::Unary(op, _) => matches!(op, UnOp::At | UnOp::Spill | UnOp::Trim(_)),
            Expr::Invoke(_, _) => true,
            _ => false,
        }
    });
    skip
}

/// What argument `i` of `name` takes when the call's result goes where `takes` says; `None`
/// where the argument evaluates as an array.
fn arg_takes(name: &str, i: usize, takes: Takes) -> Option<Takes> {
    if array_arg(name, i) {
        return None;
    }
    Some(match name {
        // IF and CHOOSE pass a reference through to what takes their result.
        "IF" | "CHOOSE" if i > 0 => takes,
        _ if value_arg(name, i) => Takes::Value,
        _ => Takes::Reference,
    })
}

struct Intersect<'a> {
    wb: &'a Workbook,
    sheet: usize,
    at: CellRef,
}

impl Intersect<'_> {
    fn expr(&self, e: &Expr, takes: Takes) -> Expr {
        match e {
            // `@` binds tighter than `:`, so a reference operator gets parentheses under it.
            Expr::Binary(op, _, _) if is_reference_op(*op) => match self.single(e.clone(), takes) {
                Expr::Unary(UnOp::At, x) => Expr::Unary(UnOp::At, Box::new(Expr::Paren(x))),
                same => same,
            },
            Expr::Binary(_, _, _) => self.operators(e),
            Expr::Paren(x) if matches!(x.as_ref(), Expr::Binary(op, _, _) if is_reference_op(*op)) => self.single(e.clone(), takes),
            Expr::Paren(x) => Expr::Paren(Box::new(self.expr(x, takes))),
            Expr::Unary(op @ (UnOp::Neg | UnOp::Plus | UnOp::Percent), x) => Expr::Unary(*op, Box::new(self.expr(x, Takes::Value))),
            Expr::Ref(_) | Expr::Name(_) | Expr::Struct(_) => self.single(e.clone(), takes),
            Expr::Call(name, args) => {
                let args = args
                    .iter()
                    .enumerate()
                    .map(|(i, a)| match arg_takes(name, i, takes) {
                        Some(t) => self.expr(a, t),
                        None => self.array(a),
                    })
                    .collect();
                self.single(Expr::Call(name.clone(), args), takes)
            }
            other => other.clone(),
        }
    }

    /// An argument evaluated as an array. IF is the exception: without CSE its condition still
    /// takes one value (`SUMPRODUCT(IF(A1:A3>1,1,0))` needs CSE in Excel 2019 and earlier).
    fn array(&self, e: &Expr) -> Expr {
        e.clone().map(&mut |x| match x {
            Expr::Call(name, mut args) if name == "IF" => {
                if let Some(condition) = args.first_mut() {
                    *condition = self.expr(condition, Takes::Value);
                }
                Expr::Call(name, args)
            }
            other => other,
        })
    }

    /// An operator chain such as `A1+A2+…` nests on its left: follow it in a loop.
    fn operators(&self, e: &Expr) -> Expr {
        let mut spine = Vec::new();
        let mut left = e;
        while let Expr::Binary(op, a, b) = left
            && !is_reference_op(*op)
        {
            spine.push((*op, b.as_ref()));
            left = a.as_ref();
        }
        let mut acc = self.expr(left, Takes::Value);
        while let Some((op, b)) = spine.pop() {
            acc = Expr::Binary(op, Box::new(acc), Box::new(self.expr(b, Takes::Value)));
        }
        acc
    }

    /// `@e` where one value is taken and `e` may give several.
    fn single(&self, e: Expr, takes: Takes) -> Expr {
        if takes == Takes::Value && crate::array_formula::may_be_array(self.wb, self.sheet, self.at, &e) {
            Expr::Unary(UnOp::At, Box::new(e))
        } else {
            e
        }
    }
}

/// The formula with `@` wherever the legacy formula `e` at `at` intersects, or `None` when it
/// needs none (or is left alone).
fn with_intersections(wb: &Workbook, sheet: usize, at: CellRef, e: &Expr) -> Option<Expr> {
    if left_alone(e) {
        return None;
    }
    let fixed = Intersect { wb, sheet, at }.expr(e, Takes::Value);
    (fixed != *e).then_some(fixed)
}

/// Spells out implicit intersection in every legacy formula of a workbook read from a file: one
/// with neither dynamic-array metadata nor a legacy array range.
pub(crate) fn mark_implicit_intersections(wb: &mut Workbook) {
    for si in 0..wb.sheets.len() {
        let fixes = sheet_intersections(wb, si);
        if fixes.is_empty() {
            continue;
        }
        let Some(sheet) = wb.sheet_mut(si) else { continue };
        for (at, e) in fixes {
            if let Some(cell) = sheet.cells.get_mut(at) {
                cell.formula = Some(Arc::new(Formula::from_expr(e)));
            }
        }
    }
}

/// The legacy formulas of sheet `si` that intersect, with their `@` spelled out.
fn sheet_intersections(wb: &Workbook, si: usize) -> Vec<(CellRef, Expr)> {
    let Some(sheet) = wb.sheet(si) else { return Vec::new() };
    sheet
        .cells
        .iter()
        .filter_map(|(at, cell)| {
            let f = cell.formula.as_deref().filter(|f| !f.dynamic && f.array.is_none())?;
            with_intersections(wb, si, at, f.expr.as_deref()?).map(|e| (at, e))
        })
        .collect()
}

/// The legacy formula to write for `f` at `at`: `f` without its `@`s, when each of them is one
/// a legacy formula implies (reading it back gives `f` again). `None` when `f` has no `@`, is a
/// dynamic-array or array formula, or has an `@` a legacy formula doesn't imply.
pub(crate) fn legacy_form(wb: &Workbook, sheet: usize, at: CellRef, f: &Formula) -> Option<Formula> {
    if f.dynamic || f.array.is_some() {
        return None;
    }
    let e = f.expr_arc()?;
    let mut has_at = false;
    e.walk(&mut |x| has_at |= matches!(x, Expr::Unary(UnOp::At, _)));
    if !has_at {
        return None;
    }
    let plain = Expr::clone(&e).map(&mut |x| match x {
        Expr::Unary(UnOp::At, inner) => *inner,
        other => other,
    });
    (with_intersections(wb, sheet, at, &plain).as_ref() == Some(e.as_ref())).then(|| Formula::from_expr(plain))
}
