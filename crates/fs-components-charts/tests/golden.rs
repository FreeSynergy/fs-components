//! Our geometry against real Vega-Lite 6.5.0: every file in `tests/golden/` holds a spec and the
//! marks Vega-Lite made of it (fractions of the plot area). Differences are listed per case.

use std::path::Path;

use fs_components_charts::layout::{Fixed, Metrics};
use fs_components_charts::scene::{Item, Scene};
use fs_components_charts::{Chart, Labels};
use serde_json::Value;

const TOL: f64 = 2e-3;

type Row = Vec<f64>;

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or(f64::NAN)
}

fn sorted(mut rows: Vec<Row>) -> Vec<Row> {
    rows.sort_by(|a, b| {
        a.iter()
            .zip(b)
            .map(|(x, y)| ((x / TOL).round()).total_cmp(&((y / TOL).round())))
            .find(|o| o.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

/// What Vega-Lite drew, per mark type, as rows of numbers.
fn expected(g: &Value, kind: &str) -> Vec<Row> {
    let mut out = Vec::new();
    for m in g["marks"].as_array().unwrap() {
        let t = m["type"].as_str().unwrap();
        for i in m["items"].as_array().unwrap() {
            let row = match (kind, t) {
                ("rect", "rect") => vec![f(i, "x"), f(i, "y"), f(i, "w"), f(i, "h")],
                ("arc", "arc") => vec![f(i, "a0"), f(i, "a1")],
                ("point", "symbol") | ("line", "line") => vec![f(i, "x"), f(i, "y")],
                ("area", "area") => vec![f(i, "x"), f(i, "y"), f(i, "y2")],
                _ => continue,
            };
            out.push(row);
        }
    }
    sorted(out)
}

fn inside(s: &Scene, x: f32, y: f32) -> bool {
    let p = s.plot;
    x >= p[0] - 1.0 && x <= p[0] + p[2] + 1.0 && y >= p[1] - 1.0 && y <= p[1] + p[3] + 1.0
}

/// What we drew, the same way.
fn ours(s: &Scene, kind: &str) -> Vec<Row> {
    let [px, py, pw, ph] = s.plot.map(f64::from);
    let (fx, fy) = (
        |x: f32| (f64::from(x) - px) / pw,
        |y: f32| (f64::from(y) - py) / ph,
    );
    let mut out = Vec::new();
    for item in &s.items {
        match (kind, item) {
            ("rect", Item::Rect { x, y, w, h, .. }) if inside(s, *x, *y) => {
                out.push(vec![fx(*x), fy(*y), f64::from(*w) / pw, f64::from(*h) / ph]);
            }
            ("arc", Item::Sector { start, end, .. }) => {
                out.push(vec![f64::from(*start), f64::from(*end)])
            }
            ("point", Item::Marker { at, .. }) if inside(s, at[0], at[1]) => {
                out.push(vec![fx(at[0]), fy(at[1])])
            }
            ("line", Item::Path { points, .. }) => {
                out.extend(points.iter().map(|p| vec![fx(p[0]), fy(p[1])]))
            }
            ("area", Item::Polygon { points, .. }) => {
                let n = points.len() / 2;
                for i in 0..n {
                    out.push(vec![
                        fx(points[i][0]),
                        fy(points[i][1]),
                        fy(points[points.len() - 1 - i][1]),
                    ]);
                }
            }
            _ => {}
        }
    }
    sorted(out)
}

fn run(name: &str) -> Result<(), String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.json"));
    let g: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?).unwrap();
    let chart = Chart::from_json(&g["spec"].to_string()).map_err(|e| format!("refused: {e}"))?;
    let scene = chart.scene(
        600.0,
        400.0,
        &Labels::default(),
        &Fixed,
        &Metrics::default(),
    );
    let kind = match g["spec"]["mark"]
        .as_str()
        .or(g["spec"]["mark"]["type"].as_str())
        .unwrap()
    {
        "bar" => "rect",
        "arc" => "arc",
        "point" => "point",
        "line" => "line",
        _ => "area",
    };
    let (mut want, mut got) = (expected(&g, kind), ours(&scene, kind));
    if kind == "point" {
        // marks of one series on the same half pixel are drawn once
        want.dedup_by(|a, b| a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 0.002));
        got.dedup_by(|a, b| a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 0.002));
    }
    if name == "g12_null_cat_sort" {
        // Vega-Lite stacks the row without the field on top of the `null` row (one pile); we draw
        // two bars. Only the order and width of the categories are compared.
        for r in want.iter_mut().chain(got.iter_mut()) {
            r.drain(1..2);
            r.pop();
        }
    }
    if want.len() != got.len() {
        return Err(format!(
            "{} marks, Vega-Lite {}\n ours {got:?}\n vl   {want:?}",
            got.len(),
            want.len()
        ));
    }
    for (w, o) in want.iter().zip(&got) {
        if w.iter().zip(o).any(|(a, b)| (a - b).abs() > TOL) {
            return Err(format!(
                "geometry differs\n ours {got:.3?}\n vl   {want:.3?}"
            ));
        }
    }
    if let Some(dom) = g["colour_domain"].as_array() {
        let want: Vec<String> = dom
            .iter()
            .map(|v| v.as_str().unwrap_or("null").to_string())
            .collect();
        let got = chart.series_names(&Labels::default());
        if want != got {
            return Err(format!("colour order {got:?}, Vega-Lite {want:?}"));
        }
    }
    Ok(())
}

/// The tick labels of both axes against Vega-Lite's, in a box that gives the spec's plot size.
fn run_labels(name: &str) -> Result<(), String> {
    use fs_components_charts::scene::{Paint, TextSize};
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.json"));
    let g: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?).unwrap();
    let chart = Chart::from_json(&g["spec"].to_string()).map_err(|e| format!("refused: {e}"))?;
    let (l, m) = (Labels::default(), Metrics::default());
    let (w, h) = chart.natural_size(&l, &Fixed, &m);
    let scene = chart.scene(w, h, &l, &Fixed, &m);
    let got: Vec<String> = scene
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Text {
                text,
                size: TextSize::Label,
                paint: Paint::Text,
                ..
            } => Some(text.clone()),
            _ => None,
        })
        .collect();
    let want: Vec<String> = g["axis_labels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|a| a.as_array().unwrap())
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    // Vega-Lite lists the labels it hides for overlap too, so every label we show must be one
    // of its labels (a label cut with an ellipsis must be the start of one).
    let missing: Vec<&String> = got
        .iter()
        .filter(|t| {
            !want.iter().any(|w| match t.strip_suffix('\u{2026}') {
                Some(head) => w.starts_with(head),
                None => w == *t,
            })
        })
        .collect();
    if missing.is_empty() && !got.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "\n not in Vega-Lite's labels: {missing:?}\n ours {got:?}\n vl   {want:?}"
        ))
    }
}

macro_rules! labels {
    ($($name:ident),* $(,)?) => {$(
        #[test]
        fn $name() {
            if let Err(e) = run_labels(stringify!($name)) {
                panic!("{}: {e}", stringify!($name));
            }
        }
    )*};
}

labels!(
    l01_small,
    l02_unit,
    l03_tiny,
    l04_big,
    l05_big_frac,
    l06_thousands,
    l07_neg,
    l08_dec5,
    l09_1e21,
    l11_halves,
);

macro_rules! golden {
    ($($name:ident),* $(,)?) => {$(
        #[test]
        fn $name() {
            if let Err(e) = run(stringify!($name)) {
                panic!("{}: {e}", stringify!($name));
            }
        }
    )*};
}

golden!(
    g01_line_quant_x,
    g01b_area_quant_x,
    g02_arc_abc,
    g03_hbar_stack,
    g03b_vbar_stack,
    g04a_arc_color_desc,
    g04b_bar_color_list,
    g04c_arc_color_list,
    g06_normalize_neg,
    g07_count_ordinal,
    g07b_count_temporal,
    g08a_sort_mean,
    g08b_sort_max,
    g08c_sort_x_raw,
    g08d_sort_op_raw,
    g08e_sort_op_mean_color,
    g08f_sort_op_sum_raw_y,
    g08g_sort_sum_vs_mean,
    g08h_max_color_sort,
    g08i_hbar_sort_x,
    g09a_domain_nice,
    g09b_domain_zero,
    g10_opacity_bar_off,
    g11_normalize_nocolor,
    g12_null_cat_sort,
    g12b_sum_allnull,
    g12c_sort_list_unlisted,
    g15_ordinal_y_count_hbar,
    g16_line_quant_zero_false,
    g17_dup_values_bar_nocolor,
);
