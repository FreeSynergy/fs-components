//! The text alternative, the colour limit, and layouts with exact numbers.

mod layout_util;

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::Item;
use fs_components_charts::{Chart, Labels};
use layout_util::{doc, rects, scene, texts};

#[test]
fn the_text_alternative_has_a_summary_and_exactly_the_drawn_data() {
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":null},{"k":"c","v":7.5}]"#;
    let c = Chart::from_json(&doc("bar", rows, layout_util::CAT)).unwrap();
    let alt = c.text_alternative(&Labels::default(), 100);
    assert_eq!(alt.table.headers, ["k", "v"]);
    assert_eq!(
        alt.table.rows,
        [["a", "3"], ["c", "7.5"]],
        "only what is drawn"
    );
    assert_eq!(
        alt.summary,
        "Bar chart with 2 rows. v from 3 to 7.5. Rows left out: 1 of 3. v: missing (1)"
    );
    let capped = c.text_alternative(&Labels::default(), 1);
    assert_eq!((capped.table.rows.len(), capped.table.omitted), (1, 1));
    assert!(alt.to_text().starts_with("Bar chart") && alt.to_text().contains("c\t7.5"));
}

#[test]
fn the_spec_description_is_the_summary_and_words_come_from_the_caller() {
    let json = r#"{"title":"T","description":"Sales grew","data":{"values":[{"k":"a","v":1}]},"mark":"bar",
        "encoding":{"x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative","aggregate":"sum"}}}"#;
    let c = Chart::from_json(json).unwrap();
    let mut l = Labels::default();
    l.sum_title = "Summe von {field}".into();
    let alt = c.text_alternative(&l, 10);
    assert_eq!(alt.summary, "T. Sales grew");
    assert_eq!(alt.table.headers, ["k", "Summe von v"]);
}

#[test]
fn more_series_than_the_palette_can_hold_is_refused_when_read() {
    let rows: Vec<String> = (0..9)
        .map(|i| format!(r#"{{"k":"a","v":1,"g":"g{i}"}}"#))
        .collect();
    let enc = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative"},"color":{"field":"g","type":"nominal"}"#;
    let e = Chart::from_json(&doc("bar", &format!("[{}]", rows.join(",")), enc)).unwrap_err();
    assert_eq!(e.path, "/encoding/color");
    assert!(e.message.contains("9 colour categories"), "{e}");
}

#[test]
fn horizontal_bars_sorted_by_value_with_exact_geometry() {
    let rows = r#"[{"k":"a","v":10},{"k":"b","v":30},{"k":"c","v":20}]"#;
    let enc =
        r#""y":{"field":"k","type":"nominal","sort":"-x"},"x":{"field":"v","type":"quantitative"}"#;
    let s = scene(&doc("bar", rows, enc), 400.0, 200.0);
    let r = rects(&s);
    assert_eq!(r.len(), 3);
    // drawn in data order a, b, c; sorted by -x the order top to bottom is b, c, a
    let top_of = |i: usize| r[i][1];
    assert!(top_of(1) < top_of(2) && top_of(2) < top_of(0), "{r:?}");
    // widths are proportional to the values (zero baseline), 30 is the longest
    let w = |i: usize| r[i][2];
    assert!((w(0) / w(1) - 1.0 / 3.0).abs() < 1e-4 && (w(2) / w(1) - 2.0 / 3.0).abs() < 1e-4);
    assert!(texts(&s).contains(&"v".to_string()));
}

#[test]
fn count_aggregates_without_a_field_and_the_title_says_so() {
    let rows = r#"[{"k":"a"},{"k":"a"},{"k":"b"}]"#;
    let enc = r#""x":{"field":"k","type":"nominal"},"y":{"aggregate":"count"}"#;
    let s = scene(&doc("bar", rows, enc), 300.0, 200.0);
    assert!(texts(&s).contains(&"Count of Records".to_string()));
    let r = rects(&s);
    assert_eq!(r.len(), 2);
    assert!(
        (r[0][3] / r[1][3] - 2.0).abs() < 1e-4,
        "a counts twice as many as b"
    );
}

#[test]
fn the_axis_stays_inside_the_box_for_any_size() {
    let rows = r#"[{"k":"alpha","v":3},{"k":"beta","v":5}]"#;
    let c = Chart::from_json(&doc("bar", rows, layout_util::CAT)).unwrap();
    for (w, h) in [
        (1.0, 1.0),
        (20.0, 20.0),
        (50.0, 400.0),
        (400.0, 40.0),
        (3000.0, 2000.0),
    ] {
        let s = c.scene(w, h, &Labels::default(), &Fixed, &Metrics::default());
        assert!(
            s.plot.iter().all(|v| v.is_finite()) && s.plot[2] >= 1.0 && s.plot[3] >= 1.0,
            "{w}x{h}: {:?}",
            s.plot
        );
        assert!(
            s.items
                .iter()
                .all(|i| !matches!(i, Item::Rect { w, h, .. } if *w < 0.0 || *h < 0.0))
        );
    }
}

#[test]
fn natural_size_follows_vega_lite_steps() {
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":5},{"k":"c","v":1},{"k":"d","v":2}]"#;
    let c = Chart::from_json(&doc("bar", rows, layout_util::CAT)).unwrap();
    let (w, h) = c.natural_size(&Labels::default(), &Fixed, &Metrics::default());
    let probe = c.scene(w, h, &Labels::default(), &Fixed, &Metrics::default());
    assert!(
        (probe.plot[2] - 80.0).abs() < 2.0,
        "4 categories x 20 px: {:?}",
        probe.plot
    );
    assert!((probe.plot[3] - 300.0).abs() < 2.0, "{:?}", probe.plot);
}

#[test]
fn pointing_at_a_bar_reads_its_datum() {
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":5}]"#;
    let s = scene(&doc("bar", rows, layout_util::CAT), 300.0, 200.0);
    let r = rects(&s);
    let hit = s
        .hit_at(r[1][0] + r[1][2] / 2.0, r[1][1] + r[1][3] / 2.0, 0.0)
        .expect("a hit");
    assert_eq!(hit.text, "k: b\nv: 5");
    assert!(s.hit_at(2.0, 2.0, 0.0).is_none());
}
