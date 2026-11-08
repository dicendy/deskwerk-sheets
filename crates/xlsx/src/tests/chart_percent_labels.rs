//! Synthetic DrawingML only: pie and doughnut slices labelled with their percentage
//! (`c:dLbls/c:showPercent`), read and saved.

use gridcraft_model::{Chart, ChartKind, Workbook};

use crate::chart::{read_chart, write_chart};
use crate::xml::parse;

fn chart(group: &str, dlbls: &str) -> Chart {
    let xml = format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart><c:plotArea>\
         <c:{group}><c:varyColors val=\"1\"/><c:ser><c:idx val=\"0\"/><c:order val=\"0\"/>{dlbls}\
         <c:val><c:numLit><c:formatCode>General</c:formatCode><c:ptCount val=\"2\"/><c:pt idx=\"0\"><c:v>3</c:v></c:pt><c:pt idx=\"1\"><c:v>1</c:v></c:pt></c:numLit></c:val>\
         </c:ser></c:{group}></c:plotArea></c:chart></c:chartSpace>"
    );
    read_chart(&parse(xml.as_bytes()).unwrap()).unwrap()
}

fn dlbls(val: u8, percent: u8) -> String {
    format!(
        "<c:dLbls><c:showLegendKey val=\"0\"/><c:showVal val=\"{val}\"/><c:showCatName val=\"0\"/><c:showSerName val=\"0\"/><c:showPercent val=\"{percent}\"/></c:dLbls>"
    )
}

#[test]
fn slice_percentages_are_read() {
    for group in ["pieChart", "doughnutChart", "pie3DChart"] {
        let c = chart(group, &dlbls(0, 1));
        assert!(c.data_label_percent && !c.data_labels, "{group}");
        let c = chart(group, &dlbls(1, 1));
        assert!(c.data_label_percent && c.data_labels, "{group}");
        let c = chart(group, &dlbls(1, 0));
        assert!(!c.data_label_percent && c.data_labels, "{group}");
        assert!(!chart(group, "").data_label_percent, "{group}");
    }
    // Only slices have a share of a total: a column's showPercent means nothing.
    let xml = format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart><c:plotArea>\
         <c:barChart><c:barDir val=\"col\"/><c:grouping val=\"clustered\"/>{}<c:ser><c:idx val=\"0\"/><c:order val=\"0\"/>\
         <c:val><c:numLit><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>3</c:v></c:pt></c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>",
        dlbls(0, 1)
    );
    assert!(!read_chart(&parse(xml.as_bytes()).unwrap()).unwrap().data_label_percent);
}

#[test]
fn slice_percentages_are_saved() {
    let wb = Workbook::new();
    for group in ["pieChart", "doughnutChart"] {
        for (val, percent) in [(0, 1), (1, 1), (1, 0), (0, 0)] {
            let c = chart(group, &dlbls(val, percent));
            let xml = write_chart(&wb, 0, &c, "Book.xlsx");
            assert!(xml.contains(&format!("<c:showVal val=\"{val}\"/>")), "{xml}");
            assert!(xml.contains(&format!("<c:showPercent val=\"{percent}\"/>")), "{xml}");
            let back = read_chart(&parse(xml.as_bytes()).unwrap()).unwrap();
            assert_eq!((back.data_labels, back.data_label_percent), (val == 1, percent == 1), "{group} {val} {percent}");
        }
    }
    // A pie turned into columns drops the percentages.
    let mut c = chart("pieChart", &dlbls(0, 1));
    c.kind = ChartKind::ColumnClustered;
    let xml = write_chart(&wb, 0, &c, "Book.xlsx");
    assert!(!xml.contains("<c:showPercent val=\"1\"/>"), "{xml}");
    assert!(!read_chart(&parse(xml.as_bytes()).unwrap()).unwrap().data_label_percent);
}
