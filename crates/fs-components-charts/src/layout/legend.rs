//! The legend of a colour channel: a title and one entry per series.
//!
//! Pattern: Template. The symbol of an entry matches the mark (square for areas, bars and arcs,
//! short line with the series' dash for lines, the series' shape for points), so a series can be
//! found by pattern as well as by colour.

use super::axis_items::fit;
use super::ctx::{Ctx, dash_of, shape_of};
use super::dash;
use super::measure::{Measure, Metrics};
use crate::scene::{Anchor, Baseline, Item, Paint, Scene, TextSize};
use crate::spec::{MarkKind, Title};

const SYMBOL: f32 = 12.0;
/// Length of the line sample of a line chart: long enough to show the pattern.
const LINE_SAMPLE: f32 = 30.0;
const GAP: f32 = 6.0;
const ROW: f32 = 4.0;
const LIMIT: f32 = 160.0;
/// Space between plot and legend.
pub const MARGIN: f32 = 18.0;

fn title(c: &Ctx<'_>) -> Option<String> {
    let def = c.spec.encoding.color.as_ref()?;
    match &def.title {
        Title::Hidden => None,
        Title::Text(t) => Some(t.clone()),
        Title::Auto => def.field.clone(),
    }
}

fn symbol(c: &Ctx<'_>) -> f32 {
    if c.spec.mark.kind == MarkKind::Line {
        LINE_SAMPLE
    } else {
        SYMBOL
    }
}

/// Width the legend needs, 0 if there is none.
pub fn width(c: &Ctx<'_>, m: &dyn Measure, k: &Metrics) -> f32 {
    let Some(def) = &c.spec.encoding.color else {
        return 0.0;
    };
    if !def.legend || c.series.is_empty() {
        return 0.0;
    }
    let entries = c
        .series
        .names
        .iter()
        .map(|n| m.width(&fit(n, LIMIT, k.label, false, m), k.label, false))
        .fold(0.0, f32::max);
    let head = title(c).map_or(0.0, |t| {
        m.width(&fit(&t, LIMIT, k.axis_title, true, m), k.axis_title, true)
    });
    MARGIN + (symbol(c) + GAP + entries).max(head)
}

/// Draw the legend with its left edge at `x`, top at `y`.
pub fn draw(c: &Ctx<'_>, x: f32, y: f32, m: &dyn Measure, k: &Metrics, out: &mut Scene) {
    let Some(def) = &c.spec.encoding.color else {
        return;
    };
    if !def.legend || c.series.is_empty() {
        return;
    }
    let mut top = y;
    if let Some(t) = title(c) {
        let text = fit(&t, LIMIT, k.axis_title, true, m);
        out.items.push(Item::Text {
            at: [x, top],
            text,
            size: TextSize::AxisTitle,
            bold: true,
            paint: Paint::Text,
            anchor: Anchor::Start,
            baseline: Baseline::Top,
            angle: 0.0,
        });
        top += k.line(TextSize::AxisTitle) + ROW;
    }
    let row = k.line(TextSize::Label).max(SYMBOL) + ROW;
    for (i, name) in c.series.names.iter().enumerate() {
        let mid = top + row / 2.0 - ROW / 2.0;
        let paint = Paint::Series(i);
        match c.spec.mark.kind {
            MarkKind::Line => {
                for points in dash::pieces(&[[x, mid], [x + LINE_SAMPLE, mid]], dash_of(i)) {
                    out.items.push(Item::Path {
                        points,
                        paint,
                        width: 2.0,
                    });
                }
            }
            MarkKind::Point => out.items.push(Item::Marker {
                at: [x + SYMBOL / 2.0, mid],
                shape: shape_of(i),
                radius: 3.5,
                paint,
                filled: false,
                opacity: 1.0,
            }),
            _ => out.items.push(Item::Rect {
                x,
                y: mid - SYMBOL / 2.0,
                w: SYMBOL,
                h: SYMBOL,
                paint,
                opacity: 1.0,
            }),
        }
        let text = fit(name, LIMIT, k.label, false, m);
        out.items.push(Item::Text {
            at: [x + symbol(c) + GAP, mid],
            text,
            size: TextSize::Label,
            bold: false,
            paint: Paint::Text,
            anchor: Anchor::Start,
            baseline: Baseline::Middle,
            angle: 0.0,
        });
        top += row;
    }
}

/// Height of the legend, to center it against the plot.
pub fn height(c: &Ctx<'_>, k: &Metrics) -> f32 {
    let head = if title(c).is_some() {
        k.line(TextSize::AxisTitle) + ROW
    } else {
        0.0
    };
    head + c.series.len() as f32 * (k.line(TextSize::Label).max(SYMBOL) + ROW)
}
