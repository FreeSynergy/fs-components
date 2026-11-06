//! The layout of marks on two axes: scales, margins, then the marks and axes in place.
//!
//! Pattern: Builder with a fixed-point loop for the margins. The room an axis needs depends on its
//! labels, which depend on the length of the axis; the loop runs until the margins stop changing
//! (at most four rounds).

use super::axis::{Axis, Source, Use};
use super::axis_items as ai;
use super::bar::Bars;
use super::build::legend_at;
use super::cats::{Cats, categories};
use super::ctx::Ctx;
use super::line::Lines;
use super::measure::{Measure, Metrics};
use super::point;
use crate::data::{Prepared, time::Zone};
use crate::scene::Scene;
use crate::spec::{Channel, ChannelDef, FieldType, MarkKind, StackMode};

enum Marks {
    Bars(Bars),
    Lines(Lines),
    Points,
}

fn plain_range(data: &Prepared, ch: Channel) -> Option<(f64, f64)> {
    data.data
        .iter()
        .filter_map(|d| d.cells[ch.index()].num())
        .fold(None, |r, v| {
            Some(r.map_or((v, v), |(a, b): (f64, f64)| (a.min(v), b.max(v))))
        })
}

fn offset(data: &Prepared, ch: Channel) -> i32 {
    match data.zones[ch.index()] {
        Zone::Offset(o) => o,
        _ => 0,
    }
}

/// Everything about the two axes that does not depend on the size of the plot.
struct Cart<'a, 'b> {
    c: &'b Ctx<'a>,
    defs: [&'a ChannelDef; 2],
    cats: [Option<Cats>; 2],
    marks: Marks,
}

impl Cart<'_, '_> {
    fn source(&self, i: usize) -> Source {
        if let Some(cs) = &self.cats[i] {
            return Source::Cats(cs.clone());
        }
        let ch = [Channel::X, Channel::Y][i];
        let range = match &self.marks {
            Marks::Bars(b) => Some(b.range()),
            Marks::Lines(l) if i == 1 => l.range(),
            _ => plain_range(self.c.data, ch),
        };
        Source::Range(range.filter(|_| self.defs[i].ftype != FieldType::Nominal))
    }

    fn axes(&self, pw: f32, ph: f32) -> (Axis, Axis) {
        let kind = self.c.spec.mark.kind;
        let make = |i: usize, len: f32| {
            let (ch, d) = ([Channel::X, Channel::Y][i], self.defs[i]);
            let u = Use {
                mark: kind,
                offset: offset(self.c.data, ch),
                percent: d.stack == StackMode::Normalize
                    && matches!(kind, MarkKind::Bar | MarkKind::Area)
                    && self.c.multi(),
                force_zero: kind == MarkKind::Bar,
            };
            Axis::new(ch, d, self.source(i), len, &u, self.c.labels)
        };
        (make(0, pw), make(1, ph))
    }
}

/// Lay out marks and axes; `frame` is `[pad, top, bottom room, legend width]`.
pub(super) fn layout(
    c: &Ctx<'_>,
    m: &dyn Measure,
    k: &Metrics,
    frame: [f32; 4],
    scene: &mut Scene,
) {
    let [pad, top, bottom_room, legend_w] = frame;
    let (w, h) = (scene.width, scene.height);
    let (Some(xd), Some(yd)) = (&c.spec.encoding.x, &c.spec.encoding.y) else {
        return;
    };
    let cats = |ch: Channel, d: &ChannelDef| {
        d.ftype
            .is_discrete()
            .then(|| categories(d, ch, &c.data.data, c.labels))
    };
    let cats = [cats(Channel::X, xd), cats(Channel::Y, yd)];
    let marks = match c.spec.mark.kind {
        MarkKind::Bar => Marks::Bars(Bars::new(
            c,
            cats[0]
                .as_ref()
                .or(cats[1].as_ref())
                .unwrap_or(&Cats::default()),
        )),
        MarkKind::Line | MarkKind::Area => Marks::Lines(Lines::new(c)),
        _ => Marks::Points,
    };
    let cart = Cart {
        c,
        defs: [xd, yd],
        cats,
        marks,
    };
    let (mut left, mut below, mut right) = (40.0_f32, 40.0_f32, pad);
    let mut plot = [0.0; 4];
    let mut pair = cart.axes(1.0, 1.0);
    for _ in 0..4 {
        plot = [
            pad + left,
            top,
            (w - pad - left - legend_w - right).max(1.0),
            (h - top - bottom_room - below).max(1.0),
        ];
        pair = cart.axes(plot[2], plot[3]);
        let next = (
            ai::extent(&pair.1, m, k),
            ai::extent(&pair.0, m, k),
            ai::overhang(&pair.0, m, k).max(pad),
        );
        if (next.0 - left).abs() < 0.5
            && (next.1 - below).abs() < 0.5
            && (next.2 - right).abs() < 0.5
        {
            break;
        }
        (left, below, right) = next;
    }
    scene.plot = plot;
    let (x, y) = pair;
    ai::grid(&x, plot, scene);
    ai::grid(&y, plot, scene);
    match &cart.marks {
        Marks::Bars(b) => b.draw(c, &x, &y, plot, scene),
        Marks::Lines(l) => l.draw(c, &x, &y, plot, scene),
        Marks::Points => point::draw(c, &x, &y, plot, scene),
    }
    ai::draw(&x, plot, m, k, scene);
    ai::draw(&y, plot, m, k, scene);
    legend_at(c, plot, w, m, k, scene);
}
