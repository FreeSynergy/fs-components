//! Drawing an axis: line, ticks, grid, labels, title; and how much room it needs.
//!
//! Pattern: Template — the x and y axis share the steps (labels that fit, thinned by parity when
//! they would overlap, shortened with an ellipsis when too long), only the geometry differs.
//! The x axis of a discrete field turns its labels by -90 degrees, as Vega-Lite does.

use super::axis::Axis;
use super::measure::{Measure, Metrics};
use crate::scene::{Anchor, Baseline, Item, Paint, Scene, TextSize};
use crate::spec::Channel;

const LABEL_LIMIT: f32 = 180.0;

fn rotated(a: &Axis) -> bool {
    a.channel == Channel::X && a.cats.is_some()
}

/// `text` cut with an ellipsis so it is at most `limit` wide.
pub fn fit(text: &str, limit: f32, px: f32, bold: bool, m: &dyn Measure) -> String {
    if m.width(text, px, bold) <= limit {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let (mut lo, mut hi) = (0, chars.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let cut: String = chars[..mid].iter().collect::<String>() + "\u{2026}";
        if m.width(&cut, px, bold) <= limit {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo].iter().collect::<String>() + "\u{2026}"
}

fn labels(a: &Axis, m: &dyn Measure, k: &Metrics, limit: f32) -> Vec<String> {
    a.ticks
        .iter()
        .map(|t| fit(&t.label, limit, k.label, false, m))
        .collect()
}

/// Which ticks get a label: every n-th, with n the smallest power of two that stops overlaps.
fn stride(a: &Axis, texts: &[String], m: &dyn Measure, k: &Metrics) -> usize {
    let size = |s: &String| {
        if rotated(a) || a.channel == Channel::Y {
            k.line(TextSize::Label)
        } else {
            m.width(s, k.label, false)
        }
    };
    let mut n = 1;
    while n < texts.len().max(1) {
        let kept: Vec<(f32, f32)> = a
            .ticks
            .iter()
            .zip(texts)
            .step_by(n)
            .map(|(t, s)| (t.pos, size(s)))
            .collect();
        let overlap = kept
            .windows(2)
            .any(|w| (w[1].0 - w[0].0).abs() < f32::midpoint(w[0].1, w[1].1) + 2.0);
        if !overlap {
            break;
        }
        n *= 2;
    }
    n
}

/// Room needed beside (y) or below (x) the plot.
pub fn extent(a: &Axis, m: &dyn Measure, k: &Metrics) -> f32 {
    if !a.visible {
        return 0.0;
    }
    let limit = LABEL_LIMIT;
    let mut size = 0.0;
    if a.labels {
        let t = labels(a, m, k, limit);
        let widest = t
            .iter()
            .map(|s| m.width(s, k.label, false))
            .fold(0.0, f32::max);
        size += k.tick
            + k.label_gap
            + if a.channel == Channel::Y || rotated(a) {
                widest
            } else {
                k.line(TextSize::Label)
            };
    } else {
        size += k.tick;
    }
    if a.title.is_some() {
        size += k.title_gap + k.line(TextSize::AxisTitle);
    }
    size
}

/// Room needed past the right end of the plot by the last x label.
pub fn overhang(a: &Axis, m: &dyn Measure, k: &Metrics) -> f32 {
    if !a.visible || !a.labels || a.channel != Channel::X || rotated(a) {
        return 0.0;
    }
    let last = labels(a, m, k, LABEL_LIMIT)
        .pop()
        .map_or(0.0, |s| m.width(&s, k.label, false) / 2.0);
    (a.ticks.last().map_or(0.0, |t| t.pos) + last - a.len).max(0.0)
}

/// Gridlines first (they go under the marks), then the axes.
pub fn grid(a: &Axis, plot: [f32; 4], out: &mut Scene) {
    if !a.visible || !a.grid {
        return;
    }
    for t in &a.ticks {
        let (from, to) = if a.channel == Channel::X {
            (
                [plot[0] + t.pos, plot[1]],
                [plot[0] + t.pos, plot[1] + plot[3]],
            )
        } else {
            (
                [plot[0], plot[1] + t.pos],
                [plot[0] + plot[2], plot[1] + t.pos],
            )
        };
        out.items.push(Item::Line {
            from,
            to,
            paint: Paint::Grid,
            width: 1.0,
        });
    }
}

/// Domain line, ticks, labels and title.
pub fn draw(a: &Axis, plot: [f32; 4], m: &dyn Measure, k: &Metrics, out: &mut Scene) {
    if !a.visible {
        return;
    }
    let x = a.channel == Channel::X;
    let (px, py, pw, ph) = (plot[0], plot[1], plot[2], plot[3]);
    let line = |from: [f32; 2], to: [f32; 2]| Item::Line {
        from,
        to,
        paint: Paint::Axis,
        width: 1.0,
    };
    out.items.push(if x {
        line([px, py + ph], [px + pw, py + ph])
    } else {
        line([px, py], [px, py + ph])
    });
    let texts = labels(a, m, k, LABEL_LIMIT);
    let n = stride(a, &texts, m, k);
    let widest = texts
        .iter()
        .map(|s| m.width(s, k.label, false))
        .fold(0.0, f32::max);
    for (i, t) in a.ticks.iter().enumerate() {
        let (at, tip) = if x {
            ([px + t.pos, py + ph], [px + t.pos, py + ph + k.tick])
        } else {
            ([px, py + t.pos], [px - k.tick, py + t.pos])
        };
        out.items.push(line(at, tip));
        if !a.labels || i % n != 0 {
            continue;
        }
        let (anchor, baseline, angle, pos) = match (x, rotated(a)) {
            (true, true) => (
                Anchor::End,
                Baseline::Middle,
                -90.0,
                [tip[0], tip[1] + k.label_gap],
            ),
            (true, false) => (
                Anchor::Middle,
                Baseline::Top,
                0.0,
                [tip[0], tip[1] + k.label_gap],
            ),
            (false, _) => (
                Anchor::End,
                Baseline::Middle,
                0.0,
                [tip[0] - k.label_gap, tip[1]],
            ),
        };
        out.items.push(Item::Text {
            at: pos,
            text: texts[i].clone(),
            size: TextSize::Label,
            bold: false,
            paint: Paint::Text,
            anchor,
            baseline,
            angle,
        });
    }
    if let Some(title) = &a.title {
        let title = fit(title, a.len, k.axis_title, true, m);
        let lines = k.line(TextSize::AxisTitle);
        let labels_room = if !a.labels {
            0.0
        } else if x && !rotated(a) {
            k.line(TextSize::Label)
        } else {
            widest
        };
        let past = k.tick
            + if a.labels {
                k.label_gap + labels_room
            } else {
                0.0
            }
            + k.title_gap
            + lines / 2.0;
        let (at, angle) = if x {
            ([px + pw / 2.0, py + ph + past], 0.0)
        } else {
            ([px - past, py + ph / 2.0], -90.0)
        };
        out.items.push(Item::Text {
            at,
            text: title,
            size: TextSize::AxisTitle,
            bold: true,
            paint: Paint::Text,
            anchor: Anchor::Middle,
            baseline: Baseline::Middle,
            angle,
        });
    }
}
