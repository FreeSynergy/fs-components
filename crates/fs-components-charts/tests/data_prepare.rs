//! Real data problems: nothing panics, nothing disappears without a count.

use fs_components_charts::data::{Cell, Invalid, IssueKind, prepare};
use fs_components_charts::spec::parse;

fn spec(mark: &str, rows: &str, enc: &str) -> fs_components_charts::spec::Spec {
    parse(&format!(
        r#"{{"data":{{"values":{rows}}},"mark":"{mark}","encoding":{{{enc}}}}}"#
    ))
    .unwrap()
}

const BAR: &str = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative"}"#;

#[test]
fn empty_data_gives_nothing_to_draw_and_no_issue() {
    let p = prepare(&spec("bar", "[]", BAR)).unwrap();
    assert_eq!(p.data, [] as [fs_components_charts::data::Datum; 0]);
    assert_eq!((p.report.rows, p.report.kept, p.report.dropped), (0, 0, 0));
    assert!(p.report.is_clean());
}

#[test]
fn null_nan_text_and_wrong_kinds_are_dropped_and_counted_by_reason() {
    let rows = r#"[{"k":"a","v":1},{"k":"b","v":null},{"k":"c"},{"k":"d","v":"NaN"},
        {"k":"e","v":"Infinity"},{"k":"f","v":""},{"k":"g","v":[1]},{"k":"h","v":true},{"k":"i","v":"12.5"},{"k":"j","v":" 7 "}]"#;
    let p = prepare(&spec("bar", rows, BAR)).unwrap();
    // Numbers are read as JavaScript reads them, which is what Vega-Lite does (tests/golden
    // g20_number_reading): "" is 0, [1] is 1, true is 1, "12.5" and " 7 " are numbers;
    // null, a missing field, "NaN" and "Infinity" are not
    let drawn: Vec<_> = p.data.iter().map(|d| d.cells[1].num().unwrap()).collect();
    assert_eq!(drawn, vec![1.0, 0.0, 1.0, 1.0, 12.5, 7.0]);
    assert_eq!((p.report.rows, p.report.kept, p.report.dropped), (10, 6, 4));
    let count = |k: IssueKind| {
        p.report
            .issues
            .iter()
            .find(|i| i.kind == k)
            .map_or(0, |i| i.count)
    };
    assert_eq!(count(IssueKind::Null), 2);
    assert_eq!(count(IssueKind::Invalid(Invalid::NotANumber)), 2);
    assert_eq!(count(IssueKind::Invalid(Invalid::WrongKind)), 0);
    let nan = p
        .report
        .issues
        .iter()
        .find(|i| i.kind == IssueKind::Invalid(Invalid::NotANumber))
        .unwrap();
    assert_eq!(nan.field, "v");
    assert_eq!(nan.example.as_deref(), Some("\"NaN\""));
}

#[test]
fn a_line_breaks_at_an_invalid_value_instead_of_dropping_the_row() {
    let rows = r#"[{"k":1,"v":1},{"k":2,"v":null},{"k":3,"v":3}]"#;
    let enc = r#""x":{"field":"k","type":"quantitative"},"y":{"field":"v","type":"quantitative"}"#;
    let p = prepare(&spec("line", rows, enc)).unwrap();
    assert_eq!((p.data.len(), p.report.broken, p.report.dropped), (3, 1, 0));
    assert!(matches!(p.data[1].cells[1], Cell::Null));
}

#[test]
fn a_null_category_is_a_category_not_a_dropped_row() {
    let rows = r#"[{"k":null,"v":1},{"v":2},{"k":"a","v":3}]"#;
    let p = prepare(&spec("bar", rows, BAR)).unwrap();
    assert_eq!((p.report.kept, p.report.dropped), (3, 0));
    assert!(p.report.is_clean());
}

#[test]
fn aggregates_group_by_the_other_channels_in_order_of_appearance() {
    let rows =
        r#"[{"k":"b","v":1},{"k":"a","v":2},{"k":"b","v":4},{"k":"a","v":null},{"k":"b","v":"x"}]"#;
    let enc = |agg: &str| {
        format!(
            r#""x":{{"field":"k","type":"nominal"}},"y":{{"field":"v","type":"quantitative","aggregate":"{agg}"}}"#
        )
    };
    let get = |agg: &str| -> Vec<(String, f64)> {
        let p = prepare(&spec("bar", rows, &enc(agg))).unwrap();
        p.data
            .iter()
            .map(|d| {
                (
                    match &d.cells[0] {
                        Cell::Text(t) => t.clone(),
                        _ => String::new(),
                    },
                    d.cells[1].num().unwrap(),
                )
            })
            .collect()
    };
    assert_eq!(get("sum"), vec![("b".into(), 5.0), ("a".into(), 2.0)]);
    assert_eq!(get("mean"), vec![("b".into(), 2.5), ("a".into(), 2.0)]);
    assert_eq!(get("min"), vec![("b".into(), 1.0), ("a".into(), 2.0)]);
    assert_eq!(get("max"), vec![("b".into(), 4.0), ("a".into(), 2.0)]);
    let c = prepare(&spec(
        "bar",
        rows,
        r#""x":{"field":"k","type":"nominal"},"y":{"aggregate":"count"}"#,
    ))
    .unwrap();
    assert_eq!(
        c.data
            .iter()
            .map(|d| d.cells[1].num().unwrap())
            .collect::<Vec<_>>(),
        vec![3.0, 2.0]
    );
}

#[test]
fn huge_values_do_not_overflow_silently() {
    let rows = r#"[{"k":"a","v":1.7e308},{"k":"a","v":1.7e308},{"k":"b","v":1.7e308}]"#;
    let sum = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative","aggregate":"sum"}"#;
    let p = prepare(&spec("bar", rows, sum)).unwrap();
    // the sum of group a does not fit a number: dropped and reported, group b stays
    assert_eq!((p.report.kept, p.report.dropped), (1, 1));
    assert!(
        p.report
            .issues
            .iter()
            .any(|i| i.kind == IssueKind::Invalid(Invalid::Overflow) && i.field == "v")
    );
    let mean = sum.replace("sum", "mean");
    let p = prepare(&spec("bar", rows, &mean)).unwrap();
    assert_eq!(
        p.data[0].cells[1].num(),
        Some(1.7e308),
        "the mean of huge values must not overflow"
    );
}

#[test]
fn negative_angles_are_not_drawn_and_are_reported() {
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":-1},{"k":"c","v":0}]"#;
    let enc =
        r#""theta":{"field":"v","type":"quantitative"},"color":{"field":"k","type":"nominal"}"#;
    let p = prepare(&spec("arc", rows, enc)).unwrap();
    assert_eq!((p.report.kept, p.report.dropped), (2, 1));
    assert!(
        p.report
            .issues
            .iter()
            .any(|i| i.kind == IssueKind::Invalid(Invalid::Negative))
    );
}

#[test]
fn dates_with_and_without_an_offset_are_not_mixed() {
    let rows = r#"[{"d":"2024-03-10","v":1},{"d":"2024-03-11T00:00Z","v":2}]"#;
    let enc = r#""x":{"field":"d","type":"temporal"},"y":{"field":"v","type":"quantitative"}"#;
    let e = prepare(&spec("line", rows, enc)).unwrap_err();
    assert!(
        e.message
            .contains("mixes times with and without a UTC offset"),
        "{e}"
    );
    assert_eq!(e.path, "/encoding/x/field");
}

#[test]
fn a_hundred_thousand_rows_are_prepared_fast() {
    let rows: Vec<String> = (0..100_000)
        .map(|i| format!(r#"{{"k":"c{}","v":{}}}"#, i % 50, i % 97))
        .collect();
    let s = spec(
        "bar",
        &format!("[{}]", rows.join(",")),
        BAR.replace(
            "\"v\",\"type\":\"quantitative\"",
            "\"v\",\"type\":\"quantitative\",\"aggregate\":\"sum\"",
        )
        .as_str(),
    );
    let t = std::time::Instant::now();
    let p = prepare(&s).unwrap();
    assert_eq!((p.report.rows, p.report.kept), (100_000, 50));
    eprintln!("100000 rows prepared in {:?}", t.elapsed()); // no wall-clock assertion: machines differ
}
