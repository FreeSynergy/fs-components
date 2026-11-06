//! Simulating colour vision deficiency, to check that series can be told apart.
//!
//! Pattern: Strategy by deficiency. Full-severity matrices of Machado, Oliveira and Fernandes
//! (2009), applied in linear light. Used to *choose* a palette and to *test* the choice.

use super::space::{Rgb, from_linear_rgb, lab, lab_of_linear, linear};

/// The kinds of deficiency simulated.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deficiency {
    /// Normal vision (no change).
    None,
    /// No red cones.
    Protanopia,
    /// No green cones.
    Deuteranopia,
    /// No blue cones.
    Tritanopia,
}

/// All of them, normal vision first.
pub const ALL: [Deficiency; 4] = [
    Deficiency::None,
    Deficiency::Protanopia,
    Deficiency::Deuteranopia,
    Deficiency::Tritanopia,
];

fn matrix(d: Deficiency) -> [[f64; 3]; 3] {
    match d {
        Deficiency::None => [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        Deficiency::Protanopia => [
            [0.152_286, 1.052_583, -0.204_868],
            [0.114_503, 0.786_281, 0.099_216],
            [-0.003_882, -0.048_116, 1.051_998],
        ],
        Deficiency::Deuteranopia => [
            [0.367_322, 0.860_646, -0.227_968],
            [0.280_085, 0.672_501, 0.047_413],
            [-0.011_820, 0.042_940, 0.968_881],
        ],
        Deficiency::Tritanopia => [
            [1.255_528, -0.076_749, -0.178_779],
            [-0.078_411, 0.930_809, 0.147_602],
            [0.004_733, 0.691_367, 0.303_900],
        ],
    }
}

/// Linear light as seen with the deficiency (not clamped).
pub fn simulate_linear(c: Rgb, d: Deficiency) -> [f64; 3] {
    let (l, m) = (linear(c), matrix(d));
    let row = |r: [f64; 3]| (r[0] * l[0] + r[1] * l[1] + r[2] * l[2]).clamp(0.0, 1.0);
    [row(m[0]), row(m[1]), row(m[2])]
}

/// The colour as seen with the deficiency.
pub fn simulate(c: Rgb, d: Deficiency) -> Rgb {
    from_linear_rgb(simulate_linear(c, d))
}

/// Lab of the colour as seen with the deficiency.
pub fn lab_seen(c: Rgb, d: Deficiency) -> [f64; 3] {
    if d == Deficiency::None {
        lab(c)
    } else {
        lab_of_linear(simulate_linear(c, d))
    }
}

/// The smallest colour difference between two colours over normal vision and all deficiencies.
pub fn worst_difference(a: Rgb, b: Rgb) -> f64 {
    ALL.iter()
        .map(|&d| super::space::delta_e(lab_seen(a, d), lab_seen(b, d)))
        .fold(f64::INFINITY, f64::min)
}
