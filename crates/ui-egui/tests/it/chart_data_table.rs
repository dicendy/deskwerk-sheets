use deskwerk_sheets_engine::Session;
use deskwerk_sheets_engine::model::{ObjKind, ObjRef};
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use serde_json::json;

#[test]
fn format_chart_pane_controls_the_engine_data_table() {
    let mut session = Session::new();
    session.new_workbook();
    session.execute("range.setValues", json!({"range": "A1", "values": [["Quarter", "North", "South"], ["Q1", 10, 30], ["Q2", 30, 10]]})).unwrap();
    let id = session.execute("insert.chart", json!({"range": "A1:C3", "type": "column", "at": "E3"})).unwrap()["chart"].as_u64().unwrap() as u32;
    let mut h = crate::support::harness(session, 4);
    h.state_mut().selected_object = Some(ObjRef::new(ObjKind::Chart, id));
    h.state_mut().grid.pane = Some("formatChart".into());
    h.run_steps(2);
    h.get_by_role_and_label(Role::DisclosureTriangle, "Data Table").click();
    h.run_steps(2);
    let table = |h: &crate::common::Harness| h.state().session.active().unwrap().wb.active().unwrap().charts[0].data_table.clone();
    assert_eq!(table(&h), None);
    h.get_by_label("Show Data Table").click();
    h.run_steps(2);
    assert!(table(&h).unwrap().show_keys);
    h.get_by_label("Show Legend Keys").click();
    h.run_steps(2);
    assert!(!table(&h).unwrap().show_keys);
    h.get_by_label("Horizontal Borders").click();
    h.run_steps(2);
    assert!(!table(&h).unwrap().horizontal_border);
    h.get_by_label("Show Data Table").click();
    h.run_steps(2);
    assert_eq!(table(&h), None);
}

fn chart_pane(kind: &str) -> crate::common::Harness {
    let mut session = Session::new();
    session.new_workbook();
    session.execute("range.setValues", json!({"range": "A1", "values": [["Quarter", "North"], ["Q1", 10], ["Q2", 1000]]})).unwrap();
    let id = session.execute("insert.chart", json!({"range": "A1:B3", "type": kind, "at": "E3"})).unwrap()["chart"].as_u64().unwrap() as u32;
    let mut h = crate::support::harness(session, 4);
    h.state_mut().selected_object = Some(ObjRef::new(ObjKind::Chart, id));
    h.state_mut().grid.pane = Some("formatChart".into());
    h.run_steps(2);
    h
}

fn first_chart(h: &crate::common::Harness) -> deskwerk_sheets_engine::model::Chart {
    h.state().session.active().unwrap().wb.active().unwrap().charts[0].clone()
}

#[test]
fn format_chart_pane_sets_a_logarithmic_value_axis() {
    let mut h = chart_pane("column");
    h.get_by_role_and_label(Role::DisclosureTriangle, "Value Axis").click();
    h.run_steps(2);
    h.get_by_label("Logarithmic scale").click();
    h.run_steps(2);
    assert_eq!(first_chart(&h).value_axis.log_base, Some(10.0));
    h.get_by_label("Logarithmic scale").click();
    h.run_steps(2);
    assert_eq!(first_chart(&h).value_axis.log_base, None);
}

#[test]
fn format_chart_pane_shows_pie_percentages() {
    let mut h = chart_pane("pie");
    h.get_by_role_and_label(Role::DisclosureTriangle, "Elements").click();
    h.run_steps(2);
    h.get_by_label("Show percentage").click();
    h.run_steps(2);
    assert!(first_chart(&h).data_label_percent);
    h.get_by_label("Show percentage").click();
    h.run_steps(2);
    assert!(!first_chart(&h).data_label_percent);
}
