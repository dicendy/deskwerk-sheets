//! Importing a combination chart keeps each series' own group kind, the first group's included,
//! and which series plot on the secondary value axis.

use deskwerk_sheets_model::{ChartAxis, ChartKind, Workbook};

use crate::chart::{read_chart, write_chart};
use crate::xml::parse;

fn ser(i: usize) -> String {
    format!("<c:ser><c:idx val=\"{i}\"/><c:order val=\"{i}\"/><c:val><c:numRef><c:f>Data!$A$1:$A$5</c:f></c:numRef></c:val></c:ser>")
}

const AX: &str = "<c:axId val=\"1\"/><c:axId val=\"2\"/>";

fn bar(sers: &str) -> String {
    format!("<c:barChart><c:barDir val=\"col\"/><c:grouping val=\"clustered\"/>{sers}{AX}</c:barChart>")
}

fn line(sers: &str) -> String {
    format!("<c:lineChart><c:grouping val=\"standard\"/>{sers}{AX}</c:lineChart>")
}

fn kinds(first: &str, second: &str) -> Vec<Option<ChartKind>> {
    let xml = format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
         <c:chart><c:plotArea>{first}{second}</c:plotArea></c:chart></c:chartSpace>"
    );
    let root = parse(xml.as_bytes()).unwrap();
    let ch = read_chart(&root).unwrap();
    assert_eq!(ch.kind, ChartKind::Combo);
    ch.series.iter().map(|s| s.kind).collect()
}

#[test]
fn first_group_with_two_columns_stays_columns() {
    let k = kinds(&bar(&(ser(0) + &ser(1))), &line(&ser(2)));
    assert_eq!(k[0], Some(ChartKind::ColumnClustered));
    assert_eq!(k[1], Some(ChartKind::ColumnClustered));
    assert!(matches!(k[2], Some(ChartKind::Line | ChartKind::LineMarkers)));
}

#[test]
fn line_first_group_stays_a_line() {
    let k = kinds(&line(&ser(0)), &bar(&ser(1)));
    assert!(matches!(k[0], Some(ChartKind::Line | ChartKind::LineMarkers)));
    assert_eq!(k[1], Some(ChartKind::ColumnClustered));
}

/// Columns on axes 1/2 and a line on its own axes 3/4, the secondary value axis listed first.
fn secondary_combo() -> deskwerk_sheets_model::Chart {
    let xml = format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart><c:plotArea>{}\
         <c:lineChart><c:grouping val=\"standard\"/>{}<c:axId val=\"3\"/><c:axId val=\"4\"/></c:lineChart>\
         <c:catAx><c:axId val=\"3\"/><c:delete val=\"1\"/></c:catAx>\
         <c:valAx><c:axId val=\"4\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"9\"/></c:scaling><c:axPos val=\"r\"/></c:valAx>\
         <c:catAx><c:axId val=\"1\"/></c:catAx>\
         <c:valAx><c:axId val=\"2\"/><c:scaling><c:logBase val=\"10\"/><c:orientation val=\"minMax\"/></c:scaling><c:axPos val=\"l\"/></c:valAx>\
         </c:plotArea></c:chart></c:chartSpace>",
        bar(&ser(0)),
        ser(1)
    );
    read_chart(&parse(xml.as_bytes()).unwrap()).unwrap()
}

#[test]
fn the_value_axis_options_are_the_primary_axis_wherever_it_is_listed() {
    let ch = secondary_combo();
    assert_eq!(ch.kind, ChartKind::Combo);
    assert!(!ch.series[0].secondary && ch.series[1].secondary);
    assert_eq!(ch.value_axis, ChartAxis { log_base: Some(10.0), ..ChartAxis::default() });
}

#[test]
fn a_secondary_line_or_area_keeps_its_kind_when_saved() {
    let wb = Workbook::new();
    for kind in [ChartKind::LineMarkers, ChartKind::Area, ChartKind::AreaStacked, ChartKind::ColumnStacked] {
        let mut ch = secondary_combo();
        ch.series[1].kind = Some(kind);
        let back = read_chart(&parse(write_chart(&wb, 0, &ch, "Book.xlsx").as_bytes()).unwrap()).unwrap();
        assert_eq!(back.kind, ChartKind::Combo, "{kind:?}");
        assert_eq!(back.series[0].kind, Some(ChartKind::ColumnClustered), "{kind:?}");
        assert_eq!(back.series[1].kind, Some(kind), "{kind:?}");
        assert!(!back.series[0].secondary && back.series[1].secondary, "{kind:?}");
        assert_eq!(back.value_axis, ch.value_axis, "{kind:?}");
    }
}
