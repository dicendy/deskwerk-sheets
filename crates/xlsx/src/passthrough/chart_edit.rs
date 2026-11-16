//! Bringing a kept chart part (#1668) in line with an edited title, legend or chart type, so the
//! chart keeps the formatting Deskwerk Sheets' model doesn't hold (fonts, colours, line dashes, manual
//! layout, style parts). The title and legend are edited where they are; a change between the
//! 2-D column, bar, line and area types rewrites the one plot group, keeping each series' shape
//! properties, points, labels, trendlines and data. Anything else (another plot group, a 3-D or
//! combination chart, a pie) gives `None`, and the chart is written from the model.

use std::fmt::Write as _;
use std::ops::Range;

use deskwerk_sheets_model::{Chart, ChartKind, LegendPos};

use super::chart_patch::{Doc, Edit};
use crate::chart::{G, PLOT_GROUPS, group_for, read_chart};
use crate::xml::esc;

const NS_A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

/// The chart types a kept plot group can change between in place.
const IN_PLACE: [ChartKind; 12] = [
    ChartKind::ColumnClustered,
    ChartKind::ColumnStacked,
    ChartKind::ColumnStacked100,
    ChartKind::BarClustered,
    ChartKind::BarStacked,
    ChartKind::BarStacked100,
    ChartKind::Line,
    ChartKind::LineMarkers,
    ChartKind::LineStacked,
    ChartKind::Area,
    ChartKind::AreaStacked,
    ChartKind::AreaStacked100,
];

/// The children of a bar, line or area plot group (CT_BarChart, CT_LineChart, CT_AreaChart).
const GROUP_KIDS: [&str; 15] = [
    "barDir",
    "grouping",
    "varyColors",
    "ser",
    "dLbls",
    "gapWidth",
    "overlap",
    "serLines",
    "dropLines",
    "hiLowLines",
    "upDownBars",
    "marker",
    "smooth",
    "axId",
    "extLst",
];

/// The children of a bar, line or area series (CT_BarSer, CT_LineSer, CT_AreaSer).
const SER_KIDS: [&str; 16] = [
    "idx",
    "order",
    "tx",
    "spPr",
    "invertIfNegative",
    "pictureOptions",
    "marker",
    "dPt",
    "dLbls",
    "trendline",
    "errBars",
    "cat",
    "val",
    "shape",
    "smooth",
    "extLst",
];

/// The fills of DrawingML shape and line properties.
const FILLS: [&str; 6] = ["noFill", "solidFill", "gradFill", "blipFill", "pattFill", "grpFill"];

/// The kind of plot group a series sits in.
#[derive(Clone, Copy, PartialEq)]
enum Fam {
    Bar,
    Line,
    Area,
}

fn family(g: G) -> Option<Fam> {
    match g {
        G::Bar { .. } => Some(Fam::Bar),
        G::Line { .. } => Some(Fam::Line),
        G::Area { .. } => Some(Fam::Area),
        _ => None,
    }
}

/// The title a chart shows: none for an empty one, as `chart::write_chart` writes it.
pub(super) fn shown_title(ch: &Chart) -> Option<&str> {
    ch.title.as_deref().filter(|t| !t.is_empty())
}

/// Whether the title, legend or type differ, the fields [`edit_fields`] edits.
pub(super) fn fields_changed(old: &Chart, new: &Chart) -> bool {
    shown_title(old) != shown_title(new) || old.legend != new.legend || old.kind != new.kind
}

/// A chart without the fields [`edit_fields`] edits, for comparing what else changed.
pub(super) fn without_fields(mut ch: Chart) -> Chart {
    ch.title = None;
    ch.legend = LegendPos::None;
    ch.kind = ChartKind::default();
    ch
}

/// The edits that change the title, legend and type of kept chart XML `data`, read as `old`, to
/// those of `new`. `None` when the XML can't follow (see the module comment).
pub(super) fn edit_fields(data: &[u8], old: &Chart, new: &Chart) -> Option<Vec<Edit>> {
    let d = Doc::parse(std::str::from_utf8(data).ok()?)?;
    let chart = d.kid(0, "chart")?;
    let mut edits = vec![];
    if shown_title(old) != shown_title(new) {
        edits.extend(title_edits(&d, chart, shown_title(new))?);
    }
    if old.legend != new.legend {
        edits.extend(legend_edits(&d, chart, new.legend)?);
    }
    if old.kind != new.kind {
        edits.extend(kind_edits(&d, chart, new.kind)?);
    }
    Some(edits)
}

/// Whether edited chart XML reads back with the model's type, title, legend and series count.
pub(super) fn reads_as(data: &[u8], model: &Chart) -> bool {
    let Ok(root) = crate::xml::parse(data) else { return false };
    read_chart(&root).is_some_and(|c| {
        c.kind == model.kind && shown_title(&c) == shown_title(model) && c.legend == model.legend && c.series.len() == model.series.len()
    })
}

fn at(i: usize) -> Range<usize> {
    i..i
}

/// The XML of element `i`, tags included.
fn outer<'d>(d: &'d Doc<'_>, i: usize) -> Option<&'d str> {
    d.text.get(d.node(i)?.outer.clone())
}

/// The XML of the children of `i` named `local`, one after the other.
fn kept(d: &Doc<'_>, i: usize, local: &str) -> String {
    d.kids(i, local).filter_map(|k| outer(d, k)).collect()
}

fn kept_or(d: &Doc<'_>, i: usize, local: &str, default: String) -> String {
    let s = kept(d, i, local);
    if s.is_empty() { default } else { s }
}

fn local<'d>(d: &'d Doc<'_>, i: usize) -> &'d str {
    d.node(i).map_or("", |n| n.local.as_str())
}

/// The XML of element `i` without its descendants named in `names`.
fn without(d: &Doc<'_>, i: usize, names: &[&str]) -> Option<String> {
    let n = d.node(i)?;
    let mut cut = vec![];
    let mut stack = n.kids.clone();
    while let Some(k) = stack.pop() {
        let kn = d.node(k)?;
        if names.contains(&kn.local.as_str()) {
            cut.push(kn.outer.clone());
        } else {
            stack.extend(kn.kids.iter().copied());
        }
    }
    cut.sort_by_key(|r| r.start);
    let mut s = String::new();
    let mut from = n.outer.start;
    for r in cut {
        s.push_str(d.text.get(from..r.start)?);
        from = r.end;
    }
    s.push_str(d.text.get(from..n.outer.end)?);
    Some(s)
}

/// The XML of element `i` with `range` (inside it) replaced by `with`.
fn splice(d: &Doc<'_>, i: usize, range: Range<usize>, with: &str) -> Option<String> {
    let n = d.node(i)?;
    Some(format!("{}{with}{}", d.text.get(n.outer.start..range.start)?, d.text.get(range.end..n.outer.end)?))
}

// ---------------------------------------------------------------- title

/// One `a:p` per line, each with the paragraph and run properties given.
fn paragraphs(a: &str, ppr: &str, rpr: &str, text: &str) -> String {
    text.split('\n').map(|line| format!("<{a}p>{ppr}<{a}r>{rpr}<{a}t>{}</{a}t></{a}r></{a}p>", esc(line))).collect()
}

/// A rich-text `c:tx` for a title, its font taken from the title's text properties
/// (`c:txPr`) when it has them.
fn rich_tx(d: &Doc<'_>, c: &str, text: &str, txpr: Option<usize>) -> String {
    let template = txpr.and_then(|p| {
        let body = d.kid(p, "bodyPr")?;
        let pfx = d.prefix(body).strip_suffix(':').filter(|x| !x.is_empty())?;
        let lst = d.kid(p, "lstStyle").and_then(|x| outer(d, x)).unwrap_or("");
        let ppr = d.kid(p, "p").and_then(|q| d.kid(q, "pPr")).and_then(|x| outer(d, x)).unwrap_or("");
        Some((pfx, format!("{}{lst}", outer(d, body)?), ppr))
    });
    let (pfx, head, ppr) = template.unwrap_or(("a", "<a:bodyPr/><a:lstStyle/>".into(), "<a:pPr><a:defRPr/></a:pPr>"));
    // The DrawingML prefix is declared here, so the title doesn't depend on the root's.
    format!("<{c}tx><{c}rich xmlns:{pfx}=\"{NS_A}\">{head}{}</{c}rich></{c}tx>", paragraphs(&format!("{pfx}:"), ppr, "", text))
}

/// The paragraphs of a kept rich title replaced by `text`, in the first paragraph's and run's
/// properties.
fn rewrite_rich(d: &Doc<'_>, rich: usize, text: &str) -> Option<Edit> {
    let ps: Vec<usize> = d.kids(rich, "p").collect();
    let (&p0, &last) = (ps.first()?, ps.last()?);
    let a = d.prefix(p0);
    let ppr = d.kid(p0, "pPr").and_then(|x| outer(d, x)).unwrap_or("");
    let rpr = d.kid(p0, "r").and_then(|r| d.kid(r, "rPr")).and_then(|x| outer(d, x)).unwrap_or("");
    Some((d.node(p0)?.outer.start..d.node(last)?.outer.end, paragraphs(a, ppr, rpr, text)))
}

/// The edit that gives kept title `t` the text `text`.
fn retitle(d: &Doc<'_>, t: usize, c: &str, text: &str) -> Option<Edit> {
    let n = d.node(t)?;
    if n.empty {
        return Some((n.outer.clone(), format!("<{c}title>{}<{c}overlay val=\"0\"/></{c}title>", rich_tx(d, c, text, None))));
    }
    let tx = d.kid(t, "tx");
    if let Some(e) = tx.and_then(|x| d.kid(x, "rich")).and_then(|r| rewrite_rich(d, r, text)) {
        return Some(e);
    }
    // A title from a cell (`c:strRef`), or none written: a rich title in its text properties.
    let new = rich_tx(d, c, text, d.kid(t, "txPr"));
    Some(match tx.and_then(|x| d.node(x)) {
        Some(x) => (x.outer.clone(), new),
        None => (at(n.inner.start), new),
    })
}

fn title_edits(d: &Doc<'_>, chart: usize, title: Option<&str>) -> Option<Vec<Edit>> {
    let c = d.prefix(chart);
    let t = d.kid(chart, "title");
    let deleted = d.kid(chart, "autoTitleDeleted").and_then(|a| d.node(a)).map(|a| a.outer.clone());
    let mut edits = vec![];
    let Some(text) = title else {
        let n = d.node(t?)?;
        edits.push((n.outer.clone(), String::new()));
        // Excel would show an automatic title (the series name) in its place.
        edits.push((deleted.unwrap_or(at(n.outer.end)), format!("<{c}autoTitleDeleted val=\"1\"/>")));
        return Some(edits);
    };
    match t {
        Some(t) => edits.push(retitle(d, t, c, text)?),
        // The title is the chart's first child; inserted before an `autoTitleDeleted` there.
        None => edits.push((at(d.node(chart)?.inner.start), format!("<{c}title>{}<{c}overlay val=\"0\"/></{c}title>", rich_tx(d, c, text, None)))),
    }
    if let Some(r) = deleted {
        edits.push((r, format!("<{c}autoTitleDeleted val=\"0\"/>")));
    }
    Some(edits)
}

// ---------------------------------------------------------------- legend

fn legend_pos(p: LegendPos) -> Option<&'static str> {
    match p {
        LegendPos::None => None,
        LegendPos::Bottom => Some("b"),
        LegendPos::Top => Some("t"),
        LegendPos::Left => Some("l"),
        LegendPos::Right => Some("r"),
    }
}

fn legend_edits(d: &Doc<'_>, chart: usize, legend: LegendPos) -> Option<Vec<Edit>> {
    let c = d.prefix(chart);
    let l = d.kid(chart, "legend");
    let Some(p) = legend_pos(legend) else {
        return Some(l.and_then(|l| d.node(l)).map(|n| (n.outer.clone(), String::new())).into_iter().collect());
    };
    let pos = format!("<{c}legendPos val=\"{p}\"/>");
    let whole = format!("<{c}legend>{pos}<{c}overlay val=\"0\"/></{c}legend>");
    let Some(l) = l else {
        // After the plot area (CT_Chart).
        return Some(vec![(at(d.node(d.kid(chart, "plotArea")?)?.outer.end), whole)]);
    };
    let n = d.node(l)?;
    if n.empty {
        return Some(vec![(n.outer.clone(), whole)]);
    }
    let mut edits = vec![match d.kid(l, "legendPos").and_then(|x| d.node(x)) {
        Some(x) => (x.outer.clone(), pos),
        None => (at(n.inner.start), pos),
    }];
    // A manual layout placed the legend for its old side.
    if let Some(x) = d.kid(l, "layout").and_then(|x| d.node(x)) {
        edits.push((x.outer.clone(), String::new()));
    }
    Some(edits)
}

// ---------------------------------------------------------------- chart type

fn kind_edits(d: &Doc<'_>, chart: usize, kind: ChartKind) -> Option<Vec<Edit>> {
    if !IN_PLACE.contains(&kind) {
        return None;
    }
    let to = group_for(kind);
    let plot = d.kid(chart, "plotArea")?;
    let mut groups = d.node(plot)?.kids.iter().copied().filter(|g| PLOT_GROUPS.contains(&local(d, *g)));
    let (Some(g), None) = (groups.next(), groups.next()) else { return None };
    let n = d.node(g)?;
    let from = match n.local.as_str() {
        "barChart" => Fam::Bar,
        "lineChart" => Fam::Line,
        "areaChart" => Fam::Area,
        _ => return None,
    };
    if !n.attrs.is_empty() || n.kids.iter().any(|k| !GROUP_KIDS.contains(&local(d, *k))) {
        return None;
    }
    let mut edits = vec![(n.outer.clone(), group(d, g, from, to)?)];
    let was_horizontal = from == Fam::Bar && d.kid(g, "barDir").and_then(|b| d.attr(b, "val")) == Some("bar");
    if was_horizontal != matches!(to, G::Bar { bar: true, .. }) {
        edits.extend(swap_axes(d, plot, g)?);
    }
    Some(edits)
}

/// Bars run along the other axis: the axes of group `g` move to the other side (bottom ↔ left,
/// top ↔ right).
fn swap_axes(d: &Doc<'_>, plot: usize, g: usize) -> Option<Vec<Edit>> {
    let ids: Vec<&str> = d.kids(g, "axId").filter_map(|a| d.attr(a, "val")).collect();
    let mut edits = vec![];
    for ax in d.node(plot)?.kids.iter().copied() {
        let id = d.kid(ax, "axId").and_then(|a| d.attr(a, "val"));
        let ours = ["catAx", "valAx", "dateAx", "serAx"].contains(&local(d, ax)) && id.is_some_and(|id| ids.contains(&id));
        let Some(p) = d.kid(ax, "axPos").filter(|_| ours) else { continue };
        let side = match d.attr(p, "val")? {
            "b" => "l",
            "l" => "b",
            "t" => "r",
            "r" => "t",
            _ => return None,
        };
        let n = d.node(p)?;
        edits.push((n.outer.clone(), format!("<{} val=\"{side}\"/>", n.qname)));
    }
    Some(edits)
}

/// Plot group `g` (of family `from`) rewritten as group `to`, in schema order.
fn group(d: &Doc<'_>, g: usize, from: Fam, to: G) -> Option<String> {
    let c = d.prefix(g);
    let (name, head) = match to {
        G::Bar { bar, grouping } => {
            ("barChart", format!("<{c}barDir val=\"{}\"/><{c}grouping val=\"{grouping}\"/>", if bar { "bar" } else { "col" }))
        }
        G::Line { stacked, .. } => ("lineChart", format!("<{c}grouping val=\"{}\"/>", if stacked { "stacked" } else { "standard" })),
        G::Area { grouping } => ("areaChart", format!("<{c}grouping val=\"{grouping}\"/>")),
        _ => return None,
    };
    let mut s = format!("<{c}{name}>{head}");
    s.push_str(&kept_or(d, g, "varyColors", format!("<{c}varyColors val=\"0\"/>")));
    for ser in d.kids(g, "ser") {
        s.push_str(&series(d, ser, from, to)?);
    }
    // Label positions differ between the types (and stacking); labels go to the default one.
    if let Some(l) = d.kid(g, "dLbls") {
        s.push_str(&without(d, l, &["dLblPos"])?);
    }
    s.push_str(&group_tail(d, g, from, to));
    s.push_str(&kept(d, g, "axId"));
    s.push_str(&kept(d, g, "extLst"));
    let _ = write!(s, "</{c}{name}>");
    Some(s)
}

/// What follows the labels in group `to`: bar gaps and overlap, line drop and high-low lines.
fn group_tail(d: &Doc<'_>, g: usize, from: Fam, to: G) -> String {
    let c = d.prefix(g);
    let keep_if = |yes: bool, local: &str| if yes { kept(d, g, local) } else { String::new() };
    match to {
        G::Bar { grouping, .. } => {
            let was_bar = from == Fam::Bar;
            let was_clustered = d.kid(g, "grouping").and_then(|x| d.attr(x, "val")).unwrap_or("clustered") == "clustered";
            let gap = if was_bar { kept(d, g, "gapWidth") } else { format!("<{c}gapWidth val=\"150\"/>") };
            let overlap = match grouping {
                "clustered" => keep_if(was_bar && was_clustered, "overlap"),
                // Stacked bars sit on each other.
                _ => format!("<{c}overlap val=\"100\"/>"),
            };
            format!("{gap}{overlap}{}", keep_if(was_bar, "serLines"))
        }
        G::Line { .. } => {
            let was_line = from == Fam::Line;
            format!(
                "{}{}{}<{c}marker val=\"1\"/>{}",
                keep_if(from != Fam::Bar, "dropLines"),
                keep_if(was_line, "hiLowLines"),
                keep_if(was_line, "upDownBars"),
                keep_if(was_line, "smooth")
            )
        }
        _ => keep_if(from != Fam::Bar, "dropLines"),
    }
}

/// Series `ser` (of a group of family `from`) rewritten for group `to`, in schema order.
fn series(d: &Doc<'_>, ser: usize, from: Fam, to: G) -> Option<String> {
    let n = d.node(ser)?;
    if n.empty || n.kids.iter().any(|k| !SER_KIDS.contains(&local(d, *k))) {
        return None;
    }
    let fam = family(to)?;
    let c = d.prefix(ser);
    let one = |local: &str| d.kid(ser, local).and_then(|k| outer(d, k)).unwrap_or("");
    // The start tag as it was, with any namespaces it declares.
    let mut s = d.text.get(n.outer.start..n.inner.start)?.to_string();
    for k in ["idx", "order", "tx"] {
        s.push_str(one(k));
    }
    s.push_str(&shape(d, ser, from, fam)?);
    match (fam, to) {
        (Fam::Bar, _) => {
            s.push_str(&if from == Fam::Bar { kept(d, ser, "invertIfNegative") } else { format!("<{c}invertIfNegative val=\"0\"/>") });
            s.push_str(one("pictureOptions"));
        }
        (Fam::Line, G::Line { markers, .. }) => s.push_str(&marker(d, ser, markers)),
        _ => s.push_str(one("pictureOptions")),
    }
    s.push_str(&kept(d, ser, "dPt"));
    if let Some(l) = d.kid(ser, "dLbls") {
        s.push_str(&without(d, l, &["dLblPos"])?);
    }
    s.push_str(&kept(d, ser, "trendline"));
    // An area series may have two error bars (both directions), the others one.
    s.push_str(&if fam == Fam::Area { kept(d, ser, "errBars") } else { one("errBars").to_string() });
    s.push_str(one("cat"));
    s.push_str(one("val"));
    match fam {
        Fam::Bar => s.push_str(one("shape")),
        Fam::Line => s.push_str(&kept_or(d, ser, "smooth", format!("<{c}smooth val=\"0\"/>"))),
        Fam::Area => {}
    }
    s.push_str(one("extLst"));
    s.push_str(d.text.get(n.inner.end..n.outer.end)?);
    Some(s)
}

/// The marker of a line series: none for a line without markers; for one with markers, a kept
/// marker that shows (an automatic one otherwise).
fn marker(d: &Doc<'_>, ser: usize, markers: bool) -> String {
    let c = d.prefix(ser);
    let kept = d.kid(ser, "marker");
    let hidden = kept.and_then(|m| d.kid(m, "symbol")).and_then(|s| d.attr(s, "val")) == Some("none");
    match kept.and_then(|m| outer(d, m)) {
        _ if !markers => format!("<{c}marker><{c}symbol val=\"none\"/></{c}marker>"),
        Some(m) if !hidden => m.to_string(),
        _ => String::new(),
    }
}

/// The shape properties of a series for its new type. A filled bar or area turned into a line
/// draws the line in its fill colour (a bar's outline is often none or white); a line turned into
/// a bar or area is filled with its line colour.
fn shape(d: &Doc<'_>, ser: usize, from: Fam, to: Fam) -> Option<String> {
    let Some(sp) = d.kid(ser, "spPr") else { return Some(String::new()) };
    let whole = outer(d, sp)?;
    if (from == Fam::Line) == (to == Fam::Line) {
        return Some(whole.to_string());
    }
    let fill = d.node(sp)?.kids.iter().copied().find(|k| FILLS.contains(&local(d, *k)));
    let ln = d.kid(sp, "ln");
    let edited = if to == Fam::Line { line_from_fill(d, sp, fill, ln) } else { fill_from_line(d, sp, fill, ln) };
    Some(edited.unwrap_or_else(|| whole.to_string()))
}

/// `None` when the shape properties stay as they are.
fn line_from_fill(d: &Doc<'_>, sp: usize, fill: Option<usize>, ln: Option<usize>) -> Option<String> {
    let f = fill.filter(|f| local(d, *f) == "solidFill")?;
    let line_fill = ln.and_then(|l| d.node(l)?.kids.iter().copied().find(|k| FILLS.contains(&local(d, *k))));
    if line_fill.is_some_and(|k| local(d, k) != "noFill") {
        // An outline of its own colour: the line keeps it.
        return None;
    }
    let a = d.prefix(f);
    let rest: String = match ln {
        Some(l) => d.node(l)?.kids.iter().copied().filter(|k| !FILLS.contains(&local(d, *k))).filter_map(|k| outer(d, k)).collect(),
        None => format!("<{a}round/>"),
    };
    let new = format!("<{a}ln w=\"28575\" cap=\"rnd\">{}{rest}</{a}ln>", outer(d, f)?);
    // The line follows the fill (CT_ShapeProperties).
    let range = match ln {
        Some(l) => d.node(l)?.outer.clone(),
        None => at(d.node(f)?.outer.end),
    };
    splice(d, sp, range, &new)
}

/// `None` when the shape properties stay as they are.
fn fill_from_line(d: &Doc<'_>, sp: usize, fill: Option<usize>, ln: Option<usize>) -> Option<String> {
    if fill.is_some() {
        return None;
    }
    let l = ln?;
    let line_fill = d.kid(l, "solidFill")?;
    splice(d, sp, at(d.node(l)?.outer.start), outer(d, line_fill)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::parse;

    const HEAD: &str = r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:chart>"#;
    const AXES: &str = r#"<c:catAx><c:axId val="7"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="b"/><c:crossAx val="8"/></c:catAx><c:valAx><c:axId val="8"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="l"/><c:crossAx val="7"/></c:valAx>"#;
    const SER: &str = r#"<c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:v>Sales</c:v></c:tx><c:spPr><a:solidFill><a:srgbClr val="4472C4"/></a:solidFill><a:ln><a:noFill/><a:prstDash val="dash"/></a:ln></c:spPr><c:invertIfNegative val="0"/><c:dPt><c:idx val="1"/><c:invertIfNegative val="0"/><c:bubble3D val="0"/><c:spPr><a:solidFill><a:srgbClr val="C00000"/></a:solidFill></c:spPr></c:dPt><c:dLbls><c:dLblPos val="outEnd"/><c:showVal val="1"/></c:dLbls><c:val><c:numLit><c:ptCount val="2"/><c:pt idx="0"><c:v>1</c:v></c:pt><c:pt idx="1"><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser>"#;

    /// A column chart with a formatted title, a legend with a manual layout, and a formatted series.
    fn columns() -> String {
        format!(
            r#"{HEAD}<c:title><c:tx><c:rich><a:bodyPr rot="0"/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="1800" b="1"/></a:pPr><a:r><a:rPr lang="en-US" sz="1800" b="1"/><a:t>Old</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/></c:title><c:autoTitleDeleted val="0"/><c:plotArea><c:layout/><c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/>{SER}<c:gapWidth val="40"/><c:overlap val="-20"/><c:axId val="7"/><c:axId val="8"/></c:barChart>{AXES}</c:plotArea><c:legend><c:legendPos val="r"/><c:layout><c:manualLayout><c:x val="0.8"/></c:manualLayout></c:layout><c:overlay val="0"/><c:txPr><a:bodyPr/><a:p><a:pPr><a:defRPr sz="900"/></a:pPr></a:p></c:txPr></c:legend><c:plotVisOnly val="1"/></c:chart></c:chartSpace>"#
        )
    }

    fn read(xml: &str) -> Chart {
        read_chart(&parse(xml.as_bytes()).unwrap()).unwrap()
    }

    /// `xml` with `new`'s title, legend and type, checked to read back as `new`.
    fn edited(xml: &str, edit: impl Fn(&mut Chart)) -> String {
        let old = read(xml);
        let mut new = old.clone();
        edit(&mut new);
        let edits = edit_fields(xml.as_bytes(), &old, &new).expect("edits");
        let out = String::from_utf8(super::super::chart_patch::apply(xml.as_bytes(), edits).unwrap()).unwrap();
        assert!(reads_as(out.as_bytes(), &new), "{out}");
        out
    }

    #[test]
    fn a_new_title_keeps_its_font() {
        let out = edited(&columns(), |c| c.title = Some("New & improved\nQ2".into()));
        assert!(out.contains(r#"<a:p><a:pPr><a:defRPr sz="1800" b="1"/></a:pPr><a:r><a:rPr lang="en-US" sz="1800" b="1"/><a:t>New &amp; improved</a:t></a:r></a:p>"#), "{out}");
        assert!(out.contains(r#"<a:bodyPr rot="0"/>"#) && !out.contains(">Old<"));
        assert_eq!(read(&out).title.as_deref(), Some("New & improved\nQ2"));
        // Two title paragraphs and the legend's.
        assert_eq!(out.matches("<a:p>").count(), 3);
    }

    #[test]
    fn titles_are_removed_typed_over_a_cell_and_added() {
        let out = edited(&columns(), |c| c.title = None);
        assert!(!out.contains("<c:title>") && out.contains(r#"<c:autoTitleDeleted val="1"/><c:plotArea>"#), "{out}");
        // A title from a cell becomes rich text in the title's own font.
        let from_cell = columns().replace(
            r#"<c:tx><c:rich><a:bodyPr rot="0"/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="1800" b="1"/></a:pPr><a:r><a:rPr lang="en-US" sz="1800" b="1"/><a:t>Old</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/>"#,
            r#"<c:tx><c:strRef><c:f>Notes!$A$1</c:f><c:strCache><c:ptCount val="1"/><c:pt idx="0"><c:v>Old</c:v></c:pt></c:strCache></c:strRef></c:tx><c:overlay val="0"/><c:txPr><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="2000"/></a:pPr><a:endParaRPr lang="en-US"/></a:p></c:txPr>"#,
        );
        let out = edited(&from_cell, |c| c.title = Some("Typed".into()));
        assert!(!out.contains("Notes!"), "{out}");
        assert!(out.contains(r#"<c:rich xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="2000"/></a:pPr><a:r><a:t>Typed</a:t></a:r></a:p></c:rich>"#), "{out}");
        // A chart without a title gets one in front.
        let untitled = edited(&columns(), |c| c.title = None);
        let out = edited(&untitled, |c| c.title = Some("Back".into()));
        assert!(out.contains(r#"<c:chart><c:title><c:tx><c:rich"#) && out.contains(r#"<c:autoTitleDeleted val="0"/>"#), "{out}");
    }

    #[test]
    fn a_moved_legend_keeps_its_font_and_drops_its_manual_layout() {
        let out = edited(&columns(), |c| c.legend = LegendPos::Bottom);
        assert!(out.contains(r#"<c:legend><c:legendPos val="b"/><c:overlay val="0"/><c:txPr><a:bodyPr/><a:p><a:pPr><a:defRPr sz="900"/>"#), "{out}");
        assert!(!out.contains("manualLayout"));
        let none = edited(&columns(), |c| c.legend = LegendPos::None);
        assert!(!none.contains("<c:legend>"));
        let back = edited(&none, |c| c.legend = LegendPos::Top);
        assert!(back.contains(r#"</c:plotArea><c:legend><c:legendPos val="t"/><c:overlay val="0"/></c:legend><c:plotVisOnly"#), "{back}");
    }

    #[test]
    fn columns_turned_into_a_line_keep_the_series_formatting() {
        let out = edited(&columns(), |c| c.kind = ChartKind::Line);
        assert!(out.contains("<c:lineChart><c:grouping val=\"standard\"/><c:varyColors val=\"0\"/><c:ser>"), "{out}");
        // The line takes the fill colour, keeping its dash; points and labels stay, the label
        // position goes.
        assert!(
            out.contains(r#"<c:spPr><a:solidFill><a:srgbClr val="4472C4"/></a:solidFill><a:ln w="28575" cap="rnd"><a:solidFill><a:srgbClr val="4472C4"/></a:solidFill><a:prstDash val="dash"/></a:ln></c:spPr><c:marker><c:symbol val="none"/></c:marker><c:dPt>"#),
            "{out}"
        );
        assert!(out.contains(r#"<c:dLbls><c:showVal val="1"/></c:dLbls>"#) && !out.contains("dLblPos"));
        assert!(!out.contains("invertIfNegative val=\"0\"/><c:dPt>") && !out.contains("gapWidth"));
        assert!(out.contains(r#"</c:val><c:smooth val="0"/></c:ser><c:marker val="1"/><c:axId val="7"/><c:axId val="8"/></c:lineChart>"#), "{out}");
        // Title and legend untouched; the axes stay where they were.
        assert!(out.contains(">Old<") && out.contains("manualLayout") && out.contains(r#"<c:axPos val="b"/>"#));
        assert!(edited(&columns(), |c| c.kind = ChartKind::LineMarkers).contains("</c:spPr><c:dPt>"));
    }

    #[test]
    fn columns_turned_into_bars_or_areas() {
        let bars = edited(&columns(), |c| c.kind = ChartKind::BarStacked);
        assert!(bars.contains(r#"<c:barChart><c:barDir val="bar"/><c:grouping val="stacked"/>"#), "{bars}");
        assert!(bars.contains(r#"<c:gapWidth val="40"/><c:overlap val="100"/><c:axId"#), "{bars}");
        // The category axis moves to the left, the value axis to the bottom.
        assert!(
            bars.contains(r#"<c:axId val="7"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="l"/>"#),
            "{bars}"
        );
        assert!(
            bars.contains(r#"<c:axId val="8"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="b"/>"#),
            "{bars}"
        );
        assert!(bars.contains(r#"<a:ln><a:noFill/><a:prstDash val="dash"/></a:ln>"#));
        let area = edited(&columns(), |c| c.kind = ChartKind::AreaStacked100);
        assert!(
            area.contains(r#"<c:areaChart><c:grouping val="percentStacked"/>"#) && !area.contains("invertIfNegative val=\"0\"/><c:dPt>"),
            "{area}"
        );
        assert!(area.contains(r#"<a:ln><a:noFill/><a:prstDash val="dash"/></a:ln></c:spPr><c:dPt>"#), "{area}");
    }

    #[test]
    fn a_line_turned_into_columns_is_filled_with_its_colour() {
        let line = edited(
            &columns().replace(
                r#"<a:solidFill><a:srgbClr val="4472C4"/></a:solidFill><a:ln><a:noFill/>"#,
                "<a:ln w=\"9525\"><a:solidFill><a:srgbClr val=\"00B050\"/></a:solidFill>",
            ),
            |c| c.kind = ChartKind::LineMarkers,
        );
        let out = edited(&line, |c| c.kind = ChartKind::ColumnClustered);
        assert!(
            out.contains(r#"<c:spPr><a:solidFill><a:srgbClr val="00B050"/></a:solidFill><a:ln w="9525"><a:solidFill><a:srgbClr val="00B050"/></a:solidFill><a:prstDash val="dash"/></a:ln></c:spPr><c:invertIfNegative val="0"/>"#),
            "{out}"
        );
        assert!(out.contains(r#"<c:gapWidth val="150"/><c:axId"#) && !out.contains("<c:marker") && !out.contains("smooth"), "{out}");
    }

    #[test]
    fn other_types_are_left_to_the_model() {
        let old = read(&columns());
        for kind in [ChartKind::Pie, ChartKind::Scatter, ChartKind::Combo, ChartKind::Radar] {
            let new = Chart { kind, ..old.clone() };
            assert!(edit_fields(columns().as_bytes(), &old, &new).is_none(), "{kind:?}");
        }
        // A 3-D group.
        let three_d = columns().replace("barChart>", "bar3DChart>");
        let old = read(&three_d);
        assert!(edit_fields(three_d.as_bytes(), &old, &Chart { kind: ChartKind::Line, ..old.clone() }).is_none());
    }
}
