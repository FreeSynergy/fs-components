//! Group `charts`: a bar chart and a line chart.
//!
//! Tests: picture comparison against `contract/umbau/bilder/charts`.

use fs_components_charts::{Chart, ChartView, Labels};
use fs_components_pictures::{Group, Tolerance};

const BARS: &str = r#"{"title":"Sales","data":{"values":[{"r":"N","v":28},{"r":"S","v":55},{"r":"E","v":-10},{"r":"W","v":43}]},
"mark":"bar","encoding":{"x":{"field":"r","type":"nominal"},"y":{"field":"v","type":"quantitative"}}}"#;
const LINE: &str = r#"{"data":{"values":[{"t":1,"v":3},{"t":2,"v":7},{"t":3,"v":4},{"t":4,"v":9}]},
"mark":"line","encoding":{"x":{"field":"t","type":"quantitative"},"y":{"field":"v","type":"quantitative"}}}"#;

fn group() -> Group {
    Group::new("charts", (320, 200), Tolerance::EXACT)
}

fn view(json: &str) -> ChartView {
    ChartView::new(Chart::from_json(json).expect("chart"), Labels::default())
}

#[test]
fn a_bar_chart() {
    let v = view(BARS);
    group().check("bars", "", |w, h| v.build(w, h));
}

#[test]
fn a_line_chart() {
    let v = view(LINE);
    group().check("line", "", |w, h| v.build(w, h));
}
