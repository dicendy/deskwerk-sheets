//! `sheet.read` caps the text of one answer. Cells share their text, the answer copies it: a file
//! whose thousands of cells all point at one 1 MB shared string would otherwise need gigabytes.

use std::sync::Arc;

use gridcraft_core::{CellRef, Value};
use gridcraft_engine::Session;
use serde_json::json;

/// `rows` cells in column A that all hold the same `bytes`-long text.
fn shared_text(rows: u32, bytes: usize) -> Session {
    let mut s = Session::new();
    s.new_workbook();
    let text: Arc<str> = Arc::from("x".repeat(bytes));
    let wb = Arc::make_mut(&mut s.doc_mut().unwrap().wb);
    let sh = wb.sheet_mut(0).unwrap();
    for row in 0..rows {
        sh.set_value(CellRef::new(row, 0), Value::Text(text.clone()));
    }
    s
}

#[test]
fn a_read_over_the_text_limit_is_refused() {
    let mut s = shared_text(100, 1 << 20);
    for params in [json!({}), json!({"formulas": true}), json!({"formatted": true}), json!({"range": "A1:A100"})] {
        let err = s.execute("sheet.read", params.clone()).unwrap_err().to_string();
        assert!(err.contains("MiB of text") && err.contains("read it in parts"), "{params}: {err}");
    }
    // A part of it reads.
    let part = s.execute("sheet.read", json!({"range": "A1:A10"})).unwrap();
    assert_eq!(part["values"].as_array().unwrap().len(), 10);
    assert_eq!(part["values"][9][0].as_str().unwrap().len(), 1 << 20);
}

#[test]
fn ordinary_reads_are_unaffected() {
    let mut s = shared_text(2000, 100);
    let all = s.execute("sheet.read", json!({})).unwrap();
    assert_eq!(all["range"], "A1:A2000");
    assert_eq!(all["values"].as_array().unwrap().len(), 2000);
}
