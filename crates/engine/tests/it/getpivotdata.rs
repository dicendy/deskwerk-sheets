//! GETPIVOTDATA over PivotTables built in the session.

use deskwerk_sheets_core::{CellError, CellRef, Value};
use deskwerk_sheets_engine::Session;
use serde_json::{Value as Json, json};

fn run(s: &mut Session, command: &str, params: Json) -> Json {
    s.execute(command, params).unwrap()
}

/// Sets `formula` in `cell` and returns the value it shows.
fn calc(s: &mut Session, cell: &str, formula: &str) -> Value {
    run(s, "cell.set", json!({"cell": cell, "input": formula}));
    s.doc().unwrap().wb.sheet(0).unwrap().value(CellRef::parse(cell).unwrap())
}

fn close(v: &Value, want: f64) -> bool {
    matches!(v, Value::Number(n) if (n - want).abs() < 1e-9)
}

/// Region × Product with Sum of Sales and Average of Units, at Sheet1!H3.
fn regions() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    let rows = json!([
        ["Region", "Product", "Units", "Sales"],
        ["East", "Apples", 10, 100],
        ["West", "Apples", 5, 50],
        ["East", "Pears", 8, 120],
        ["North", "Pears", 2, 30],
        ["West", "Plums", 7, 70],
        ["East", "Plums", 4, 60],
        ["North", "Apples", 9, 90],
        ["West", "Pears", 3, 45]
    ]);
    run(&mut s, "range.setValues", json!({"range": "A1", "values": rows}));
    run(
        &mut s,
        "insert.pivotTable",
        json!({
            "source": "Sheet1!A1:D9",
            "destination": "Sheet1!H3",
            "rows": ["Region"],
            "columns": ["Product"],
            "values": [{"field": "Sales"}, {"field": "Units", "func": "average"}]
        }),
    );
    s
}

#[test]
fn getpivotdata_totals_and_items() {
    let mut s = regions();
    let cases: &[(&str, f64)] = &[
        ("=GETPIVOTDATA(\"Sum of Sales\",$H$3)", 565.0),
        ("=GETPIVOTDATA(\"Sales\",H3)", 565.0),
        ("=GETPIVOTDATA(\"sum of sales\",H3,\"Region\",\"East\")", 280.0),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Region\",\"east\",\"Product\",\"Apples\")", 100.0),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Product\",\"Apples\")", 240.0),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Product\",\"Pears\",\"Region\",\"West\")", 45.0),
        ("=GETPIVOTDATA(\"Average of Units\",H3,\"Region\",\"North\")", 5.5),
        ("=GETPIVOTDATA(\"Units\",H3)", 6.0),
        // Names and items from cells.
        ("=GETPIVOTDATA(D1,H3,A1,A3,B1,B3)", 50.0),
    ];
    for (i, (formula, want)) in cases.iter().enumerate() {
        let v = calc(&mut s, &format!("A{}", 20 + i), formula);
        assert!(close(&v, *want), "{formula}: {v:?}");
    }
    // Any cell of the report names the PivotTable.
    let pt = s.doc().unwrap().wb.sheet(0).unwrap().pivots[0].clone();
    let inner = pt.last_range.unwrap().end.a1();
    let v = calc(&mut s, "B20", &format!("=GETPIVOTDATA(\"Sales\",{inner},\"Region\",\"West\")"));
    assert!(close(&v, 165.0), "{v:?}");
    // The formula follows the source.
    run(&mut s, "cell.set", json!({"cell": "D2", "input": "110"}));
    let v = s.doc().unwrap().wb.sheet(0).unwrap().value(CellRef::parse("B20").unwrap());
    assert!(close(&v, 165.0), "{v:?}");
    let v = s.doc().unwrap().wb.sheet(0).unwrap().value(CellRef::parse("A20").unwrap());
    assert!(close(&v, 575.0), "{v:?}");
}

#[test]
fn getpivotdata_errors() {
    let mut s = regions();
    let cases: &[(&str, CellError)] = &[
        // No PivotTable there.
        ("=GETPIVOTDATA(\"Sales\",A1)", CellError::Ref),
        ("=GETPIVOTDATA(\"Sales\",\"H3\")", CellError::Ref),
        // Unknown value field, field or item.
        ("=GETPIVOTDATA(\"Profit\",H3)", CellError::Ref),
        ("=GETPIVOTDATA(\"Region\",H3)", CellError::Ref),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Colour\",\"Red\")", CellError::Ref),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Region\",\"South\")", CellError::Ref),
        // A combination no record has.
        ("=GETPIVOTDATA(\"Sales\",H3,\"Region\",\"North\",\"Product\",\"Plums\")", CellError::Ref),
        // A field without its item.
        ("=GETPIVOTDATA(\"Sales\",H3,\"Region\")", CellError::Ref),
        // Errors in the arguments come through.
        ("=GETPIVOTDATA(1/0,H3)", CellError::Div0),
        ("=GETPIVOTDATA(\"Sales\",H3,\"Region\",NA())", CellError::NA),
    ];
    for (i, (formula, want)) in cases.iter().enumerate() {
        let v = calc(&mut s, &format!("A{}", 20 + i), formula);
        assert_eq!(v, Value::Error(*want), "{formula}");
    }
}

#[test]
fn getpivotdata_date_groups_filters_and_percentages() {
    let mut s = Session::new();
    s.new_workbook();
    let rows = json!([
        ["Region", "Date", "Sales"],
        ["East", "1/15/2026", 100],
        ["West", "2/10/2026", 50],
        ["East", "4/3/2026", 120],
        ["North", "4/20/2026", 30],
        ["West", "7/5/2026", 70],
        ["East", "10/30/2026", 60]
    ]);
    run(&mut s, "range.setValues", json!({"range": "A1", "values": rows}));
    run(
        &mut s,
        "insert.pivotTable",
        json!({
            "source": "Sheet1!A1:C7",
            "destination": "Sheet1!H3",
            "rows": [{"field": "Date", "group": "quarters"}],
            "values": ["Sales"],
            "filters": ["Region"]
        }),
    );
    run(&mut s, "pivot.filter", json!({"pivot": "PivotTable1", "field": "Region", "selected": ["East", "West"]}));
    assert!(close(&calc(&mut s, "A20", "=GETPIVOTDATA(\"Sales\",H3)"), 400.0));
    assert!(close(&calc(&mut s, "A21", "=GETPIVOTDATA(\"Sales\",H3,\"Date\",\"Qtr2\")"), 120.0));
    assert!(close(&calc(&mut s, "A22", "=GETPIVOTDATA(\"Sales\",H3,\"Quarters\",\"qtr1\")"), 150.0));
    // Region is a report filter, not a row or column field.
    assert_eq!(calc(&mut s, "A23", "=GETPIVOTDATA(\"Sales\",H3,\"Region\",\"East\")"), Value::Error(CellError::Ref));
    run(&mut s, "pivot.valueSettings", json!({"pivot": "PivotTable1", "field": "Sum of Sales", "showAs": "percentOfGrandTotal"}));
    assert!(close(&calc(&mut s, "A24", "=GETPIVOTDATA(\"Sales\",H3,\"Date\",\"Qtr1\")"), 0.375));
}
