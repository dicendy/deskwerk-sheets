//! The session's number locale (`app.locale.set`): German and French display, typed entry and
//! the text the cell editor starts from. The control protocol stays en-US.

use gridcraft_core::{CellRef, Locale, Value};
use serde_json::json;

use super::{s, v};
use crate::Session;

fn shown(s: &Session, a: &str) -> String {
    let d = s.doc().unwrap();
    crate::display::cell_text(&d.wb, d.wb.active().unwrap(), CellRef::parse(a).unwrap())
}

fn edit_text(s: &Session, a: &str) -> String {
    let d = s.doc().unwrap();
    crate::display::edit_text(&d.wb, d.wb.active().unwrap(), CellRef::parse(a).unwrap())
}

fn typed(s: &mut Session, cell: &str, input: &str, locale: &str) {
    s.execute("cell.set", json!({"cell": cell, "input": input, "locale": locale})).unwrap();
}

#[test]
fn english_is_the_default_and_unknown_locales_are_refused() {
    let mut s = s();
    assert_eq!(s.locale(), Locale::En);
    assert_eq!(s.doc().unwrap().wb.locale(), Locale::En);
    assert!(s.execute("app.locale.set", json!({})).is_err());
    assert!(s.execute("app.locale.set", json!({"locale": "xx"})).is_err());
    assert!(s.execute("cell.set", json!({"cell": "A1", "input": "1", "locale": "xx"})).is_err());
    assert_eq!(s.execute("app.locale.set", json!({"locale": "de-AT"})).unwrap(), json!({"locale": "de"}));
    assert_eq!(s.doc().unwrap().wb.locale(), Locale::De);
    // A workbook opened later takes it too.
    s.new_workbook();
    assert_eq!(s.doc().unwrap().wb.locale(), Locale::De);
}

#[test]
fn german_entry_display_and_editing() {
    let mut s = s();
    s.execute("app.locale.set", json!({"locale": "de"})).unwrap();
    for (cell, input) in [("A1", "1.234,56"), ("A2", "31.12.2026"), ("A3", "12,5 %"), ("A4", "1.234,56 €"), ("A5", "WAHR"), ("A6", "1,5")] {
        typed(&mut s, cell, input, "de");
    }
    assert_eq!(v(&s, "A1"), Value::Number(1234.56));
    assert_eq!(v(&s, "A2"), Value::Number(46387.0));
    assert_eq!(v(&s, "A3"), Value::Number(0.125));
    assert_eq!(v(&s, "A4"), Value::Number(1234.56));
    assert_eq!(v(&s, "A5"), Value::Bool(true));
    assert_eq!(v(&s, "A6"), Value::Number(1.5));
    assert_eq!(shown(&s, "A1"), "1.234,56");
    assert_eq!(shown(&s, "A2"), "31.12.2026");
    assert_eq!(shown(&s, "A4"), "1.234,56 €");
    assert_eq!(shown(&s, "A5"), "WAHR");
    // The editor starts from text that reads back as the same value.
    assert_eq!(edit_text(&s, "A1"), "1234,56");
    assert_eq!(edit_text(&s, "A2"), "31.12.2026");
    assert_eq!(edit_text(&s, "A5"), "WAHR");
    assert_eq!(edit_text(&s, "A6"), "1,5");
    // Text that would read as a number in German is quoted for editing.
    typed(&mut s, "B1", "'1,5", "de");
    assert_eq!(v(&s, "B1"), Value::text("1,5"));
    assert_eq!(edit_text(&s, "B1"), "'1,5");
    // Back in en-US the same cells show as before.
    s.execute("app.locale.set", json!({"locale": "en"})).unwrap();
    assert_eq!(shown(&s, "A2"), "12/31/2026");
    assert_eq!(shown(&s, "A5"), "TRUE");
    assert_eq!(edit_text(&s, "A6"), "1.5");
}

#[test]
fn french_entry_and_display() {
    let mut s = s();
    s.execute("app.locale.set", json!({"locale": "fr"})).unwrap();
    typed(&mut s, "A1", "31/12/2026", "fr");
    typed(&mut s, "A2", "1\u{202F}234,5", "fr");
    typed(&mut s, "A3", "VRAI", "fr");
    assert_eq!(v(&s, "A1"), Value::Number(46387.0));
    assert_eq!(v(&s, "A2"), Value::Number(1234.5));
    assert_eq!(v(&s, "A3"), Value::Bool(true));
    assert_eq!(shown(&s, "A1"), "31/12/2026");
    assert_eq!(shown(&s, "A2"), "1\u{202F}234,50");
    assert_eq!(shown(&s, "A3"), "VRAI");
    assert_eq!(edit_text(&s, "A2"), "1234,5");
}

#[test]
fn the_control_protocol_types_in_english_whatever_the_locale() {
    let mut s = s();
    s.execute("app.locale.set", json!({"locale": "de"})).unwrap();
    s.execute("cell.set", json!({"cell": "A1", "input": "1.5"})).unwrap();
    s.execute("cell.set", json!({"cell": "A2", "input": "1,5"})).unwrap();
    s.execute("cell.set", json!({"cell": "A3", "input": "=SUM(1.5,2)"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1.5));
    assert_eq!(v(&s, "A2"), Value::text("1,5"));
    assert_eq!(v(&s, "A3"), Value::Number(3.5));
    // Formulas are canonical even with a locale.
    typed(&mut s, "A4", "=SUM(1.5,2)", "de");
    assert_eq!(v(&s, "A4"), Value::Number(3.5));
    s.execute("range.fill", json!({"range": "B1:B2", "input": "2,5", "locale": "de"})).unwrap();
    assert_eq!(v(&s, "B2"), Value::Number(2.5));
}

#[test]
fn switching_recalculates_text_and_keeps_a_saved_workbook_saved() {
    let mut s = s();
    s.execute("cell.set", json!({"cell": "A1", "input": "=TEXT(46387,\"mmmm\")"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("December"));
    {
        let d = s.doc_mut().unwrap();
        d.saved = d.wb.clone();
    }
    let undo = s.doc().unwrap().undo.len();
    s.execute("app.locale.set", json!({"locale": "de"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("Dezember"));
    s.execute("app.locale.set", json!({"locale": "fr"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("décembre"));
    let d = s.doc().unwrap();
    assert!(!d.is_dirty());
    assert_eq!(d.undo.len(), undo);
}

#[test]
fn validation_reads_input_in_the_locale() {
    let mut s = s();
    s.execute("data.validation", json!({"range": "A1", "type": "decimal", "operator": "lessThan", "formula1": "2"})).unwrap();
    let d = s.doc().unwrap();
    let at = CellRef::parse("A1").unwrap();
    assert!(crate::cmd::data::check_validation_in(&d.wb, 0, at, "1,5", Locale::De).is_none());
    assert!(crate::cmd::data::check_validation_in(&d.wb, 0, at, "2,5", Locale::De).is_some());
    // In en-US "1,5" is text, not a number below 2.
    assert!(crate::cmd::data::check_validation(&d.wb, 0, at, "1,5").is_some());
    assert!(crate::cmd::data::check_validation(&d.wb, 0, at, "1.5").is_none());
}

#[test]
fn pasted_text_follows_the_session_locale() {
    let mut s = s();
    s.execute("app.locale.set", json!({"locale": "de"})).unwrap();
    let text = "1.234,56\t31.12.2026\n-1.234,56\t12,5 %\n1.234\t2026-12-31\n";
    s.execute("edit.paste", json!({"at": "A1", "text": text})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1234.56));
    assert_eq!(v(&s, "B1"), Value::Number(46387.0));
    assert_eq!(v(&s, "A2"), Value::Number(-1234.56));
    assert_eq!(v(&s, "B2"), Value::Number(0.125));
    // A dotted group of three is a thousands group in German; ISO dates read in every locale.
    assert_eq!(v(&s, "A3"), Value::Number(1234.0));
    assert_eq!(v(&s, "B3"), Value::Number(46387.0));
    assert_eq!(shown(&s, "B1"), "31.12.2026");
    // An explicit locale wins over the session's.
    s.execute("edit.paste", json!({"at": "D1", "text": "1,234.5\t1,5", "locale": "en"})).unwrap();
    assert_eq!(v(&s, "D1"), Value::Number(1234.5));
    assert_eq!(v(&s, "E1"), Value::text("1,5"));
    assert!(s.execute("edit.paste", json!({"at": "D1", "text": "1", "locale": "xx"})).is_err());

    s.execute("app.locale.set", json!({"locale": "fr"})).unwrap();
    s.execute("edit.paste", json!({"at": "A5", "text": "1\u{202F}234,56\t31/12/2026\t12,5\u{a0}%\t-1\u{a0}234,5\t1.234"})).unwrap();
    assert_eq!(v(&s, "A5"), Value::Number(1234.56));
    assert_eq!(v(&s, "B5"), Value::Number(46387.0));
    assert_eq!(v(&s, "C5"), Value::Number(0.125));
    assert_eq!(v(&s, "D5"), Value::Number(-1234.5));
    assert_eq!(v(&s, "E5"), Value::text("1.234"));
}

#[test]
fn pasted_text_stays_english_without_a_locale() {
    let mut s = s();
    s.execute("edit.paste", json!({"at": "A1", "text": "1,234.5\t1.234,56\t3/4/2026\t2026-12-31"})).unwrap();
    assert_eq!(v(&s, "A1"), Value::Number(1234.5));
    assert_eq!(v(&s, "B1"), Value::text("1.234,56"));
    assert_eq!(v(&s, "C1"), Value::Number(46085.0));
    assert_eq!(v(&s, "D1"), Value::Number(46387.0));
}

#[test]
fn csv_import_follows_the_session_locale() {
    let mut s = s();
    s.execute("app.locale.set", json!({"locale": "de"})).unwrap();
    let csv = "Betrag;Datum;Anteil\n1.234,56;31.12.2026;12,5 %\n-1.234,56;03.04.2026;12,5%\n";
    s.execute("file.open", json!({"name": "de.csv", "base64": crate::io::base64_encode(csv.as_bytes())})).unwrap();
    assert_eq!(v(&s, "A1"), Value::text("Betrag"));
    assert_eq!(v(&s, "A2"), Value::Number(1234.56));
    assert_eq!(v(&s, "B2"), Value::Number(46387.0));
    assert_eq!(v(&s, "C2"), Value::Number(0.125));
    assert_eq!(v(&s, "A3"), Value::Number(-1234.56));
    assert_eq!(v(&s, "B3"), Value::Number(46115.0));
    assert_eq!(v(&s, "C3"), Value::Number(0.125));

    // An explicit locale reads in en-US: the comma splits fields.
    s.execute("file.open", json!({"name": "en.csv", "base64": crate::io::base64_encode(b"x\n1,5\n"), "locale": "en"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(1.0));
    assert_eq!(v(&s, "B2"), Value::Number(5.0));

    s.execute("app.locale.set", json!({"locale": "fr"})).unwrap();
    s.execute("data.fromTextCsv", json!({"text": "Montant;Date\n1 234,56;31/12/2026\n-1\u{202F}234,5;2026-12-31\n"})).unwrap();
    assert_eq!(v(&s, "A2"), Value::Number(1234.56));
    assert_eq!(v(&s, "B2"), Value::Number(46387.0));
    assert_eq!(v(&s, "A3"), Value::Number(-1234.5));
    assert_eq!(v(&s, "B3"), Value::Number(46387.0));
}
