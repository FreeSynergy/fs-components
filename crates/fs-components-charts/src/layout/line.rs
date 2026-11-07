//! Lines and areas: one path per series, broken where a value is invalid.
//!
//! Pattern: none (the layout of two marks; `build` picks it by `match` on the mark kind, there is no trait). Points of a series are joined in order of
//! their x position; a point without a usable x or y ends the path (Vega-Lite's default `invalid`
//! mode for paths). A series that is one point alone is drawn as a symbol so it does not vanish.
//! Areas with a colour stack, series 0 on top.

use std::collections::HashMap;

use super::axis::{Axis, Scale};
use super::ctx::{Ctx, dash_of, shape_of};
use super::dash;
use super::stack::{Entry, stack};
use crate::scene::{Dash, Item, Paint, Scene, Shape};
use crate::spec::{Channel, MarkKind, StackMode};

const LINE_WIDTH: f32 = 2.0;
const DOT: f32 = 3.0;

/// Top and base value of a datum (a line has base 0).
#[derive(Clone, Copy)]
struct Edges {
    top: f64,
    base: f64,
}

/// One usable point on its way to the screen.
struct Pt {
    top: [f32; 2],
    base: f32,
    datum: usize,
}

/// The lines or areas of a chart, in data space.
pub struct Lines {
    edges: Vec<Option<Edges>>,
    kind: MarkKind,
}

impl Lines {
    /// Stack the data for an area; a line keeps its values.
    pub fn new(c: &Ctx<'_>) -> Self {
        let kind = c.spec.mark.kind;
        let mode = c
            .spec
            .encoding
            .y
            .as_ref()
            .and_then(|d| d.stack)
            .unwrap_or(StackMode::Zero);
        let mut groups: HashMap<String, usize> = HashMap::new();
        let (mut entries, mut at) = (Vec::new(), Vec::new());
        for (i, d) in c.data.data.iter().enumerate() {
            let Some(value) = d.cells[Channel::Y.index()].num() else {
                continue;
            };
            let n = groups.len();
            let group = *groups
                .entry(format!("{:?}", d.cells[Channel::X.index()]))
                .or_insert(n);
            entries.push(Entry::new(group, c.rank_of(d), value));
            at.push(i);
        }
        let mut edges = vec![None; c.data.data.len()];
        if kind == MarkKind::Area {
            for (span, &i) in stack(&entries, mode, false).iter().zip(&at) {
                let up = span[1] > 0.0 || span[0] == 0.0;
                edges[i] = Some(if up {
                    Edges {
                        top: span[1],
                        base: span[0],
                    }
                } else {
                    Edges {
                        top: span[0],
                        base: span[1],
                    }
                });
            }
        } else {
            for (e, &i) in entries.iter().zip(&at) {
                edges[i] = Some(Edges {
                    top: e.value,
                    base: 0.0,
                });
            }
        }
        Self { edges, kind }
    }

    /// The range of the y axis: the values, and zero as well for an area.
    pub fn range(&self) -> Option<(f64, f64)> {
        let area = self.kind == MarkKind::Area;
        let mut range: Option<(f64, f64)> = None;
        for e in self.edges.iter().flatten() {
            let (lo, hi) = if area {
                (e.top.min(e.base), e.top.max(e.base))
            } else {
                (e.top, e.top)
            };
            range = Some(range.map_or((lo, hi), |(a, b)| (a.min(lo), b.max(hi))));
        }
        range
    }

    /// Draw one path (and polygon) per series.
    pub fn draw(&self, c: &Ctx<'_>, x: &Axis, y: &Axis, plot: [f32; 4], out: &mut Scene) {
        let Scale::Linear(ys) = &y.scale else { return };
        let to_y = |v: f64| plot[1] + ys.map(v) as f32;
        for s in 0..c.series.len().max(1) {
            // None ends a path: the point cannot be placed
            let mut seq: Vec<(f32, Option<Pt>)> = Vec::new();
            for (i, d) in c
                .data
                .data
                .iter()
                .enumerate()
                .filter(|(_, d)| c.series_of(d) == s)
            {
                let px = x.place(&d.cells[Channel::X.index()], c.labels);
                let pt = px.zip(self.edges[i]).map(|(px, e)| Pt {
                    top: [plot[0] + px, to_y(e.top)],
                    base: to_y(e.base),
                    datum: i,
                });
                seq.push((px.unwrap_or(f32::INFINITY), pt));
            }
            seq.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut run: Vec<Pt> = Vec::new();
            for (_, pt) in seq {
                match pt {
                    Some(p) => run.push(p),
                    None => self.finish(c, s, &mut run, out),
                }
            }
            self.finish(c, s, &mut run, out);
        }
    }

    /// Draw a finished run of points and empty it.
    fn finish(&self, c: &Ctx<'_>, s: usize, run: &mut Vec<Pt>, out: &mut Scene) {
        let merged = decimate(run, c.max_marks / c.series.len().max(1), out);
        let paint = Paint::Series(s);
        let top: Vec<[f32; 2]> = run.iter().map(|p| p.top).collect();
        if self.kind == MarkKind::Area && merged {
            strips(run, paint, out);
        } else if self.kind == MarkKind::Area {
            area(run, paint, out);
        } else if top.len() > 1 {
            let dash = if c.multi() { dash_of(s) } else { Dash::Solid };
            for points in dash::pieces(&top, dash) {
                out.items.push(Item::Path {
                    points,
                    paint,
                    width: LINE_WIDTH,
                });
            }
        }
        if c.spec.mark.point || (top.len() == 1 && self.kind == MarkKind::Line) {
            let shape = if c.multi() {
                shape_of(s)
            } else {
                Shape::Circle
            };
            for at in &top {
                out.items.push(Item::Marker {
                    at: *at,
                    shape,
                    radius: DOT,
                    paint,
                    filled: true,
                    opacity: 1.0,
                });
            }
        }
        for p in run.iter() {
            out.hits.push(c.hit(
                &c.data.data[p.datum],
                [p.top[0] - DOT, p.top[1] - DOT, 2.0 * DOT, 2.0 * DOT],
            ));
        }
        run.clear();
    }
}

/// The polygon between the top edge and the base edge; a single point becomes a vertical rule.
fn area(run: &[Pt], paint: Paint, out: &mut Scene) {
    match run {
        [] => {}
        [p] => out.items.push(Item::Line {
            from: p.top,
            to: [p.top[0], p.base],
            paint,
            width: LINE_WIDTH,
        }),
        _ => {
            let mut points: Vec<[f32; 2]> = run.iter().map(|p| p.top).collect();
            points.extend(run.iter().rev().map(|p| [p.top[0], p.base]));
            out.items.push(Item::Polygon {
                points,
                paint,
                opacity: 1.0,
            });
            let edge = run.iter().map(|p| p.top).collect();
            out.items.push(Item::Path {
                points: edge,
                paint,
                width: 1.0,
            });
        }
    }
}

/// Keep a run to at most about `budget` points without changing what it looks like: per pixel
/// column only the first, lowest, highest and last point stay (the envelope of the line at the
/// resolution of the screen). What cannot be merged that way is counted as omitted.
fn decimate(run: &mut Vec<Pt>, budget: usize, out: &mut Scene) -> bool {
    if run.len() <= budget {
        return false;
    }
    let mut kept: Vec<Pt> = Vec::new();
    let mut column: Vec<Pt> = Vec::new();
    let flush = |column: &mut Vec<Pt>, kept: &mut Vec<Pt>| {
        if column.len() <= 4 {
            kept.append(column);
            return;
        }
        let at = |f: &dyn Fn(&Pt, &Pt) -> bool| {
            (0..column.len()).fold(0, |b, i| if f(&column[i], &column[b]) { i } else { b })
        };
        let mut idx = vec![
            0,
            at(&|a, b| a.top[1] < b.top[1]),
            at(&|a, b| a.top[1] > b.top[1]),
            column.len() - 1,
        ];
        idx.sort_unstable();
        idx.dedup();
        for i in idx {
            kept.push(Pt {
                top: column[i].top,
                base: column[i].base,
                datum: column[i].datum,
            });
        }
        column.clear();
    };
    let mut col = f32::NAN;
    for p in run.drain(..) {
        let c = p.top[0].floor();
        if c != col {
            flush(&mut column, &mut kept);
            col = c;
        }
        column.push(p);
    }
    flush(&mut column, &mut kept);
    if kept.len() > budget {
        out.omitted += kept.len() - budget;
        kept.truncate(budget);
    }
    *run = kept;
    true
}

/// A merged area as one-pixel strips from the base to the highest point of each column. A
/// polygon of that many zigzag points is too slow to fill (minutes in Blinc's tessellator).
fn strips(run: &[Pt], paint: Paint, out: &mut Scene) {
    let mut i = 0;
    while i < run.len() {
        let col = run[i].top[0].floor();
        let mut j = i;
        let mut top = run[i].top[1];
        while j < run.len() && run[j].top[0].floor() == col {
            top = top.min(run[j].top[1]);
            j += 1;
        }
        let base = run[i].base;
        let (y, h) = (top.min(base), (top - base).abs());
        if h > 0.0 {
            out.items.push(Item::Rect {
                x: col,
                y,
                w: 1.0,
                h,
                paint,
                opacity: 1.0,
            });
        }
        i = j;
    }
}
