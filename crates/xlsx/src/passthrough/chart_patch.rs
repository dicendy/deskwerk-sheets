//! Bringing a kept chart part (#1668) up to date with its model chart, so a chart whose series
//! references moved (rows or columns inserted or deleted, cells cut and pasted, a sheet renamed)
//! or whose cells changed keeps its formatting on save (#1676, #1677). Only the series formulas
//! (`<c:f>`), a literal series name and the caches behind them change; every other byte stays as
//! it was read. [`patch`] gives `None` when the kept XML can't follow the model; the chart is then
//! written from the model.

use std::ops::Range;

use deskwerk_sheets_core::Value;
use deskwerk_sheets_model::Series;

use super::write::{same_sheet, sheet_names_in};
use super::*;
use crate::chart::{PLOT_GROUPS, chart_ref_to_file, is_defined_name, is_ref, ref_values};

/// A replacement of a byte range of the kept part.
pub(super) type Edit = (Range<usize>, String);

/// An element of the kept part.
pub(super) struct Node {
    /// The qualified name (`c:numCache`), for writing a replacement with the same prefix.
    pub(super) qname: String,
    pub(super) local: String,
    pub(super) attrs: Vec<(String, String)>,
    /// From the `<` of the start tag to the `>` of the end tag.
    pub(super) outer: Range<usize>,
    /// Between the tags; empty for an empty element.
    pub(super) inner: Range<usize>,
    pub(super) empty: bool,
    pub(super) kids: Vec<usize>,
}

/// The elements of a part with their byte ranges, the root first.
pub(super) struct Doc<'a> {
    pub(super) text: &'a str,
    pub(super) nodes: Vec<Node>,
}

fn new_node(e: &BytesStart<'_>, outer: Range<usize>, empty: bool) -> Node {
    let lossy = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    let attrs = e
        .attributes()
        .with_checks(false)
        .flatten()
        .map(|a| {
            let key = a.key.as_ref();
            let local = key.iter().rposition(|&b| b == b':').and_then(|i| key.get(i + 1..)).unwrap_or(key);
            (lossy(local), lossy(a.value.as_ref()))
        })
        .collect();
    let end = outer.end;
    Node { qname: lossy(e.name().as_ref()), local: lossy(e.local_name().as_ref()), attrs, outer, inner: end..end, empty, kids: vec![] }
}

impl<'a> Doc<'a> {
    /// `None` when the part is not well-formed, nests too deeply or holds too many elements.
    pub(super) fn parse(text: &'a str) -> Option<Doc<'a>> {
        let mut reader = Reader::from_str(text);
        let mut nodes: Vec<Node> = vec![];
        let mut stack: Vec<usize> = vec![];
        loop {
            let start = usize::try_from(reader.buffer_position()).ok()?;
            let ev = reader.read_event().ok()?;
            let end = usize::try_from(reader.buffer_position()).ok()?;
            let (e, empty) = match ev {
                Event::Start(e) => (e, false),
                Event::Empty(e) => (e, true),
                Event::End(_) => {
                    let n = nodes.get_mut(stack.pop()?)?;
                    n.inner.end = start;
                    n.outer.end = end;
                    continue;
                }
                Event::Eof => break,
                _ => continue,
            };
            if stack.len() >= xml::MAX_DEPTH || nodes.len() >= xml::MAX_XML_NODES {
                return None;
            }
            let i = nodes.len();
            match stack.last() {
                Some(&p) => nodes.get_mut(p)?.kids.push(i),
                // A second root element.
                None if i > 0 => return None,
                None => {}
            }
            nodes.push(new_node(&e, start..end, empty));
            if !empty {
                stack.push(i);
            }
        }
        (stack.is_empty() && !nodes.is_empty()).then_some(Doc { text, nodes })
    }

    pub(super) fn node(&self, i: usize) -> Option<&Node> {
        self.nodes.get(i)
    }

    /// The first child of `i` with the local name `local`.
    pub(super) fn kid(&self, i: usize, local: &str) -> Option<usize> {
        self.kids(i, local).next()
    }

    pub(super) fn kids<'s>(&'s self, i: usize, local: &'s str) -> impl Iterator<Item = usize> + 's {
        self.node(i).map(|n| n.kids.as_slice()).unwrap_or_default().iter().copied().filter(move |k| self.node(*k).is_some_and(|n| n.local == local))
    }

    pub(super) fn attr(&self, i: usize, key: &str) -> Option<&str> {
        self.node(i)?.attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// The text of an element, entities resolved.
    pub(super) fn text_of(&self, i: usize) -> Option<String> {
        self.text.get(self.node(i)?.inner.clone()).map(unescape)
    }

    /// The qualified-name prefix of an element with its colon (`c:`), or nothing.
    pub(super) fn prefix(&self, i: usize) -> &str {
        self.node(i).and_then(|n| n.qname.rfind(':').and_then(|c| n.qname.get(..=c))).unwrap_or("")
    }

    /// The edit that sets the text of element `i` to the escaped `text`.
    pub(super) fn set_text(&self, i: usize, text: &str) -> Option<Edit> {
        let n = self.node(i)?;
        if n.empty {
            let q = &n.qname;
            // `<c:v/>`: write it out whole, without the attributes it never has in a chart.
            return Some((n.outer.clone(), format!("<{q}>{}</{q}>", xml::esc(text))));
        }
        Some((n.inner.clone(), xml::esc(text)))
    }
}

/// Resolves the predefined and character entities; anything else stays as written.
fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(rest.get(..amp).unwrap_or(""));
        let tail = rest.get(amp..).unwrap_or("");
        let entity = tail.find(';').and_then(|semi| tail.get(1..semi).map(|name| (name, semi)));
        let resolved = entity.and_then(|(name, semi)| {
            let c = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => name
                    .strip_prefix("#x")
                    .map(|h| u32::from_str_radix(h, 16))
                    .or_else(|| name.strip_prefix('#').map(str::parse::<u32>))
                    .and_then(Result::ok)
                    .and_then(char::from_u32),
            };
            c.map(|c| (c, semi))
        });
        match resolved {
            Some((c, semi)) => {
                out.push(c);
                rest = tail.get(semi + 1..).unwrap_or("");
            }
            None => {
                out.push('&');
                rest = tail.get(1..).unwrap_or("");
            }
        }
    }
    out.push_str(rest);
    out
}

/// Where a series keeps one of its data (name, categories, values, bubble sizes).
#[derive(Clone, Copy)]
enum Slot {
    Absent,
    /// A literal (`numLit`, `strLit`, or the `<c:v>` of a name).
    Lit(usize),
    /// A reference element (`numRef`…) with its formula and the local name of its cache.
    Ref {
        el: usize,
        f: usize,
        cache: &'static str,
    },
}

impl Doc<'_> {
    /// The data under `holder` (`<c:cat>`, `<c:val>`…), as `chart::data_ref` reads it.
    fn data_slot(&self, holder: Option<usize>) -> Slot {
        let Some(h) = holder else { return Slot::Absent };
        for (k, cache) in [("numRef", "numCache"), ("strRef", "strCache"), ("multiLvlStrRef", "multiLvlStrCache")] {
            if let Some(el) = self.kid(h, k)
                && let Some(f) = self.kid(el, "f")
            {
                return Slot::Ref { el, f, cache };
            }
        }
        ["numLit", "strLit"].iter().find_map(|k| self.kid(h, k)).map_or(Slot::Absent, Slot::Lit)
    }

    /// A series name (`<c:tx>`), as `chart::read_series` reads it.
    fn name_slot(&self, tx: Option<usize>) -> Slot {
        let Some(tx) = tx else { return Slot::Absent };
        if let Some(el) = self.kid(tx, "strRef")
            && let Some(f) = self.kid(el, "f")
        {
            return Slot::Ref { el, f, cache: "strCache" };
        }
        self.kid(tx, "v").map_or(Slot::Absent, Slot::Lit)
    }

    /// The first of two data holders that holds data (`cat`, else `xVal`).
    fn either(&self, ser: usize, a: &str, b: &str) -> Slot {
        match self.data_slot(self.kid(ser, a)) {
            Slot::Absent => self.data_slot(self.kid(ser, b)),
            s => s,
        }
    }
}

struct Patcher<'d, 'a> {
    wb: &'a Workbook,
    si: usize,
    doc: &'d Doc<'a>,
    edits: Vec<Edit>,
    /// The formulas of the series slots.
    handled: HashSet<usize>,
    /// Sheets the changed series formulas named before or after; `None` in the list for a
    /// formula without a sheet.
    moved: Vec<Option<String>>,
}

fn present(s: Option<&str>) -> Option<&str> {
    s.filter(|s| !s.trim().is_empty())
}

impl Patcher<'_, '_> {
    fn series(&mut self, ser: usize, old: &Series, new: &Series) -> Option<()> {
        let d = self.doc;
        self.name(d.name_slot(d.kid(ser, "tx")), old.name.as_deref(), new.name.as_deref())?;
        self.data(d.either(ser, "cat", "xVal"), old.categories.as_deref(), new.categories.as_deref())?;
        self.data(d.either(ser, "val", "yVal"), Some(old.values.as_str()), Some(new.values.as_str()))?;
        self.data(d.data_slot(d.kid(ser, "bubbleSize")), old.bubble_sizes.as_deref(), new.bubble_sizes.as_deref())
    }

    fn name(&mut self, slot: Slot, old: Option<&str>, new: Option<&str>) -> Option<()> {
        let is_reference = |n: &str| is_ref(n) || is_defined_name(self.wb, self.si, n);
        match slot {
            Slot::Absent => (old.is_none() && new.is_none()).then_some(()),
            Slot::Lit(v) => {
                let new = new?;
                if old == Some(new) {
                    return Some(());
                }
                if is_reference(new) {
                    return None;
                }
                // As `chart::series_xml` writes a literal name.
                let lit = new.trim().strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(new);
                self.edits.push(self.doc.set_text(v, lit)?);
                Some(())
            }
            Slot::Ref { el, f, cache } => {
                let new = new?;
                if old != Some(new) && !is_reference(new) {
                    return None;
                }
                self.reference(el, f, cache, old, new)
            }
        }
    }

    fn data(&mut self, slot: Slot, old: Option<&str>, new: Option<&str>) -> Option<()> {
        let (old, new) = (present(old), present(new));
        match slot {
            Slot::Absent => (old.is_none() && new.is_none()).then_some(()),
            Slot::Lit(_) => (old.is_some() && old == new).then_some(()),
            Slot::Ref { el, f, cache } => {
                let new = new.filter(|n| !n.trim_start().starts_with('{'))?;
                self.reference(el, f, cache, old, new)
            }
        }
    }

    /// A series formula: rewritten when the model's moved, its cache refreshed from the cells.
    fn reference(&mut self, el: usize, f: usize, cache: &'static str, old: Option<&str>, new: &str) -> Option<()> {
        let old = old?;
        self.handled.insert(f);
        let changed = old != new;
        if changed {
            let file = chart_ref_to_file(self.wb, self.si, new);
            let before = self.doc.text_of(f)?;
            for formula in [before.as_str(), file.as_str()] {
                let names = sheet_names_in(formula);
                if names.is_empty() {
                    self.moved.push(None);
                }
                self.moved.extend(names.into_iter().map(Some));
            }
            self.edits.push(self.doc.set_text(f, &file)?);
        }
        let c = self.doc.kid(el, cache);
        let values = if cache == "multiLvlStrCache" { None } else { ref_values(self.wb, self.si, new) };
        match (c, values) {
            (Some(c), Some(values)) => self.refresh(c, cache == "numCache", &values),
            // A cache that can't be rebuilt (multi-level categories, a name, too many cells) is
            // left for the reader to refresh when the formula still says the same.
            (Some(c), None) if changed => self.edits.push((self.doc.node(c)?.outer.clone(), String::new())),
            _ => {}
        }
        Some(())
    }

    /// Rewrites a number or text cache that no longer matches `values`.
    fn refresh(&mut self, c: usize, num: bool, values: &[Value]) {
        let d = self.doc;
        let pts: Vec<(usize, String)> = values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                Value::Number(n) if num => Some((i, xml::num(*n))),
                _ if num => None,
                v => (!v.is_empty()).then(|| (i, v.display())),
            })
            .collect();
        if self.cache_matches(c, values.len(), &pts, num) {
            return;
        }
        let Some(node) = d.node(c) else { return };
        let p = d.prefix(c);
        let q = &node.qname;
        let mut s = format!("<{q}>");
        if num {
            match d.kid(c, "formatCode").and_then(|k| d.node(k)).and_then(|k| d.text.get(k.outer.clone())) {
                Some(code) => s.push_str(code),
                None => {
                    let _ = write!(s, "<{p}formatCode>General</{p}formatCode>");
                }
            }
        }
        let _ = write!(s, "<{p}ptCount val=\"{}\"/>", values.len());
        for (i, v) in &pts {
            let _ = write!(s, "<{p}pt idx=\"{i}\"><{p}v>{}</{p}v></{p}pt>", xml::esc(v));
        }
        let _ = write!(s, "</{q}>");
        self.edits.push((node.outer.clone(), s));
    }

    /// Whether a kept cache already holds these points (numbers compared as numbers).
    fn cache_matches(&self, c: usize, count: usize, pts: &[(usize, String)], num: bool) -> bool {
        let d = self.doc;
        let kept_count = d.kid(c, "ptCount").and_then(|k| d.attr(k, "val")).and_then(|v| v.trim().parse::<usize>().ok());
        if kept_count != Some(count) {
            return false;
        }
        let kept: Vec<(Option<usize>, Option<String>)> =
            d.kids(c, "pt").map(|p| (d.attr(p, "idx").and_then(|v| v.trim().parse().ok()), d.kid(p, "v").and_then(|v| d.text_of(v)))).collect();
        let same = |a: &str, b: &str| if num { a.trim().parse::<f64>().ok() == b.parse::<f64>().ok() } else { a == b };
        kept.len() == pts.len() && kept.iter().zip(pts).all(|((ki, kv), (i, v))| *ki == Some(*i) && kv.as_deref().is_some_and(|kv| same(kv, v)))
    }

    /// Whether a formula outside the series (a title or label from a cell, error bars) may have
    /// moved with the series: it names a sheet a changed series formula named. Those formulas
    /// aren't in the model, so they can't be followed.
    fn others_may_have_moved(&self) -> bool {
        if self.moved.is_empty() {
            return false;
        }
        let d = self.doc;
        d.nodes.iter().enumerate().filter(|(i, n)| n.local == "f" && !self.handled.contains(i)).any(|(i, _)| {
            let names = d.text_of(i).map(|t| sheet_names_in(&t)).unwrap_or_default();
            self.moved.iter().any(|m| m.as_ref().is_none_or(|m| names.iter().any(|n| same_sheet(n, m))))
        })
    }
}

/// The edits that bring kept chart XML `data`, read as `old`, up to `new` (which differs from
/// `old` at most in its series references); no edits when nothing changed. `si` is the index of
/// the sheet the chart is on. `None` when the XML can't follow: a series changed between a
/// reference and a literal, a literal changed, the XML has other series than the model, or a
/// formula the model doesn't hold may have moved with the series.
pub(super) fn patch(wb: &Workbook, si: usize, data: &[u8], old: &Chart, new: &Chart) -> Option<Vec<Edit>> {
    let doc = Doc::parse(std::str::from_utf8(data).ok()?)?;
    let plot = doc.kid(doc.kid(0, "chart")?, "plotArea")?;
    let groups = doc.node(plot)?.kids.iter().copied().filter(|g| doc.node(*g).is_some_and(|n| PLOT_GROUPS.contains(&n.local.as_str())));
    let sers: Vec<usize> = groups.flat_map(|g| doc.kids(g, "ser")).collect();
    if sers.len() != old.series.len() || sers.len() != new.series.len() {
        return None;
    }
    let mut p = Patcher { wb, si, doc: &doc, edits: vec![], handled: HashSet::new(), moved: vec![] };
    for ((s, o), n) in sers.iter().zip(&old.series).zip(&new.series) {
        p.series(*s, o, n)?;
    }
    if p.others_may_have_moved() {
        return None;
    }
    Some(p.edits)
}

/// `data` with the edits made; `None` when two overlap.
pub(super) fn apply(data: &[u8], mut edits: Vec<Edit>) -> Option<Vec<u8>> {
    edits.sort_by_key(|(r, _)| r.start);
    let mut out = Vec::with_capacity(data.len());
    let mut at = 0;
    for (r, s) in &edits {
        out.extend_from_slice(data.get(at..r.start)?);
        out.extend_from_slice(s.as_bytes());
        at = r.end;
    }
    out.extend_from_slice(data.get(at..)?);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_resolves_entities_and_leaves_stray_ampersands() {
        assert_eq!(unescape("A &amp; B &lt;&#65;&#x42;&apos;&quot;&gt;"), "A & B <AB'\">");
        assert_eq!(unescape("a & b &bogus; &#xZZ;"), "a & b &bogus; &#xZZ;");
    }

    #[test]
    fn the_index_refuses_malformed_xml() {
        assert!(Doc::parse("<a><b></a>").is_none());
        assert!(Doc::parse("<a/><b/>").is_none());
        assert!(Doc::parse("<a>").is_none());
        let d = Doc::parse("<?xml version=\"1.0\"?><x:a><!-- <b> --><x:b k=\"1\"/>t</x:a>").unwrap();
        assert_eq!(d.nodes.len(), 2);
        assert_eq!(d.prefix(1), "x:");
        assert_eq!(d.attr(1, "k"), Some("1"));
        assert_eq!(d.text.get(d.nodes[1].outer.clone()), Some("<x:b k=\"1\"/>"));
    }

    #[test]
    fn overlapping_edits_are_refused() {
        assert_eq!(apply(b"abcdef", vec![(4..5, "E".into()), (0..2, "AB".into())]).unwrap(), b"ABcdEf");
        assert!(apply(b"abcdef", vec![(0..3, String::new()), (2..4, String::new())]).is_none());
    }
}
