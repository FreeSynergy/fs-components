//! Dashed lines, made of pieces.
//!
//! Pattern: Pure function. Blinc 0.6 has no dash code in its GPU stroke (`Stroke::dash` is read
//! by nothing), so a dashed line is cut into its dashes here and each dash is an ordinary path.
//! The pattern runs on along the whole polyline, round corners included.

use crate::scene::Dash;

/// Lengths in pixels of dash, gap, dash, gap ... for a pattern (a line is 2 px wide).
pub fn pattern(d: Dash) -> &'static [f32] {
    match d {
        Dash::Dashed => &[8.0, 5.0],
        Dash::Dotted => &[2.0, 4.0],
        Dash::DashDot => &[8.0, 4.0, 2.0, 4.0],
        _ => &[],
    }
}

/// The dashes of a polyline: each piece is a short path; a solid pattern returns the line as is.
pub fn pieces(points: &[[f32; 2]], d: Dash) -> Vec<Vec<[f32; 2]>> {
    let pat = pattern(d);
    if pat.is_empty() || points.len() < 2 {
        return vec![points.to_vec()];
    }
    let mut out: Vec<Vec<[f32; 2]>> = Vec::new();
    let (mut i, mut left, mut on) = (0, pat[0], true);
    let mut current: Vec<[f32; 2]> = Vec::new();
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        if !len.is_finite() || len == 0.0 {
            continue;
        }
        let mut at = 0.0;
        if on && current.is_empty() {
            current.push(a);
        }
        while len - at > left {
            at += left;
            let t = at / len;
            let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            if on {
                current.push(p);
                out.push(std::mem::take(&mut current));
            } else {
                current.push(p);
            }
            on = !on;
            i = (i + 1) % pat.len();
            left = pat[i];
        }
        left -= len - at;
        if on {
            current.push(b);
        }
    }
    if on && current.len() > 1 {
        out.push(current);
    }
    out
}
