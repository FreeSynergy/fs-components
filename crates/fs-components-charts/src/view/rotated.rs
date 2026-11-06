//! Rotated texts as Blinc elements.
//!
//! Pattern: Adapter. Blinc's canvas moves the origin of a text with the transform but draws the
//! glyphs unrotated (reproduction in `DESIGN.md`), so a rotated label is a small text element that
//! the layout engine rotates about its own centre. Everything else is drawn on the canvas.

use blinc_layout::prelude::{div, text};

use super::look::Look;
use crate::Element;
use crate::layout::Measure;
use crate::scene::{Anchor, Baseline, Item, Scene};

/// One element for each rotated text of the scene, placed with absolute positions.
pub fn elements(scene: &Scene, look: &Look, m: &dyn Measure) -> Vec<impl Element + use<>> {
    let mut out = Vec::new();
    for item in &scene.items {
        let Item::Text {
            at,
            text: words,
            size,
            bold,
            paint,
            anchor,
            baseline,
            angle,
        } = item
        else {
            continue;
        };
        if *angle == 0.0 {
            continue;
        }
        let px = look.px(*size);
        let (w, h) = (m.width(words, px, *bold) + 2.0, look.metrics.line(*size));
        // the unrotated box, then the vector from the anchor to its centre, turned by the angle
        let x0 = match anchor {
            Anchor::Start => 0.0,
            Anchor::Middle => -w / 2.0,
            _ => -w,
        };
        let y0 = match baseline {
            Baseline::Top => 0.0,
            Baseline::Middle => -h / 2.0,
            _ => -h,
        };
        let (cx, cy) = (x0 + w / 2.0, y0 + h / 2.0);
        let (s, c) = angle.to_radians().sin_cos();
        let centre = [at[0] + cx * c - cy * s, at[1] + cx * s + cy * c];
        let mut t = text(words.clone())
            .size(px)
            .color(look.colour(*paint))
            .no_wrap();
        if *bold {
            t = t.bold();
        }
        out.push(
            div()
                .absolute()
                .left(centre[0] - w / 2.0)
                .top(centre[1] - h / 2.0)
                .w(w)
                .h(h)
                .rotate_deg(*angle)
                .child(t),
        );
    }
    out
}
