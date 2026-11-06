//! Inputs that used to panic, go non-finite or leave a blank chart.

mod layout_util;

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::{Item, Scene};
use fs_components_charts::{Chart, Labels};
use layout_util::{CAT, TIME, doc, finite, rects, scene, texts};

fn lay(json: &str) -> Scene {
    scene(json, 400.0, 300.0)
}

fn no_garbage(s: &Scene) {
    assert!(finite(s), "a coordinate is not a number");
    for t in texts(s) {
        assert!(
            !t.contains("NaN") && !t.contains("inf"),
            "garbage text {t:?}"
        );
    }
}

#[test]
fn a_time_beyond_any_date_is_reported_not_a_panic() {
    for t in ["1e300", "9e15", "-9e15", "1.7e308"] {
        let rows = format!(r#"[{{"d":0,"v":1}},{{"d":{t},"v":2}},{{"d":86400000,"v":3}}]"#);
        let s = lay(&doc("line", &rows, TIME));
        no_garbage(&s);
        let t = texts(&s);
        assert!(
            t.iter().any(|x| x.starts_with("d: not a date (1)")),
            "{t:?}"
        );
    }
    // the edge itself is a date
    let rows = r#"[{"d":-8.64e15,"v":1},{"d":8.64e15,"v":2}]"#;
    no_garbage(&lay(&doc("line", rows, TIME)));
}

#[test]
fn subnormal_and_nearly_equal_numbers_give_finite_charts() {
    let enc = r#""x":{"field":"x","type":"quantitative","scale":{"zero":false}},"y":{"field":"y","type":"quantitative","scale":{"zero":false}}"#;
    for (a, b) in [
        ("5e-324", "5e-324"),
        ("5e-324", "1e-323"),
        ("1", "1.0000000000000002"),
        ("1e-310", "3e-310"),
    ] {
        let rows = format!(r#"[{{"x":{a},"y":{a}}},{{"x":{b},"y":{b}}}]"#);
        for mark in ["point", "line", "area"] {
            let s = lay(&doc(mark, &rows, enc));
            no_garbage(&s);
            assert!(!s.items.is_empty());
        }
    }
    let fixed = r#""x":{"field":"x","type":"quantitative","scale":{"domain":[1,1.0000000000000002]}},"y":{"field":"y","type":"quantitative"}"#;
    let s = lay(&doc(
        "point",
        r#"[{"x":1,"y":1},{"x":1.0000000000000002,"y":2}]"#,
        fixed,
    ));
    no_garbage(&s);
    assert!(
        s.items.iter().any(|i| matches!(i, Item::Marker { .. })),
        "the points are not lost"
    );
}

#[test]
fn a_pie_of_huge_values_is_whole_not_blank() {
    let enc =
        r#""theta":{"field":"v","type":"quantitative"},"color":{"field":"k","type":"nominal"}"#;
    let s = lay(&doc(
        "arc",
        r#"[{"k":"a","v":1.7e308},{"k":"b","v":1.7e308}]"#,
        enc,
    ));
    no_garbage(&s);
    let secs: Vec<(f32, f32)> = s
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Sector { start, end, .. } = i {
                Some((*start, *end))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(secs.len(), 2);
    assert!(
        (secs[0].1 - secs[0].0 - std::f32::consts::PI).abs() < 1e-3,
        "{secs:?}"
    );
}

#[test]
fn normalize_without_colour_reads_percent() {
    let rows = r#"[{"k":"a","v":1},{"k":"b","v":3}]"#;
    let enc = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative","stack":"normalize"}"#;
    let t = texts(&lay(&doc("bar", rows, enc)));
    assert!(
        t.contains(&"100%".to_string()) && t.contains(&"0%".to_string()),
        "{t:?}"
    );
}

#[test]
fn when_every_date_is_unreadable_there_is_no_fake_axis() {
    let rows = r#"[{"d":"Jan 1, 2024","v":1},{"d":"yesterday","v":2},{"d":"","v":3}]"#;
    let s = lay(&doc("line", rows, TIME));
    let t = texts(&s);
    assert!(t.contains(&"No data".to_string()), "{t:?}");
    assert!(!t.iter().any(|x| x.contains("1970")), "{t:?}");
    // and it says why, naming the first thing it could not read
    assert!(
        t.iter()
            .any(|x| x.starts_with("d: not a date (3)") && x.contains("Jan 1, 2024")),
        "{t:?}"
    );
}

#[test]
fn every_note_is_shown() {
    let rows = r#"[{"a":null,"b":"x","c":"y","d":1},{"a":"z","b":null,"c":null,"d":"q"},{"a":"w","b":"x","c":[1,2],"d":{"o":1}}]"#;
    let enc = r#""x":{"field":"a","type":"quantitative"},"y":{"field":"b","type":"quantitative"},"color":{"field":"c","type":"nominal"}"#;
    let s = lay(&doc("point", rows, enc));
    let t = texts(&s);
    let notes = t
        .iter()
        .filter(|x| x.contains("not a number") || x.contains("missing") || x.starts_with("Rows"))
        .count();
    assert!(notes >= 4, "{t:?}");
}

#[test]
fn text_alternative_names_the_rows_it_does_not_list_and_sizes_are_sanitised() {
    let rows: Vec<String> = (0..10)
        .map(|i| format!(r#"{{"k":"c{i}","v":{i}}}"#))
        .collect();
    let c = Chart::from_json(&doc("bar", &format!("[{}]", rows.join(",")), CAT)).unwrap();
    let alt = c.text_alternative(&Labels::default(), 3);
    assert!(
        alt.to_text().ends_with("7 more rows are not listed."),
        "{}",
        alt.to_text()
    );
    for (w, h) in [
        (f32::INFINITY, f32::INFINITY),
        (f32::NAN, 5.0),
        (-3.0, 1e30),
    ] {
        let s = c.scene(w, h, &Labels::default(), &Fixed, &Metrics::default());
        assert!(finite(&s) && s.width.is_finite() && s.height.is_finite());
    }
    let (w, h) = c.natural_size(&Labels::default(), &Fixed, &Metrics::default());
    assert!(w.is_finite() && h.is_finite());
}

#[test]
fn a_string_of_a_sum_is_not_a_number() {
    let rows = r#"[{"k":"a","v":"abc"},{"k":"b","v":2}]"#;
    let enc = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative","aggregate":"sum"}"#;
    let s = lay(&doc("bar", rows, enc));
    assert_eq!(
        rects(&s).len(),
        1,
        "category a has no number to sum, so it has no bar"
    );
    no_garbage(&s);
    assert!(
        !texts(&s).contains(&"a".to_string()),
        "and no place on the axis"
    );
}

#[test]
fn schema_5_asks_for_200_pixels() {
    let rows = r#"[{"x":1,"y":1}]"#;
    let enc = r#""x":{"field":"x","type":"quantitative"},"y":{"field":"y","type":"quantitative"}"#;
    let size = |schema: &str| {
        let json = format!(
            r#"{{{schema}"data":{{"values":{rows}}},"mark":"point","encoding":{{{enc}}}}}"#
        );
        let c = Chart::from_json(&json).unwrap();
        let (w, h) = c.natural_size(&Labels::default(), &Fixed, &Metrics::default());
        let probe = c.scene(w, h, &Labels::default(), &Fixed, &Metrics::default());
        (probe.plot[2], probe.plot[3])
    };
    let (w5, _) = size(r#""$schema":"https://vega.github.io/schema/vega-lite/v5.json","#);
    let (w6, _) = size(r#""$schema":"https://vega.github.io/schema/vega-lite/v6.json","#);
    assert!(
        (w5 - 200.0).abs() < 3.0 && (w6 - 300.0).abs() < 3.0,
        "{w5} {w6}"
    );
}

#[test]
fn opacity_is_that_of_vega_lite() {
    use fs_components_charts::scene::Item;
    // bars and areas opaque (also when not stacked), unaggregated points 0.7 (Vega-Lite 6.5.0)
    let two = r#"[{"x":1,"g":"a","v":3},{"x":2,"g":"a","v":4},{"x":1,"g":"b","v":2},{"x":2,"g":"b","v":5}]"#;
    let enc = r#""x":{"field":"x","type":"quantitative"},"y":{"field":"v","type":"quantitative","stack":null},"color":{"field":"g","type":"nominal"}"#;
    let area = lay(&doc("area", two, enc));
    assert!(
        area.items
            .iter()
            .all(|i| !matches!(i, Item::Polygon { opacity, .. } if *opacity != 1.0))
    );
    let bars = lay(&doc(
        "bar",
        two,
        &enc.replace("quantitative\"},\"y", "nominal\"},\"y"),
    ));
    assert!(
        bars.items
            .iter()
            .all(|i| !matches!(i, Item::Rect { opacity, .. } if *opacity != 1.0))
    );
    let pts = lay(&doc("point", two, enc));
    assert!(
        pts.items
            .iter()
            .any(|i| matches!(i, Item::Marker { opacity, .. } if (*opacity - 0.7).abs() < 1e-6))
    );
}
