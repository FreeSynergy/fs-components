//! Bars: one rectangle per datum, from a baseline, stacked when there is a colour.
//!
//! Pattern: Strategy (one mark layout). The category axis has a band scale; the value axis a
//! linear scale that includes zero. A negative value hangs below the zero line. Values of one
//! category pile up (positive up, negative down) with series 0 on top, as in Vega-Lite; with
//! `stack: null` they overlap, drawn at 0.7 opacity as Vega-Lite does for layered bars.

use super::axis::{Axis, Scale};
use super::cats::Cats;
use super::ctx::Ctx;
use super::stack::{Entry, stack};
use crate::scene::{Item, Paint, Scene};
use crate::spec::{Channel, StackMode};

struct Seg {
    cat: usize,
    series: usize,
    span: [f64; 2],
    datum: usize,
}

/// The bars of a chart, in data space.
pub struct Bars {
    vertical: bool,
    segs: Vec<Seg>,
    overlay: bool,
}

impl Bars {
    /// Stack the data over the categories `cats` of the discrete axis.
    pub fn new(c: &Ctx<'_>, cats: &Cats) -> Self {
        let enc = &c.spec.encoding;
        let vertical = enc.x.as_ref().is_some_and(|d| d.ftype.is_discrete());
        let (cat_ch, val_ch) = if vertical {
            (Channel::X, Channel::Y)
        } else {
            (Channel::Y, Channel::X)
        };
        let mode = enc.get(val_ch).map_or(StackMode::Zero, |d| d.stack);
        let mut entries = Vec::new();
        let mut keep = Vec::new();
        for (i, d) in c.data.data.iter().enumerate() {
            let (Some(cat), Some(value)) = (
                cats.index_of(&d.cells[cat_ch.index()], c.labels),
                d.cells[val_ch.index()].num(),
            ) else {
                continue;
            };
            entries.push(Entry {
                group: cat,
                series: c.series_of(d),
                value,
            });
            keep.push(i);
        }
        let spans = stack(&entries, mode);
        let segs = entries
            .iter()
            .zip(&spans)
            .zip(&keep)
            .map(|((e, s), &datum)| Seg {
                cat: e.group,
                series: e.series,
                span: *s,
                datum,
            })
            .collect();
        Self {
            vertical,
            segs,
            overlay: mode == StackMode::Off && c.multi(),
        }
    }

    /// The range of the value axis, zero included.
    pub fn range(&self) -> (f64, f64) {
        self.segs.iter().fold((0.0, 0.0), |(lo, hi), s| {
            (lo.min(s.span[0]), hi.max(s.span[1]))
        })
    }

    /// Draw the bars into `out`; `plot` is `[x, y, w, h]`.
    pub fn draw(&self, c: &Ctx<'_>, x: &Axis, y: &Axis, plot: [f32; 4], out: &mut Scene) {
        let (cat, val) = if self.vertical { (x, y) } else { (y, x) };
        let (Scale::Band(band), Scale::Linear(lin)) = (&cat.scale, &val.scale) else {
            return;
        };
        let opacity = if self.overlay { 0.7 } else { 1.0 };
        for s in &self.segs {
            let start = band.start(s.cat) as f32;
            let width = band.bandwidth() as f32;
            let (a, b) = (lin.map(s.span[0]) as f32, lin.map(s.span[1]) as f32);
            let (lo, len) = (a.min(b), (a - b).abs());
            let (rx, ry, rw, rh) = if self.vertical {
                (plot[0] + start, plot[1] + lo, width, len)
            } else {
                (plot[0] + lo, plot[1] + start, len, width)
            };
            if rw <= 0.0 || rh <= 0.0 {
                continue;
            }
            out.items.push(Item::Rect {
                x: rx,
                y: ry,
                w: rw,
                h: rh,
                paint: Paint::Series(s.series),
                opacity,
            });
            out.hits
                .push(c.hit(&c.data.data[s.datum], [rx, ry, rw, rh]));
        }
    }
}
