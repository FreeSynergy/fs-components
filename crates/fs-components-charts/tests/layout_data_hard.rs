//! Hard data at the layout level: huge values, long labels, unicode, dates across DST, many rows.

mod layout_util;

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::Item;
use fs_components_charts::{Chart, Labels};
use layout_util::{CAT, TIME, doc, finite, rects, scene, texts};

#[test]
fn huge_and_tiny_values_stay_finite() {
    let rows = r#"[{"k":"a","v":1.7e308},{"k":"b","v":-1.7e308},{"k":"c","v":1e-300}]"#;
    let s = scene(&doc("bar", rows, CAT), 400.0, 300.0);
    assert!(
        finite(&s),
        "{:?}",
        s.items.iter().take(4).collect::<Vec<_>>()
    );
    assert_eq!(
        rects(&s).len(),
        2,
        "the tiny bar is below a pixel: no rectangle, no NaN"
    );
    let line = scene(
        &doc("line", rows, CAT.replace("nominal", "ordinal").as_str()),
        400.0,
        300.0,
    );
    assert!(finite(&line));
    let t = texts(&line);
    assert!(
        t.iter().any(|x| x.contains('e')),
        "huge ticks use exponent form: {t:?}"
    );
}

#[test]
fn long_labels_are_cut_and_the_plot_keeps_its_room() {
    let long = "x".repeat(300);
    let rows = format!(r#"[{{"k":"{long}","v":1}},{{"k":"b","v":2}}]"#);
    let s = scene(&doc("bar", &rows, CAT), 400.0, 300.0);
    let t = texts(&s);
    let cut = t
        .iter()
        .find(|x| x.ends_with('\u{2026}'))
        .expect("an ellipsis");
    assert!(
        cut.chars().count() * 6 <= 181,
        "{} chars",
        cut.chars().count()
    );
    assert!(s.plot[3] > 100.0 && s.plot[2] > 100.0, "{:?}", s.plot);
    let title = format!(
        r#"{{"title":"{}","data":{{"values":[]}},"mark":"bar","encoding":{{{CAT}}}}}"#,
        "T".repeat(500)
    );
    let s = scene(&title, 200.0, 100.0);
    let t = texts(&s);
    assert!(
        t[0].ends_with('\u{2026}') && t[0].chars().count() * 8 <= 200,
        "{}",
        t[0]
    );
}

#[test]
fn unicode_passes_through_untouched() {
    let names = ["日本語", "Ünïcödé", "😀 emoji", "עברית", "a\u{0301}"];
    let rows: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, n)| format!(r#"{{"k":"{n}","v":{}}}"#, i + 1))
        .collect();
    let s = scene(
        &doc("bar", &format!("[{}]", rows.join(",")), CAT),
        500.0,
        300.0,
    );
    let t = texts(&s);
    for n in names {
        assert!(t.iter().any(|x| x == n), "{n} missing in {t:?}");
    }
}

#[test]
fn dates_across_a_dst_change_are_where_the_offsets_say() {
    // Berlin: noon on 30 March is +01:00, noon on 31 March is +02:00 (23 hours later), 1 April is 24 h more
    let rows = r#"[{"d":"2024-03-30T12:00+01:00","v":1},{"d":"2024-03-31T12:00+02:00","v":2},{"d":"2024-04-01T12:00+02:00","v":3}]"#;
    let s = scene(&doc("point", rows, TIME), 400.0, 300.0);
    let xs: Vec<f32> = s
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Marker { at, .. } = i {
                Some(at[0])
            } else {
                None
            }
        })
        .collect();
    assert_eq!(xs.len(), 3);
    let (d1, d2) = (xs[1] - xs[0], xs[2] - xs[1]);
    assert!((d1 / d2 - 23.0 / 24.0).abs() < 1e-4, "{d1} / {d2}");
    // wall clock without offsets: always 24 h
    let naive = r#"[{"d":"2024-03-30T12:00","v":1},{"d":"2024-03-31T12:00","v":2},{"d":"2024-04-01T12:00","v":3}]"#;
    let s = scene(&doc("point", naive, TIME), 400.0, 300.0);
    let xs: Vec<f32> = s
        .items
        .iter()
        .filter_map(|i| {
            if let Item::Marker { at, .. } = i {
                Some(at[0])
            } else {
                None
            }
        })
        .collect();
    assert!(((xs[1] - xs[0]) - (xs[2] - xs[1])).abs() < 1e-3);
    // the axis of the first data set is written in +01:00: 30 March 12:00 is the first instant
    let c = Chart::from_json(&doc("point", rows, TIME)).unwrap();
    let alt = c.text_alternative(&Labels::default(), 10);
    assert_eq!(alt.table.rows[0][0], "2024-03-30T12:00+01:00");
    assert_eq!(alt.table.rows[1][0], "2024-03-31T12:00+02:00");
}

#[test]
fn a_hundred_thousand_rows_lay_out_quickly_and_stay_finite() {
    let rows: Vec<String> = (0..100_000)
        .map(|i| {
            format!(
                r#"{{"x":{},"y":{},"g":"g{}"}}"#,
                i % 1000,
                (i * 7919) % 1013,
                i % 4
            )
        })
        .collect();
    let json = doc(
        "point",
        &format!("[{}]", rows.join(",")),
        r#""x":{"field":"x","type":"quantitative"},"y":{"field":"y","type":"quantitative"},"color":{"field":"g","type":"nominal"}"#,
    );
    let t = std::time::Instant::now();
    let chart = Chart::from_json(&json).unwrap();
    let parse = t.elapsed();
    let s = chart.scene(
        600.0,
        400.0,
        &Labels::default(),
        &Fixed,
        &Metrics::default(),
    );
    let layout = t.elapsed() - parse;
    let n = s
        .items
        .iter()
        .filter(|i| matches!(i, Item::Marker { .. }))
        .count();
    // marks of one series on one half pixel are merged; what is left fits the budget
    assert!(
        n > 1000 && n <= Metrics::default().max_marks + 4,
        "{n} markers"
    );
    assert!(finite(&s));
    eprintln!("100000 rows: parse+prepare {parse:?}, layout {layout:?}");
}

#[test]
fn many_categories_thin_their_labels_instead_of_overlapping() {
    let rows: Vec<String> = (0..500)
        .map(|i| format!(r#"{{"k":"category {i}","v":{}}}"#, i + 1))
        .collect();
    let s = scene(
        &doc("bar", &format!("[{}]", rows.join(",")), CAT),
        600.0,
        300.0,
    );
    let labelled = texts(&s)
        .iter()
        .filter(|t| t.starts_with("category"))
        .count();
    assert!(
        labelled > 5 && labelled < 60,
        "{labelled} labels for 500 bars"
    );
    assert_eq!(rects(&s).len(), 500);
}
