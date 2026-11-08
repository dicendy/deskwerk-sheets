//! Synthetic DrawingML only: value-axis bounds, major unit and number format, read and saved.

use std::io::Read;

use crate::{chart::read_chart, read_xlsx, write_xlsx, xml::parse};
use gridcraft_model::{ChartAxis, Workbook};

fn chart(val_ax: &str) -> gridcraft_model::Chart {
    let xml = format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart><c:plotArea>\
        <c:barChart><c:barDir val=\"col\"/><c:grouping val=\"clustered\"/><c:ser><c:idx val=\"0\"/><c:order val=\"0\"/>\
        <c:val><c:numLit><c:formatCode>General</c:formatCode><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>12</c:v></c:pt></c:numLit></c:val>\
        </c:ser><c:axId val=\"1\"/><c:axId val=\"2\"/></c:barChart><c:catAx><c:axId val=\"1\"/></c:catAx>{val_ax}</c:plotArea></c:chart></c:chartSpace>"
    );
    read_chart(&parse(xml.as_bytes()).unwrap()).unwrap()
}

#[test]
fn value_axis_options_are_read() {
    assert_eq!(chart("<c:valAx><c:axId val=\"2\"/></c:valAx>").value_axis, ChartAxis::default());
    let c = chart(
        "<c:valAx><c:axId val=\"2\"/><c:scaling><c:orientation val=\"minMax\"/><c:max val=\"100\"/><c:min val=\"-20\"/></c:scaling>\
        <c:numFmt formatCode=\"0.0&quot;%&quot;\" sourceLinked=\"0\"/><c:majorUnit val=\"20\"/></c:valAx>",
    );
    assert_eq!(c.value_axis, ChartAxis { min: Some(-20.0), max: Some(100.0), major_unit: Some(20.0), number_format: Some("0.0\"%\"".into()) });
    // A format linked to the source cells, "General" and a unit that is not positive are automatic.
    let c = chart("<c:valAx><c:axId val=\"2\"/><c:numFmt formatCode=\"0.00\" sourceLinked=\"1\"/><c:majorUnit val=\"0\"/></c:valAx>");
    assert_eq!(c.value_axis, ChartAxis::default());
    let c = chart("<c:valAx><c:axId val=\"2\"/><c:numFmt formatCode=\"General\" sourceLinked=\"0\"/></c:valAx>");
    assert_eq!(c.value_axis, ChartAxis::default());
}

#[test]
fn value_axis_options_survive_xlsx_save_reopen() {
    let opts = ChartAxis { min: Some(0.0), max: Some(250.0), major_unit: Some(50.0), number_format: Some("#,##0 \"kg\"".into()) };
    let mut c = chart("<c:valAx><c:axId val=\"2\"/></c:valAx>");
    c.id = 1;
    c.value_axis = opts.clone();
    let mut wb = Workbook::new();
    wb.sheet_mut(0).unwrap().charts.push(c);
    let bytes = write_xlsx(&wb).unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap().by_name("xl/charts/chart1.xml").unwrap().read_to_string(&mut xml).unwrap();
    assert!(xml.contains("<c:max val=\"250\"/><c:min val=\"0\"/></c:scaling>"), "{xml}");
    assert!(xml.contains("<c:numFmt formatCode=\"#,##0 &quot;kg&quot;\" sourceLinked=\"0\"/>"), "{xml}");
    assert!(xml.contains("<c:crossBetween val=\"between\"/><c:majorUnit val=\"50\"/></c:valAx>"), "{xml}");
    let (back, _) = read_xlsx(&bytes).unwrap();
    assert_eq!(back.sheets[0].charts[0].value_axis, opts);
}

#[test]
fn reversed_bounds_are_not_written() {
    let mut c = chart("<c:valAx><c:axId val=\"2\"/></c:valAx>");
    c.id = 1;
    c.value_axis = ChartAxis { min: Some(10.0), max: Some(5.0), ..ChartAxis::default() };
    let mut wb = Workbook::new();
    wb.sheet_mut(0).unwrap().charts.push(c);
    let bytes = write_xlsx(&wb).unwrap();
    let (back, _) = read_xlsx(&bytes).unwrap();
    assert_eq!(back.sheets[0].charts[0].value_axis, ChartAxis::default());
}
