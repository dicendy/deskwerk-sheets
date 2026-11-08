//! German and French notation: the short date built-ins as a German or French Excel shows them,
//! and format codes as such an Excel reads them in `TEXT()`.

use gridcraft_core::{DateSystem, Locale};

use crate::{NumberFormat, builtin_id};

/// The format a German or French Excel shows the short date built-ins with: id 14 (`m/d/yyyy`)
/// and id 22 (`m/d/yyyy h:mm`) follow the system's date order (`31.12.2026`, `31/12/2026`).
/// `None` for every other format and for en-US.
pub(crate) fn short_date(fmt: &NumberFormat, loc: Locale) -> Option<NumberFormat> {
    if loc == Locale::En || !fmt.is_date() {
        return None;
    }
    let code = match (builtin_id(fmt.code())?, loc) {
        (14, Locale::De) => "dd.mm.yyyy",
        (14, Locale::Fr) => "dd/mm/yyyy",
        (22, Locale::De) => "dd.mm.yyyy hh:mm",
        (22, Locale::Fr) => "dd/mm/yyyy hh:mm",
        _ => return None,
    };
    Some(NumberFormat::parse_cached(code))
}

/// Numeric text as `TEXT()` converts it: en-US reads it as before, German and French with the
/// locale's separators and day-first dates.
pub(crate) fn number_text(t: &str, sys: DateSystem, loc: Locale) -> Option<f64> {
    match loc {
        Locale::En => gridcraft_core::parse::parse_number_text(t),
        _ => gridcraft_core::parse::parse_number_text_local(t, sys, loc),
    }
}

/// Colour names of the locale's `[Rot]`/`[Rouge]` tags with their en-US names.
fn color_names(loc: Locale) -> &'static [(&'static str, &'static str)] {
    match loc {
        Locale::En => &[],
        Locale::De => &[
            ("schwarz", "Black"),
            ("blau", "Blue"),
            ("zyan", "Cyan"),
            ("grün", "Green"),
            ("magenta", "Magenta"),
            ("rot", "Red"),
            ("weiß", "White"),
            ("gelb", "Yellow"),
        ],
        Locale::Fr => &[
            ("noir", "Black"),
            ("bleu", "Blue"),
            ("cyan", "Cyan"),
            ("vert", "Green"),
            ("magenta", "Magenta"),
            ("rouge", "Red"),
            ("blanc", "White"),
            ("jaune", "Yellow"),
        ],
    }
}

/// The en-US content of a `[...]` tag written in `loc`: colours (`Rot` → `Red`, `Farbe10` /
/// `Couleur10` → `Color10`); anything else (conditions, `[h]`, `[$€-407]`) is kept.
fn bracket_content(content: &str, loc: Locale) -> String {
    let lower = content.trim().to_lowercase();
    if let Some((_, en)) = color_names(loc).iter().find(|(name, _)| *name == lower) {
        return (*en).to_string();
    }
    let prefix = match loc {
        Locale::De => "farbe",
        Locale::Fr => "couleur",
        Locale::En => return content.to_string(),
    };
    match lower.strip_prefix(prefix) {
        Some(n) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => format!("Color{n}"),
        _ => content.to_string(),
    }
}

/// True for a digit placeholder.
fn is_placeholder(c: Option<char>) -> bool {
    matches!(c, Some('0' | '#' | '?'))
}

/// One section of a localized code, before the separators are put right: the letters mapped to
/// en-US and whether it is a date or time section.
struct Mapped {
    /// Pieces: `true` for copied verbatim (quoted text, escapes, tags), `false` for code text.
    pieces: Vec<(bool, String)>,
    date: bool,
}

/// Maps one section's letters (`T`/`J` in German, `j`/`a` in French) and its `Standard` keyword.
fn map_section(sec: &[char], loc: Locale) -> Mapped {
    let mut pieces: Vec<(bool, String)> = Vec::new();
    let mut date = false;
    let mut code = String::new();
    let mut i = 0;
    let starts_ci = |i: usize, word: &str| {
        let mut n = 0;
        word.chars().all(|w| {
            let ok = sec.get(i + n).is_some_and(|c| c.to_lowercase().eq(w.to_lowercase()));
            n += 1;
            ok
        })
    };
    while let Some(&c) = sec.get(i) {
        let verbatim = |pieces: &mut Vec<(bool, String)>, code: &mut String, s: String| {
            if !code.is_empty() {
                pieces.push((false, std::mem::take(code)));
            }
            pieces.push((true, s));
        };
        match c {
            '"' => {
                let end = sec.get(i + 1..).and_then(|r| r.iter().position(|&d| d == '"')).map_or(sec.len(), |p| i + 2 + p);
                verbatim(&mut pieces, &mut code, sec.get(i..end).unwrap_or(&[]).iter().collect());
                i = end;
                continue;
            }
            '\\' | '_' | '*' => {
                let end = (i + 2).min(sec.len());
                verbatim(&mut pieces, &mut code, sec.get(i..end).unwrap_or(&[]).iter().collect());
                i = end;
                continue;
            }
            '[' => {
                if let Some(p) = sec.get(i + 1..).and_then(|r| r.iter().position(|&d| d == ']')) {
                    let content: String = sec.get(i + 1..i + 1 + p).unwrap_or(&[]).iter().collect();
                    let lc = content.to_lowercase();
                    if lc.starts_with(['h', 'm', 's']) {
                        date = true;
                    }
                    verbatim(&mut pieces, &mut code, format!("[{}]", bracket_content(&content, loc)));
                    i += p + 2;
                    continue;
                }
                code.push(c);
            }
            _ if starts_ci(i, "standard") => {
                code.push_str("General");
                i += "standard".len();
                continue;
            }
            'A' | 'a' if starts_ci(i, "am/pm") || starts_ci(i, "a/p") => {
                // Kept for the time of day; `a/p` is two letters and a slash.
                let n = if starts_ci(i, "am/pm") { 5 } else { 3 };
                code.extend(sec.get(i..i + n).unwrap_or(&[]).iter());
                date = true;
                i += n;
                continue;
            }
            _ => {
                let mapped = match (loc, c) {
                    (Locale::De, 'T' | 't') | (Locale::Fr, 'J' | 'j') => 'd',
                    (Locale::De, 'J' | 'j') | (Locale::Fr, 'A' | 'a') => 'y',
                    _ => c,
                };
                if matches!(mapped.to_ascii_lowercase(), 'd' | 'm' | 'y' | 'h' | 's') {
                    date = true;
                }
                code.push(mapped);
            }
        }
        i += 1;
    }
    if !code.is_empty() {
        pieces.push((false, code));
    }
    Mapped { pieces, date }
}

/// A format code as a German or French Excel reads it in `TEXT()`, written en-US (the notation
/// [`NumberFormat::parse`] reads):
///
/// - German: `,` is the decimal and `.` the thousands separator in number sections (`#.##0,00`);
///   `T` is the day and `J` the year (`TT.MM.JJJJ`); `,` before sub-seconds (`ss,00`).
/// - French: `,` is the decimal separator and a space (also no-break or U+202F) between digit
///   placeholders the thousands separator (`# ##0,00`); `.` is literal; `j` is the day and `a`
///   the year (`jj/mm/aaaa`).
/// - Both: `Standard` is General, colour tags are the locale's (`[Rot]`, `[Rouge]`,
///   `[Farbe10]`, `[Couleur10]`). Quoted text, escapes and other tags are kept as they are.
///
/// en-US codes are returned unchanged.
pub fn canonical_format_code(code: &str, loc: Locale) -> String {
    if loc == Locale::En {
        return code.to_string();
    }
    let chars: Vec<char> = code.chars().collect();
    // Split into sections at `;` outside quotes, escapes and tags.
    let mut sections: Vec<&[char]> = Vec::new();
    let (mut start, mut i, mut quoted) = (0, 0, false);
    while let Some(&c) = chars.get(i) {
        match c {
            '"' => quoted = !quoted,
            '\\' | '_' | '*' if !quoted => i += 1,
            ';' if !quoted => {
                sections.push(chars.get(start..i).unwrap_or(&[]));
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    sections.push(chars.get(start..).unwrap_or(&[]));
    let mut out = String::with_capacity(code.len() + 8);
    for (k, sec) in sections.iter().enumerate() {
        if k > 0 {
            out.push(';');
        }
        let mapped = map_section(sec, loc);
        for (verbatim, piece) in &mapped.pieces {
            if *verbatim {
                out.push_str(piece);
                continue;
            }
            let pc: Vec<char> = piece.chars().collect();
            for (j, &c) in pc.iter().enumerate() {
                let prev = j.checked_sub(1).and_then(|p| pc.get(p)).copied();
                let next = pc.get(j + 1).copied();
                match c {
                    // Sub-seconds: `ss,00`.
                    ',' if mapped.date => out.push(if next == Some('0') && prev.is_some_and(|p| p.eq_ignore_ascii_case(&'s')) { '.' } else { ',' }),
                    _ if mapped.date => out.push(c),
                    ',' => out.push('.'),
                    '.' if loc == Locale::De => out.push(','),
                    '.' => out.push_str("\\."),
                    ' ' | '\u{a0}' | '\u{202F}' if loc == Locale::Fr && is_placeholder(prev) && is_placeholder(next) => out.push(','),
                    _ => out.push(c),
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_codes() {
        let de = |c: &str| canonical_format_code(c, Locale::De);
        assert_eq!(de("#.##0,00"), "#,##0.00");
        assert_eq!(de("0,0%"), "0.0%");
        assert_eq!(de("0,00E+00"), "0.00E+00");
        assert_eq!(de("TT.MM.JJJJ"), "dd.MM.yyyy");
        assert_eq!(de("TTTT, T. MMMM JJJJ"), "dddd, d. MMMM yyyy");
        assert_eq!(de("hh:mm:ss,00"), "hh:mm:ss.00");
        assert_eq!(de("Standard"), "General");
        assert_eq!(de("[Rot]#.##0;[Blau]-#.##0"), "[Red]#,##0;[Blue]-#,##0");
        assert_eq!(de("[Farbe10]0"), "[Color10]0");
        assert_eq!(de("#.##0,00 \"T€\""), "#,##0.00 \"T€\"");
        assert_eq!(de("0,0\\T"), "0.0\\T");
        assert_eq!(de("[h]:mm"), "[h]:mm");
        assert_eq!(de("@"), "@");
    }

    #[test]
    fn french_codes() {
        let fr = |c: &str| canonical_format_code(c, Locale::Fr);
        assert_eq!(fr("# ##0,00"), "#,##0.00");
        assert_eq!(fr("#\u{202F}##0,00 €"), "#,##0.00 €");
        assert_eq!(fr("0,0 %"), "0.0 %");
        assert_eq!(fr("jj/mm/aaaa"), "dd/mm/yyyy");
        assert_eq!(fr("jjjj j mmmm aaaa"), "dddd d mmmm yyyy");
        assert_eq!(fr("h:mm AM/PM"), "h:mm AM/PM");
        assert_eq!(fr("Standard"), "General");
        assert_eq!(fr("[Rouge]0,0"), "[Red]0.0");
        assert_eq!(fr("0.0"), "0\\.0");
    }

    #[test]
    fn english_codes_are_kept() {
        for c in ["#,##0.00", "m/d/yyyy", "General", "[Red]0;\"a;b\"", ""] {
            assert_eq!(canonical_format_code(c, Locale::En), c);
        }
    }

    #[test]
    fn short_date_built_ins_follow_the_locale() {
        let short = |code: &str, loc| short_date(&NumberFormat::parse(code), loc).map(|f| f.code().to_string());
        assert_eq!(short("m/d/yyyy", Locale::De).as_deref(), Some("dd.mm.yyyy"));
        assert_eq!(short("m/d/yyyy", Locale::Fr).as_deref(), Some("dd/mm/yyyy"));
        assert_eq!(short("m/d/yyyy h:mm", Locale::De).as_deref(), Some("dd.mm.yyyy hh:mm"));
        assert_eq!(short("m/d/yyyy h:mm", Locale::Fr).as_deref(), Some("dd/mm/yyyy hh:mm"));
        assert_eq!(short("m/d/yyyy", Locale::En), None);
        assert_eq!(short("d-mmm-yy", Locale::De), None);
        assert_eq!(short("yyyy-mm-dd", Locale::Fr), None);
        assert_eq!(short("0.00", Locale::De), None);
    }
}
