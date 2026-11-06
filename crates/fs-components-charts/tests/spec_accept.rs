//! Specs inside the subset are read, with the values Vega-Lite gives them.

use fs_components_charts::spec::{Aggregate, FieldType, MarkKind, Sort, StackMode, Title, parse};

const BAR: &str = r#"{
  "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
  "title": "Sales",
  "description": "Sales per region",
  "width": 320, "height": 200,
  "data": {"values": [{"r": "N", "v": 3}, {"r": "S", "v": 5}]},
  "mark": "bar",
  "encoding": {
    "x": {"field": "r", "type": "nominal", "sort": "-y", "axis": {"title": "Region", "labels": false}},
    "y": {"field": "v", "type": "quantitative", "aggregate": "sum", "scale": {"domain": [0, 10], "nice": false}}
  }
}"#;

#[test]
fn a_bar_chart_is_read_completely() {
    let s = parse(BAR).unwrap();
    assert_eq!(s.title.as_deref(), Some("Sales"));
    assert_eq!(s.description.as_deref(), Some("Sales per region"));
    assert_eq!((s.width, s.height), (Some(320.0), Some(200.0)));
    assert_eq!(s.mark.kind, MarkKind::Bar);
    assert_eq!(s.data.len(), 2);
    let x = s.encoding.x.unwrap();
    assert_eq!(x.ftype, FieldType::Nominal);
    assert!(matches!(
        x.sort,
        Sort::ByChannel {
            descending: true,
            ..
        }
    ));
    assert_eq!(x.axis.title, Title::Text("Region".into()));
    assert!(!x.axis.labels);
    let y = s.encoding.y.unwrap();
    assert_eq!(y.aggregate, Some(Aggregate::Sum));
    assert_eq!(y.scale.domain, Some([0.0, 10.0]));
    assert_eq!(y.scale.nice, Some(false));
    assert_eq!(y.stack, StackMode::Zero);
}

#[test]
fn defaults_are_those_of_vega_lite() {
    let s = parse(r#"{"data":{"values":[]},"mark":{"type":"arc","innerRadius":40},
        "encoding":{"theta":{"field":"v","type":"quantitative"},"color":{"field":"k","type":"nominal","legend":null}}}"#).unwrap();
    assert_eq!(s.mark.inner_radius, 40.0);
    assert!(!s.encoding.color.unwrap().legend);
    let s = parse(
        r#"{"data":{"values":[]},"mark":"line","encoding":{
        "x":{"field":"d","type":"temporal"},"y":{"aggregate":"count"}}}"#,
    )
    .unwrap();
    // `count` needs no field and no type: it is quantitative
    assert_eq!(s.encoding.y.unwrap().ftype, FieldType::Quantitative);
    assert_eq!(s.encoding.x.unwrap().sort, Sort::Ascending);
}

#[test]
fn average_is_a_name_of_mean_and_title_object_is_read() {
    let s = parse(r#"{"title":{"text":"T"},"data":{"values":[]},"mark":"point","usermeta":{"a":1},"encoding":{
        "x":{"field":"a","type":"quantitative"},"y":{"field":"b","type":"quantitative","aggregate":"average"}}}"#).unwrap();
    assert_eq!(s.title.as_deref(), Some("T"));
    assert_eq!(s.encoding.y.unwrap().aggregate, Some(Aggregate::Mean));
}

#[test]
fn nested_field_paths_follow_vega_lite() {
    use fs_components_charts::spec::lookup;
    let row: serde_json::Value = serde_json::json!({"a": {"b": 1}, "c.d": 2});
    let row = row.as_object().unwrap();
    assert_eq!(lookup(row, "a.b").unwrap(), 1);
    assert_eq!(lookup(row, r"c\.d").unwrap(), 2);
    assert!(lookup(row, "c.d").is_none());
}
