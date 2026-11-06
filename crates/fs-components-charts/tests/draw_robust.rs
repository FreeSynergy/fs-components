//! The inputs that panicked Blinc's tessellator, drawn on a real texture.

mod common;

use common::{out_dir, render, view};

fn doc(mark: &str, rows: &str, enc: &str) -> String {
    format!(r#"{{"data":{{"values":{rows}}},"mark":"{mark}","encoding":{{{enc}}}}}"#)
}

#[test]
fn subnormal_and_huge_times_draw_without_a_panic() {
    let enc = r#""x":{"field":"x","type":"quantitative","scale":{"zero":false}},"y":{"field":"y","type":"quantitative","scale":{"zero":false}}"#;
    let (dir, _tmp) = out_dir("robust");
    for (i, rows) in [
        r#"[{"x":5e-324,"y":5e-324}]"#,
        r#"[{"x":5e-324,"y":1e-323},{"x":1e-323,"y":5e-324}]"#,
        r#"[{"x":1,"y":1},{"x":1.0000000000000002,"y":2}]"#,
    ]
    .iter()
    .enumerate()
    {
        for mark in ["point", "line", "area"] {
            let pic = render(
                &view(&doc(mark, rows, enc)),
                300,
                200,
                false,
                &dir,
                &format!("sub_{mark}_{i}"),
            );
            assert!(
                pic.ink([0.0, 0.0, 300.0, 200.0], pic.bg) > 100,
                "{mark} {i} drew something"
            );
        }
    }
    let time = r#""x":{"field":"d","type":"temporal"},"y":{"field":"v","type":"quantitative"}"#;
    let pic = render(
        &view(&doc(
            "line",
            r#"[{"d":1e300,"v":1},{"d":0,"v":2},{"d":86400000,"v":3}]"#,
            time,
        )),
        300,
        200,
        false,
        &dir,
        "time_huge",
    );
    assert!(pic.ink([0.0, 0.0, 300.0, 200.0], pic.bg) > 100);
}

#[test]
fn marks_outside_a_fixed_domain_are_cut_at_the_plot() {
    let (dir, _tmp) = out_dir("clip");
    let rows = r#"[{"k":"a","v":3},{"k":"b","v":100}]"#;
    let enc = r#""x":{"field":"k","type":"nominal"},"y":{"field":"v","type":"quantitative","scale":{"domain":[0,5]}}"#;
    let v = view(&doc("bar", rows, enc));
    let pic = render(&v, 300, 240, false, &dir, "clip");
    let s = v.scene(300.0, 240.0);
    let p = s.plot;
    // above the plot (the 100-high bar would reach there) the surface is bare
    assert_eq!(
        pic.ink([p[0], 0.0, p[2], p[1] - 1.0], pic.bg),
        0,
        "nothing above the plot"
    );
}
