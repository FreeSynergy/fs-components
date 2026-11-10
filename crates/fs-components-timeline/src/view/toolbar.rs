//! The bar above the timeline: the zoom levels.
//!
//! Pattern: View function — state and `Say` in, elements out; a click calls the handle.

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};

use crate::handle::Timeline;
use crate::model::Zoom;

/// One `blinc_cn` button per zoom level, the current one primary.
pub fn toolbar(t: &Timeline) -> Div {
    let s = t.look.style;
    let current = t.snapshot().zoom;
    let mut row = div()
        .id("tl-toolbar")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .w_full()
        .h(s.heights.control + s.spacing.s);
    for z in Zoom::ALL {
        let me = t.clone();
        let variant = if z == current {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        };
        row = row.child(
            cn::button(t.say.say(&format!("timeline.zoom.{}", z.name())))
                .variant(variant)
                .on_click(move |_| {
                    me.set_zoom(z);
                })
                .build_component()
                .id(&format!("tl-zoom-{}", z.name())),
        );
    }
    row
}
