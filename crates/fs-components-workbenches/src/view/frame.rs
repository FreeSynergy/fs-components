//! The outer layout: sidebar and content side by side.
//!
//! Pattern: none — fixed skeleton; the areas are passed in.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

/// Sidebar (search box above `side`) left, `main` right, filling `width` x `height`.
pub(crate) fn frame(
    search: impl ElementBuilder + 'static,
    side: impl ElementBuilder + 'static,
    main: impl ElementBuilder + 'static,
    width: f32,
    height: f32,
) -> Div {
    div()
        .class("fs-rd-root")
        .w(width)
        .h(height)
        .child(
            div()
                .class("fs-rd-side")
                .h(height)
                .min_h(0.0)
                .child(div().class("fs-rd-search").child(search))
                .child(side),
        )
        .child(div().class("fs-rd-main").h(height).min_h(0.0).child(main))
}
