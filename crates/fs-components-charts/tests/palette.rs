//! The series colours come from the theme, stay legible, and stay apart for every kind of vision.

use blinc_core::Color;
use blinc_theme::{ColorScheme, ColorToken, ThemeState};
use fs_components_charts::color::cvd::{ALL, Deficiency, lab_seen, simulate, worst_difference};
use fs_components_charts::color::space::{contrast, delta_e, lab, rgb_of_lab};
use fs_components_charts::color::{MIN_CONTRAST, MIN_DIFFERENCE, pick};

fn rgb(c: Color) -> [f32; 3] {
    [c.r, c.g, c.b]
}

#[test]
fn colour_maths_matches_known_values() {
    // white and black in Lab, and the WCAG extremes
    let w = lab([1.0, 1.0, 1.0]);
    assert!(
        (w[0] - 100.0).abs() < 0.05 && w[1].abs() < 0.05 && w[2].abs() < 0.05,
        "{w:?}"
    );
    assert!(lab([0.0, 0.0, 0.0])[0].abs() < 1e-9);
    assert!((contrast([0.0; 3], [1.0; 3]) - 21.0).abs() < 1e-9);
    // pure sRGB red: L 53.24, a 80.09, b 67.20
    let r = lab([1.0, 0.0, 0.0]);
    assert!(
        (r[0] - 53.24).abs() < 0.1 && (r[1] - 80.09).abs() < 0.2 && (r[2] - 67.20).abs() < 0.2,
        "{r:?}"
    );
    // Lab round trip
    let c = [0.2, 0.5, 0.9];
    let back = rgb_of_lab(lab(c));
    assert!((0..3).all(|i| (back[i] - c[i]).abs() < 0.003), "{back:?}");
}

#[test]
fn simulation_keeps_greys_and_collapses_red_and_green() {
    for d in ALL {
        let g = simulate([0.5, 0.5, 0.5], d);
        assert!((0..3).all(|i| (g[i] - 0.5).abs() < 0.01), "{d:?}: {g:?}");
    }
    // red against green: far apart for normal vision, close for red-green deficiency
    let (red, green) = ([0.88, 0.1, 0.14], [0.15, 0.67, 0.18]);
    assert!(delta_e(lab(red), lab(green)) > 60.0);
    assert!(
        delta_e(
            lab_seen(red, Deficiency::Deuteranopia),
            lab_seen(green, Deficiency::Deuteranopia)
        ) < 30.0
    );
    assert!(worst_difference(red, green) < 30.0);
}

fn check(scheme: ColorScheme) -> Vec<[f32; 3]> {
    ThemeState::init(blinc_cn::cn_bundle(), scheme);
    let t = ThemeState::get();
    let accents: Vec<_> = [
        ColorToken::Primary,
        ColorToken::Warning,
        ColorToken::Info,
        ColorToken::Secondary,
        ColorToken::Accent,
        ColorToken::Success,
        ColorToken::Error,
    ]
    .iter()
    .map(|&a| rgb(t.color(a)))
    .collect();
    let bg = rgb(t.color(ColorToken::Background));
    let p = pick(&accents, bg);
    assert_eq!(p[0], accents[0], "the first series is the theme's primary");
    for (i, c) in p.iter().enumerate() {
        assert!(
            contrast(*c, bg) >= MIN_CONTRAST,
            "{scheme:?} colour {i} has contrast {}",
            contrast(*c, bg)
        );
        for (j, d) in p.iter().enumerate().take(i) {
            let diff = worst_difference(*c, *d);
            assert!(
                diff >= MIN_DIFFERENCE,
                "{scheme:?}: colours {i} and {j} differ by only {diff:.1} for some vision"
            );
        }
    }
    p
}

#[test]
fn light_and_dark_themes_give_legible_colours_that_stay_apart() {
    let (light, dark) = (check(ColorScheme::Light), check(ColorScheme::Dark));
    // the default themes offer at least five colours apart; this is what charts may use
    assert!(
        light.len() >= 5 && dark.len() >= 5,
        "{} / {}",
        light.len(),
        dark.len()
    );
    assert_ne!(light, dark, "the palette follows the theme");
}

#[test]
fn a_theme_with_only_alike_colours_offers_few_and_never_repeats() {
    let bg = [1.0, 1.0, 1.0];
    let alike = [[0.10, 0.20, 0.60], [0.11, 0.21, 0.61], [0.10, 0.20, 0.60]];
    let p = pick(&alike, bg);
    assert!(!p.is_empty() && p.len() <= 5, "{p:?}");
    let invisible = [[0.99, 0.99, 0.99]];
    assert!(
        pick(&invisible, bg).is_empty(),
        "a colour that cannot be seen on the surface is not offered"
    );
}
