//! Real data problems at the layout level: exact numbers, nothing panics, nothing is silent.

mod layout_util;

use fs_components_charts::scene::{Item, Paint};
use fs_components_charts::{Chart, Labels};
use layout_util::{CAT, TIME, doc, finite, rects, scene, texts};

#[test]
fn empty_data_says_so_in_the_callers_words() {
    let s = scene(&doc("bar", "[]", CAT), 300.0, 200.0);
    assert_eq!(texts(&s), vec!["No data".to_string()]);
    assert!(rects(&s).is_empty() && finite(&s));
    let c = Chart::from_json(&doc("bar", "[]", CAT)).unwrap();
    let alt = c.text_alternative(&Labels::default(), 10);
    assert!(alt.table.rows.is_empty() && alt.table.headers == ["k", "v"]);
    assert!(
        alt.summary.contains("Bar chart with 0 rows."),
        "{}",
        alt.summary
    );
}

#[test]
fn one_point_every_mark_shows_it() {
    let one = r#"[{"k":"a","v":4,"d":"2024-03-10","g":"x"}]"#;
    let bar = scene(&doc("bar", one, CAT), 300.0, 200.0);
    let r = rects(&bar);
    assert_eq!(r.len(), 1);
    // a band scale with one category: 90 % of the plot width is the bar, centred
    assert!(
        (r[0][2] - 0.9 * bar.plot[2]).abs() < 0.01,
        "{r:?} {:?}",
        bar.plot
    );
    let line = scene(&doc("line", one, TIME), 300.0, 200.0);
    assert!(
        line.items
            .iter()
            .any(|i| matches!(i, Item::Marker { filled: true, .. })),
        "a line of one point is a dot"
    );
    assert!(!line.items.iter().any(|i| matches!(i, Item::Path { .. })));
    let pt = scene(
        &doc("point", one, CAT.replace("nominal", "ordinal").as_str()),
        300.0,
        200.0,
    );
    assert!(pt.items.iter().any(|i| matches!(i, Item::Marker { .. })) && finite(&pt));
    let area = scene(&doc("area", one, TIME), 300.0, 200.0);
    assert!(
        area.items.iter().any(|i| matches!(
            i,
            Item::Line {
                paint: Paint::Series(0),
                ..
            }
        )),
        "an area of one point is a rule"
    );
    let arc = scene(
        &doc("arc", one, r#""theta":{"field":"v","type":"quantitative"}"#),
        300.0,
        200.0,
    );
    let secs = arc
        .items
        .iter()
        .filter(|i| matches!(i, Item::Sector { .. }))
        .count();
    assert_eq!(secs, 1, "a single value is the whole circle");
}

#[test]
fn all_values_equal_still_have_a_scale() {
    for v in ["5", "0", "-3", "1e308", "1e-300"] {
        let rows = format!(r#"[{{"k":"a","v":{v}}},{{"k":"b","v":{v}}},{{"k":"c","v":{v}}}]"#);
        let s = scene(&doc("bar", &rows, CAT), 300.0, 200.0);
        assert!(finite(&s), "value {v}");
        let r = rects(&s);
        if v == "0" {
            assert!(r.is_empty(), "zero-height bars have no area");
        } else {
            assert_eq!(r.len(), 3, "value {v}");
            assert!(
                r.iter().all(|b| (b[3] - r[0][3]).abs() < 1e-3),
                "equal values, equal bars"
            );
        }
    }
}

#[test]
fn nulls_and_text_break_a_line_and_are_reported_under_the_chart() {
    let rows = r#"[{"d":"2024-03-01","v":1},{"d":"2024-03-02","v":2},{"d":"2024-03-03","v":null},
        {"d":"2024-03-04","v":"NaN"},{"d":"2024-03-05","v":5},{"d":"2024-03-06","v":6}]"#;
    let s = scene(&doc("line", rows, TIME), 400.0, 300.0);
    let paths: Vec<_> = s
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Path {
                points,
                paint: Paint::Series(_),
                ..
            } = i
            {
                Some(points.len())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        paths,
        vec![2, 2],
        "two runs of two points; the gap is not bridged"
    );
    let t = texts(&s);
    assert!(t.iter().any(|x| x == "Rows that break a line: 2."), "{t:?}");
    assert!(t.iter().any(|x| x == "v: missing (1)"), "{t:?}");
    assert!(
        t.iter()
            .any(|x| x.starts_with("v: not a number (1)") && x.contains("\"NaN\"")),
        "{t:?}"
    );
}

#[test]
fn bars_with_bad_values_leave_them_out_and_say_how_many() {
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":null},{"k":"c","v":"abc"},{"k":"d","v":4}]"#;
    let s = scene(&doc("bar", rows, CAT), 400.0, 300.0);
    assert_eq!(rects(&s).len(), 2);
    let t = texts(&s);
    assert!(t.iter().any(|x| x == "Rows left out: 2 of 4."), "{t:?}");
    // categories of the dropped rows are not on the axis either: only a and d are labelled
    assert!(
        t.iter().any(|x| x == "a") && t.iter().any(|x| x == "d") && !t.iter().any(|x| x == "b"),
        "{t:?}"
    );
}
