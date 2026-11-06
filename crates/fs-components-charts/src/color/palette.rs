//! Choosing the series colours from the theme's own colours.
//!
//! Pattern: Strategy (greedy farthest-point selection). Candidates are the theme's accent colours
//! and lighter and darker versions of them; those with too little contrast against the surface are
//! left out. Each pick is the candidate whose *worst* difference to the colours already picked (as
//! seen by normal vision, protanopia, deuteranopia and tritanopia) is the largest, and only while
//! that worst difference stays above [`MIN_DIFFERENCE`]: a theme that cannot offer eight colours
//! apart offers fewer, and a chart with more series says so instead of repeating colours. The
//! first colour is the theme's primary. Nothing here is a literal colour.

use super::cvd::worst_difference;
use super::space::{Rgb, contrast, lab, rgb_of_lab};

/// WCAG 1.4.11: graphics need a contrast of 3:1 against their surroundings.
pub const MIN_CONTRAST: f64 = 3.0;
/// How many series colours are offered at most.
pub const SLOTS: usize = 8;
/// The smallest colour difference (CIE 1976 Delta E) allowed between two series colours, for
/// normal vision and for each simulated deficiency. 15 is clearly different side by side.
pub const MIN_DIFFERENCE: f64 = 15.0;

/// Up to [`SLOTS`] colours from `base` (the theme's accents, primary first) that stand out against
/// `surface`, as far apart as possible for all kinds of colour vision.
pub fn pick(base: &[Rgb], surface: Rgb) -> Vec<Rgb> {
    let mut pool: Vec<Rgb> = Vec::new();
    for &c in base {
        let l = lab(c);
        for shift in [0.0, 18.0, -18.0, 32.0, -32.0] {
            let tone = l[0] + shift;
            // a shifted tone must stay clearly coloured: no near-black or near-white copies
            let usable = shift == 0.0 || (28.0..=85.0).contains(&tone) && l[1].hypot(l[2]) >= 25.0;
            let v = if shift == 0.0 {
                c
            } else {
                rgb_of_lab([tone.clamp(5.0, 95.0), l[1], l[2]])
            };
            if usable && contrast(v, surface) >= MIN_CONTRAST {
                pool.push(v);
            }
        }
    }
    let Some(&first) = pool.first() else {
        return Vec::new();
    };
    let mut chosen = vec![first];
    while chosen.len() < SLOTS {
        let best = pool
            .iter()
            .map(|&c| {
                (
                    c,
                    chosen
                        .iter()
                        .map(|&p| worst_difference(c, p))
                        .fold(f64::INFINITY, f64::min),
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        match best {
            Some((c, d)) if d >= MIN_DIFFERENCE => chosen.push(c),
            _ => break,
        }
    }
    chosen
}
