//! Real data problems at the layout level: exact numbers, nothing panics, nothing is silent.

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::{Item, Paint, Scene};
use fs_components_charts::{Chart, Labels};

fn scene(json: &str, w: f32, h: f32) -> Scene {
    Chart::from_json(json)
        .unwrap_or_else(|e| panic!("{e}"))
        .scene(w, h, &Labels::default(), &Fixed, &Metrics::default())
}

fn doc(mark: &str, rows: &str, enc: &str) -> String {
    format!(r#"{{"data":{{"values":{rows}}},"mark":"{mark}","encoding":{{{enc}}}}}"#)
}

const CAT: &str = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative"}"#;
const TIME: &str = r#""x":{"field":"d","type":"temporal"},"y":{"field":"v","type":"quantitative"}"#;

fn texts(s: &Scene) -> Vec<String> {
    s.items
        .iter()
        .filter_map(|i| {
            if let Item::Text { text, .. } = i {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

fn rects(s: &Scene) -> Vec<[f32; 4]> {
    s.items
        .iter()
        .filter_map(|i| {
            if let Item::Rect { x, y, w, h, .. } = i {
                Some([*x, *y, *w, *h])
            } else {
                None
            }
        })
        .collect()
}

fn finite(s: &Scene) -> bool {
    let ok = |v: &[f32]| v.iter().all(|x| x.is_finite());
    s.items.iter().all(|i| match i {
        Item::Rect { x, y, w, h, .. } => ok(&[*x, *y, *w, *h]),
        Item::Line { from, to, .. } => ok(from) && ok(to),
        Item::Path { points, .. } | Item::Polygon { points, .. } => points.iter().all(|p| ok(p)),
        Item::Marker { at, .. } => ok(at),
        Item::Text { at, .. } => ok(at),
        Item::Sector {
            centre, start, end, ..
        } => ok(centre) && ok(&[*start, *end]),
        _ => true,
    })
}

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
    assert!(
        parse.as_secs() < 10 && layout.as_secs() < 10,
        "parse {parse:?} layout {layout:?}"
    );
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
