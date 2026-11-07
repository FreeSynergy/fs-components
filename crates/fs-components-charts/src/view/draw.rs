//! Drawing a scene on a Blinc draw context.
//!
//! Pattern: Adapter over `Item` — one function per shape; the only code that talks to Blinc's
//! drawing API. Colours come from the `Look`, text widths from `Measure`.

use blinc_core::{
    Brush, ClipShape, Color, CornerRadius, DrawContext, FontWeight, Point, Rect, Stroke, TextAlign,
    TextBaseline, TextStyle, Transform,
};

use super::look::Look;
use super::shapes::{closed, open, polygon, sector};
use crate::layout::Measure;
use crate::scene::{Anchor, Baseline, Item, Paint, Scene, Shape, TextSize};

/// Draw every item of `scene`.
pub fn draw(ctx: &mut dyn DrawContext, scene: &Scene, look: &Look, m: &dyn Measure) {
    let clip = scene.clip_marks && scene.marks_from < scene.items.len();
    for (n, item) in scene.items.iter().enumerate() {
        if clip && n == scene.marks_from {
            let p = scene.plot;
            ctx.push_clip(ClipShape::Rect(Rect::new(p[0], p[1], p[2], p[3])));
        }
        one(ctx, item, look, m);
    }
    if clip {
        ctx.pop_clip();
    }
}

/// Draw one item.
fn one(ctx: &mut dyn DrawContext, item: &Item, look: &Look, m: &dyn Measure) {
    match item {
        Item::Rect {
            x,
            y,
            w,
            h,
            paint,
            opacity,
        } => {
            let c = alpha(look.colour(*paint), *opacity);
            ctx.fill_rect(
                Rect::new(*x, *y, *w, *h),
                CornerRadius::default(),
                Brush::Solid(c),
            );
        }
        Item::Line {
            from,
            to,
            paint,
            width,
        } => {
            let p = open(&[*from, *to]);
            ctx.stroke_path(&p, &Stroke::new(*width), Brush::Solid(look.colour(*paint)));
        }
        Item::Path {
            points,
            paint,
            width,
        } => {
            let s = Stroke::new(*width);
            ctx.stroke_path(&open(points), &s, Brush::Solid(look.colour(*paint)));
        }
        Item::Polygon {
            points,
            paint,
            opacity,
        } => {
            let c = alpha(look.colour(*paint), *opacity);
            ctx.fill_path(&closed(points), Brush::Solid(c));
        }
        Item::Marker {
            at,
            shape,
            radius,
            paint,
            filled,
            opacity,
        } => {
            marker(
                ctx,
                *at,
                *shape,
                *radius,
                alpha(look.colour(*paint), *opacity),
                *filled,
            );
        }
        Item::Sector {
            centre,
            inner,
            outer,
            start,
            end,
            paint,
        } => {
            let p = sector(*centre, *inner, *outer, *start, *end);
            ctx.fill_path(&p, Brush::Solid(look.colour(*paint)));
            let gap = Stroke::new(1.5);
            ctx.stroke_path(&p, &gap, Brush::Solid(look.colour(Paint::Background)));
        }
        Item::Text { angle, .. } if *angle != 0.0 => {} // a text element, see `rotated`
        Item::Text {
            at,
            text,
            size,
            bold,
            paint,
            anchor,
            baseline,
            angle,
        } => {
            label(
                ctx,
                look,
                m,
                (*at, text, *size, *bold, *paint),
                (*anchor, *baseline, *angle),
            );
        }
    }
}

fn alpha(c: Color, a: f32) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a * a)
}

fn marker(ctx: &mut dyn DrawContext, at: [f32; 2], shape: Shape, r: f32, c: Color, filled: bool) {
    let stroke = Stroke::new(2.0);
    if let Some(poly) = polygon(shape, at, r) {
        let p = closed(&poly);
        if filled {
            ctx.fill_path(&p, Brush::Solid(c));
        } else {
            ctx.stroke_path(&p, &stroke, Brush::Solid(c));
        }
    } else if shape == Shape::Cross {
        let h = r * 1.2;
        ctx.stroke_path(
            &open(&[[at[0] - h, at[1]], [at[0] + h, at[1]]]),
            &stroke,
            Brush::Solid(c),
        );
        ctx.stroke_path(
            &open(&[[at[0], at[1] - h], [at[0], at[1] + h]]),
            &stroke,
            Brush::Solid(c),
        );
    } else if filled {
        ctx.fill_circle(Point::new(at[0], at[1]), r, Brush::Solid(c));
    } else {
        ctx.stroke_circle(Point::new(at[0], at[1]), r, &stroke, Brush::Solid(c));
    }
}

type Words<'a> = ([f32; 2], &'a str, TextSize, bool, Paint);

fn label(
    ctx: &mut dyn DrawContext,
    look: &Look,
    m: &dyn Measure,
    w: Words<'_>,
    place: (Anchor, Baseline, f32),
) {
    let (at, text, size, bold, paint) = w;
    let (anchor, baseline, angle) = place;
    let px = look.px(size);
    let width = m.width(text, px, bold);
    let line = look.metrics.line(size);
    let dx = match anchor {
        Anchor::Start => 0.0,
        Anchor::Middle => -width / 2.0,
        _ => -width,
    };
    let dy = match baseline {
        Baseline::Top => 0.0,
        Baseline::Middle => -line / 2.0,
        _ => -line,
    };
    let style = TextStyle {
        size: px,
        color: look.colour(paint),
        weight: if bold {
            FontWeight::Bold
        } else {
            FontWeight::Regular
        },
        align: TextAlign::Left,
        baseline: TextBaseline::Top,
        ..TextStyle::default()
    };
    if angle == 0.0 {
        ctx.draw_text(text, Point::new(at[0] + dx, at[1] + dy), &style);
    } else {
        ctx.push_transform(Transform::translate(at[0], at[1]));
        ctx.push_transform(Transform::rotate(angle.to_radians()));
        ctx.draw_text(text, Point::new(dx, dy), &style);
        ctx.pop_transform();
        ctx.pop_transform();
    }
}
