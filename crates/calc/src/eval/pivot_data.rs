//! GETPIVOTDATA(data_field, pivot_table, [field1, item1], ...).
//!
//! The PivotTable is the one whose report (`last_range`, or its anchor before the first refresh)
//! meets the `pivot_table` reference. Its report on the sheet is static text and numbers, and in
//! the Compact layout the cells alone don't say which row field a label belongs to, so the value
//! is not read back from the report: it is computed from the PivotTable's definition over its
//! source list, the way a refresh computes the report. The records the report filters let
//! through are narrowed to those whose row/column field items match the pairs given, and the
//! value field's function summarizes them.
//!
//! - `data_field` is a value field's caption ("Sum of Sales") or its source column ("Sales").
//! - A field is a row or column field, by source column name or, for a date-grouped field, by
//!   its grouping ("Quarters"). An item matches a record by its label as the report shows it
//!   (case-insensitively), or for an ungrouped number by value.
//! - `#REF!`: no PivotTable at the reference, an unknown value field or field, a field without an
//!   item, an item with no records (including combinations no record has), a source that
//!   can't be read (outside the workbook), or a calculated value field.
//! - Show values as: % of grand, column and row total are applied; running total and rank give
//!   the plain summary.
//!
//! Deviations from Excel: the value is that of the source as it is now, so it differs from the
//! report while the PivotTable waits for a refresh; Excel's `#REF!` for cells the report hides
//! (subtotals or grand totals turned off, items under a collapsed parent, an inner field named
//! without its outer fields) is not reproduced; the value is the summary of the matching
//! records all the same.

use std::collections::HashSet;

use deskwerk_sheets_core::date::datetime_from_serial;
use deskwerk_sheets_core::{CellError, CellRef, RangeRef, Value, number_to_text, split_sheet};
use deskwerk_sheets_model::{PivotDateGroup, PivotFunc, PivotShowAs, PivotTable, PivotValue, Workbook};
use deskwerk_sheets_numfmt::{NumberFormat, format_value_in};

use super::{Area, Ev, Evaluator, Expr, err};

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A PivotTable's source list: header names, each column's number format and the records.
struct List {
    headers: Vec<String>,
    formats: Vec<String>,
    records: Vec<Vec<Value>>,
}

impl List {
    fn col(&self, name: &str) -> Option<usize> {
        let name = name.trim();
        self.headers.iter().position(|h| h.eq_ignore_ascii_case(name))
    }
    fn format(&self, col: usize) -> &str {
        self.formats.get(col).map_or("General", String::as_str)
    }
}

/// A row or column field named in the call, with the item asked for.
struct Pair {
    col: usize,
    group: PivotDateGroup,
    on_rows: bool,
    item: Value,
}

/// A report filter with a selection: source column, date grouping, selected labels (lower-case).
struct Filter {
    col: usize,
    group: PivotDateGroup,
    selected: HashSet<String>,
}

impl Evaluator<'_> {
    pub(super) fn get_pivot_data(&mut self, args: &[Expr]) -> Ev {
        self.host.note_volatile();
        let [data, table, pairs @ ..] = args else { return err(CellError::Value) };
        let data_field = match self.value(data).scalar() {
            Value::Error(e) => return err(e),
            v => label_text(&v),
        };
        let (sheet, pt) = match self.pivot_at(table) {
            Ok(found) => found,
            Err(e) => return err(e),
        };
        let mut asked = Vec::with_capacity(pairs.len() / 2);
        for pair in pairs.chunks(2) {
            let [field, item] = pair else { return err(CellError::Ref) };
            let field = match self.value(field).scalar() {
                Value::Error(e) => return err(e),
                v => label_text(&v),
            };
            match self.value(item).scalar() {
                Value::Error(e) => return err(e),
                v => asked.push((field, v)),
            }
        }
        let list = match self.pivot_list(&pt, sheet) {
            Ok(list) => list,
            Err(e) => return err(e),
        };
        Ev::V(pivot_value(self.host.workbook(), &pt, &list, &data_field, &asked))
    }

    /// The sheet and PivotTable whose report meets the reference `e`.
    fn pivot_at(&mut self, e: &Expr) -> Result<(usize, PivotTable), CellError> {
        let area = match self.eval(e) {
            Ev::R(areas) => areas.first().copied().ok_or(CellError::Ref)?,
            Ev::V(Value::Error(x)) => return Err(x),
            _ => return Err(CellError::Ref),
        };
        let sh = self.host.workbook().sheet(area.sheet).ok_or(CellError::Ref)?;
        let found = sh.pivots.iter().find(|pt| pt.last_range.unwrap_or_else(|| RangeRef::cell(pt.anchor)).intersects(&area.range));
        found.cloned().map(|pt| (area.sheet, pt)).ok_or(CellError::Ref)
    }

    /// Reads the PivotTable's source list (header row plus records; blank records skipped).
    fn pivot_list(&mut self, pt: &PivotTable, sheet: usize) -> Result<List, CellError> {
        let (si, range) = resolve_source(self.host.workbook(), &pt.source, sheet, 0).ok_or(CellError::Ref)?;
        let range = if range.is_full_cols() || range.is_full_rows() {
            let used = self.host.used_range(si).ok_or(CellError::Ref)?;
            range.intersection(&used).ok_or(CellError::Ref)?
        } else {
            range
        };
        if range.height() < 2 {
            return Err(CellError::Ref);
        }
        let formats = column_formats(self.host.workbook(), si, range);
        let block = match self.area_value(&Area { sheet: si, range }) {
            Value::Array(a) => a,
            Value::Error(e) => return Err(e),
            _ => return Err(CellError::Ref),
        };
        let headers = unique_headers((0..block.cols).map(|c| block.get(0, c).map(label_text).unwrap_or_default()));
        let records = (1..block.rows)
            .map(|r| (0..block.cols).map(|c| block.get(r, c).map(Value::scalar).unwrap_or_default()).collect::<Vec<Value>>())
            .filter(|rec| !rec.iter().all(Value::is_empty))
            .collect();
        Ok(List { headers, formats, records })
    }
}

/// Resolves a PivotTable source (range text, table name or defined name) to its sheet and range,
/// header row included, as a refresh does.
fn resolve_source(wb: &Workbook, text: &str, default_sheet: usize, depth: u32) -> Option<(usize, RangeRef)> {
    let t = text.trim().trim_start_matches('=').trim();
    if t.is_empty() || depth > 4 {
        return None;
    }
    if let Some((si, ti)) = wb.table(t) {
        let tb = wb.sheet(si)?.tables.get(ti)?;
        if !tb.header_row {
            return None;
        }
        let end_row = tb.range.end.row.saturating_sub(u32::from(tb.totals_row)).max(tb.range.start.row);
        return Some((si, RangeRef::new(tb.range.start, CellRef::new(end_row, tb.range.end.col))));
    }
    let (sheet, body) = split_sheet(t);
    if let Some(r) = RangeRef::parse(&body.replace('$', "")) {
        let si = match &sheet {
            Some(n) => wb.sheet_index(n)?,
            None => default_sheet,
        };
        return Some((si, r));
    }
    let formula = wb.name(t, default_sheet).filter(|_| sheet.is_none())?.formula.clone();
    resolve_source(wb, &formula, default_sheet, depth + 1)
}

/// Each source column's number format: that of its first record (General without records).
fn column_formats(wb: &Workbook, sheet: usize, range: RangeRef) -> Vec<String> {
    let Some(sh) = wb.sheet(sheet) else { return Vec::new() };
    (range.start.col..=range.end.col).map(|c| wb.styles.get(sh.style_id(CellRef::new(range.start.row + 1, c))).num_fmt.as_str().to_string()).collect()
}

/// Header names made unique the way a refresh does: a repeated name gets 2, 3… appended.
fn unique_headers(raw: impl Iterator<Item = String>) -> Vec<String> {
    let mut taken: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for h in raw {
        let h = h.trim().to_string();
        let mut name = h.clone();
        let mut k = 2;
        while taken.contains(&name.to_ascii_lowercase()) {
            name = format!("{h}{k}");
            k += 1;
        }
        taken.insert(name.to_ascii_lowercase());
        out.push(name);
    }
    out
}

/// A value as text for names and labels.
fn label_text(v: &Value) -> String {
    match v {
        Value::Empty => String::new(),
        Value::Number(n) => number_to_text(*n),
        Value::Text(t) => t.to_string(),
        Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.into(),
        Value::Error(e) => e.as_str().into(),
        Value::Array(_) => label_text(&v.scalar()),
    }
}

/// The label a report shows for a source value: its date group ("2026", "Qtr1", "Jan",
/// "15-Jan"), the number formatted with the column's format, "(blank)" for an empty cell.
fn item_label(wb: &Workbook, v: &Value, group: PivotDateGroup, fmt: &str) -> String {
    match v {
        Value::Empty => "(blank)".into(),
        Value::Number(n) => match datetime_from_serial(wb.date_system, *n).filter(|_| group != PivotDateGroup::None) {
            Some(dt) => {
                let mon = MONTHS.get(dt.month.saturating_sub(1) as usize).copied().unwrap_or("Jan");
                match group {
                    PivotDateGroup::Years => dt.year.to_string(),
                    PivotDateGroup::Quarters => format!("Qtr{}", dt.month.saturating_sub(1) / 3 + 1),
                    PivotDateGroup::Months => mon.to_string(),
                    PivotDateGroup::Days | PivotDateGroup::None => format!("{}-{mon}", dt.day),
                }
            }
            None => format_value_in(v, &NumberFormat::parse_cached(fmt), wb.date_system, wb.locale()).text,
        },
        Value::Array(_) => item_label(wb, &v.scalar(), group, fmt),
        _ => label_text(v),
    }
}

/// Whether a record's value `v` is the item asked for.
fn item_matches(wb: &Workbook, v: &Value, item: &Value, group: PivotDateGroup, fmt: &str) -> bool {
    if group == PivotDateGroup::None
        && let (Value::Number(a), Value::Number(b)) = (v, item)
    {
        return (a - b).abs() <= f64::EPSILON * a.abs().max(b.abs());
    }
    let want = label_text(item);
    let want = want.trim();
    if item_label(wb, v, group, fmt).trim().to_lowercase() == want.to_lowercase() {
        return true;
    }
    // A number item typed as text in General form ("1500" for a number shown as "1,500").
    group == PivotDateGroup::None && matches!(v, Value::Number(_)) && label_text(v) == want
}

/// The value field named `name`: by caption first, then by source column.
fn data_value<'a>(pt: &'a PivotTable, name: &str) -> Option<&'a PivotValue> {
    let name = name.trim().to_lowercase();
    pt.values.iter().find(|v| v.name.trim().to_lowercase() == name).or_else(|| pt.values.iter().find(|v| v.source_col.trim().to_lowercase() == name))
}

/// The caption Excel gives a date grouping's field.
fn group_caption(g: PivotDateGroup) -> Option<&'static str> {
    match g {
        PivotDateGroup::Years => Some("Years"),
        PivotDateGroup::Quarters => Some("Quarters"),
        PivotDateGroup::Months => Some("Months"),
        PivotDateGroup::Days => Some("Days"),
        PivotDateGroup::None => None,
    }
}

/// The row or column field named `name`: (source column, grouping, on rows).
fn axis_field(pt: &PivotTable, list: &List, name: &str) -> Option<(usize, PivotDateGroup, bool)> {
    let name = name.trim();
    let fields = || pt.rows.iter().map(|f| (f, true)).chain(pt.columns.iter().map(|f| (f, false)));
    let by_name = fields().find(|(f, _)| f.source_col.trim().eq_ignore_ascii_case(name));
    let (field, on_rows) = by_name.or_else(|| fields().find(|(f, _)| group_caption(f.date_group).is_some_and(|c| c.eq_ignore_ascii_case(name))))?;
    Some((list.col(&field.source_col)?, field.date_group, on_rows))
}

/// The report filters that hide items, as a refresh applies them (a filter on a row or column
/// field compares that field's grouped labels).
fn report_filters(pt: &PivotTable, list: &List) -> Vec<Filter> {
    let axis_group = |name: &str| pt.rows.iter().chain(pt.columns.iter()).find(|f| f.source_col.eq_ignore_ascii_case(name)).map(|f| f.date_group);
    pt.filters
        .iter()
        .filter_map(|f| {
            let selected = f.selected.as_ref()?.iter().map(|s| s.to_ascii_lowercase()).collect();
            Some(Filter { col: list.col(&f.source_col)?, group: axis_group(&f.source_col).unwrap_or_default(), selected })
        })
        .collect()
}

fn cell(rec: &[Value], col: usize) -> &Value {
    rec.get(col).unwrap_or(&Value::Empty)
}

/// Summary of `col` over the records matching every pair; `None` when no record matches.
fn summarize(wb: &Workbook, list: &List, records: &[&[Value]], pairs: &[&Pair], col: usize, func: PivotFunc) -> Option<Value> {
    let mut acc = Acc::default();
    let mut any = false;
    for rec in records {
        if pairs.iter().all(|p| item_matches(wb, cell(rec, p.col), &p.item, p.group, list.format(p.col))) {
            any = true;
            acc.add(cell(rec, col));
        }
    }
    any.then(|| acc.result(func))
}

fn ratio(a: Value, b: Option<Value>) -> Value {
    match (a, b) {
        (Value::Error(e), _) | (_, Some(Value::Error(e))) => Value::Error(e),
        (Value::Number(_), Some(Value::Number(0.0))) => Value::Error(CellError::Div0),
        (Value::Number(x), Some(Value::Number(y))) => Value::number(x / y),
        (value, _) => value,
    }
}

/// GETPIVOTDATA's result for a PivotTable over its source list.
fn pivot_value(wb: &Workbook, pt: &PivotTable, list: &List, data_field: &str, asked: &[(String, Value)]) -> Value {
    let Some(value) = data_value(pt, data_field) else { return Value::Error(CellError::Ref) };
    // Calculated fields (no source column) aren't summarized here.
    let Some(col) = list.col(&value.source_col) else { return Value::Error(CellError::Ref) };
    let mut pairs = Vec::with_capacity(asked.len());
    for (name, item) in asked {
        let Some((col, group, on_rows)) = axis_field(pt, list, name) else { return Value::Error(CellError::Ref) };
        pairs.push(Pair { col, group, on_rows, item: item.clone() });
    }
    let filters = report_filters(pt, list);
    let shown = |rec: &&Vec<Value>| {
        filters.iter().all(|f| f.selected.contains(&item_label(wb, cell(rec, f.col), f.group, list.format(f.col)).to_ascii_lowercase()))
    };
    let records: Vec<&[Value]> = list.records.iter().filter(shown).map(Vec::as_slice).collect();
    let all: Vec<&Pair> = pairs.iter().collect();
    let Some(raw) = summarize(wb, list, &records, &all, col, value.func) else { return Value::Error(CellError::Ref) };
    // The total over the pairs on rows (`Some(true)`), on columns (`Some(false)`) or none.
    let total = |axis: Option<bool>| {
        let some: Vec<&Pair> = pairs.iter().filter(|p| axis == Some(p.on_rows)).collect();
        summarize(wb, list, &records, &some, col, value.func)
    };
    match value.show_as {
        PivotShowAs::Normal | PivotShowAs::RunningTotal | PivotShowAs::Rank => raw,
        PivotShowAs::PercentOfGrandTotal => ratio(raw, total(None)),
        PivotShowAs::PercentOfColumnTotal => ratio(raw, total(Some(false))),
        PivotShowAs::PercentOfRowTotal => ratio(raw, total(Some(true))),
    }
}

/// Running summary of one value field (as the report's aggregation does).
#[derive(Default)]
struct Acc {
    count: u64,
    nums: u64,
    sum: f64,
    prod: f64,
    min: f64,
    max: f64,
    mean: f64,
    m2: f64,
    err: Option<CellError>,
}

impl Acc {
    fn add(&mut self, v: &Value) {
        match v {
            Value::Empty => {}
            Value::Number(n) => self.add_number(*n),
            Value::Error(e) => {
                self.count += 1;
                self.err = self.err.or(Some(*e));
            }
            _ => self.count += 1,
        }
    }

    fn add_number(&mut self, n: f64) {
        self.count += 1;
        self.nums += 1;
        self.sum += n;
        if self.nums == 1 {
            (self.prod, self.min, self.max) = (n, n, n);
        } else {
            self.prod *= n;
            self.min = self.min.min(n);
            self.max = self.max.max(n);
        }
        let d = n - self.mean;
        self.mean += d / self.nums as f64;
        self.m2 += d * (n - self.mean);
    }

    fn result(&self, f: PivotFunc) -> Value {
        let n = self.nums as f64;
        match f {
            PivotFunc::Count => return Value::number(self.count as f64),
            PivotFunc::CountNumbers => return Value::number(n),
            _ => {}
        }
        if let Some(e) = self.err {
            return Value::Error(e);
        }
        let none = self.nums == 0;
        match f {
            PivotFunc::Sum => Value::number(self.sum),
            PivotFunc::Average if none => Value::Error(CellError::Div0),
            PivotFunc::Average => Value::number(self.sum / n),
            PivotFunc::Max => Value::number(if none { 0.0 } else { self.max }),
            PivotFunc::Min => Value::number(if none { 0.0 } else { self.min }),
            PivotFunc::Product => Value::number(if none { 0.0 } else { self.prod }),
            PivotFunc::StdDev | PivotFunc::Var if self.nums < 2 => Value::Error(CellError::Div0),
            PivotFunc::StdDevP | PivotFunc::VarP if none => Value::Error(CellError::Div0),
            PivotFunc::Var => Value::number((self.m2 / (n - 1.0)).max(0.0)),
            PivotFunc::StdDev => Value::number((self.m2 / (n - 1.0)).max(0.0).sqrt()),
            PivotFunc::VarP => Value::number((self.m2 / n).max(0.0)),
            PivotFunc::StdDevP => Value::number((self.m2 / n).max(0.0).sqrt()),
            PivotFunc::Count | PivotFunc::CountNumbers => Value::Empty,
        }
    }
}
