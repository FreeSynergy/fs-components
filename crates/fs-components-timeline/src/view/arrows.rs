//! Dependency arrows, drawn on a Blinc canvas over the rows.
//!
//! Pattern: Adapter — our points in, calls on Blinc's `DrawContext` out; the routes come from
//! `route`, so the only thing here is the pen. The canvas does not redraw by itself.

use blinc_core::{Brush, DrawContext, Path, Stroke};
use blinc_layout::canvas::canvas;
use blinc_layout::div::Div;
use fs_components_fields::Look;

use super::parts::at;
use crate::layout::Rect;
use crate::route::Arrow;

fn open(points: &[(f32, f32)]) -> Path {
    let mut it = points.iter();
    let Some(first) = it.next() else {
        return Path::new();
    };
    it.fold(Path::new().move_to(first.0, first.1), |p, q| {
        p.line_to(q.0, q.1)
    })
}

fn pen(ctx: &mut dyn DrawContext, arrows: &[Arrow], color: blinc_core::Color) {
    for a in arrows {
        ctx.stroke_path(&open(&a.line), &Stroke::new(1.5), Brush::Solid(color));
        let h = [a.head[0], a.head[1], a.head[2], a.head[0]];
        ctx.fill_path(&open(&h).close(), Brush::Solid(color));
    }
}

/// A canvas over `area` that draws `arrows` in the weak text colour of the style.
pub fn arrows(look: &Look, area: Rect, arrows: Vec<Arrow>) -> Div {
    let color = Look::color(look.palette().text_weak);
    at(area).id("tl-arrows").pointer_events_none().child(
        canvas(move |ctx, _| pen(ctx, &arrows, color))
            .redraw_while(|| false)
            .w(area.w)
            .h(area.h),
    )
}
