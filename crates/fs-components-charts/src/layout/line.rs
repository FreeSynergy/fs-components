//! Lines and areas: one path per series, broken where a value is invalid.
//!
//! Pattern: Strategy (one mark layout for two marks). Points of a series are joined in order of
//! their x position; a point without a usable x or y ends the path (Vega-Lite's default `invalid`
//! mode for paths). A series that is one point alone is drawn as a symbol so it does not vanish.
//! Areas with a colour stack, series 0 on top.

use std::collections::HashMap;

use super::axis::{Axis, Scale};
use super::ctx::{Ctx, dash_of, shape_of};
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
            .map_or(StackMode::Zero, |d| d.stack);
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
            entries.push(Entry {
                group,
                series: c.series_of(d),
                value,
            });
            at.push(i);
        }
        let mut edges = vec![None; c.data.data.len()];
        if kind == MarkKind::Area {
            for (span, &i) in stack(&entries, mode).iter().zip(&at) {
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
        let paint = Paint::Series(s);
        let top: Vec<[f32; 2]> = run.iter().map(|p| p.top).collect();
        if self.kind == MarkKind::Area {
            area(run, paint, out);
        } else if top.len() > 1 {
            let dash = if c.multi() { dash_of(s) } else { Dash::Solid };
            out.items.push(Item::Path {
                points: top.clone(),
                paint,
                width: LINE_WIDTH,
                dash,
            });
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
                opacity: 0.7,
            });
            let edge = run.iter().map(|p| p.top).collect();
            out.items.push(Item::Path {
                points: edge,
                paint,
                width: 1.0,
                dash: Dash::Solid,
            });
        }
    }
}
