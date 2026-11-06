//! The outer layout: search and list on the left, the person on the right.
//!
//! Pattern: Template — fixed skeleton; the areas are passed in. Sizes are worked out here because
//! the list must be told its height in pixels.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

/// Height of the bar above the list in pixels.
const SEARCH_HEIGHT: f32 = 60.0;
/// The part of the width the list takes.
const LIST_SHARE: f32 = 0.42;

/// The areas of the frame, ready made.
pub(in crate::people) struct Areas<S, L, D> {
    /// Search field and New button.
    pub search: S,
    /// The list.
    pub list: L,
    /// The person.
    pub detail: D,
}

/// Width and height of the area of the list inside `width` x `height`.
pub(in crate::people) fn list_size(width: f32, height: f32) -> (f32, f32) {
    (
        (width * LIST_SHARE).max(0.0),
        (height - SEARCH_HEIGHT).max(0.0),
    )
}

/// Width of the person's area.
pub(in crate::people) fn detail_width(width: f32) -> f32 {
    (width * (1.0 - LIST_SHARE)).max(0.0)
}

/// Everything, filling `width` x `height`.
pub(in crate::people) fn frame<S, L, D>(areas: Areas<S, L, D>, width: f32, height: f32) -> Div
where
    S: ElementBuilder + 'static,
    L: ElementBuilder + 'static,
    D: ElementBuilder + 'static,
{
    let (list_width, list_height) = list_size(width, height);
    let side = div()
        .class("fs-people-side")
        .flex_col()
        .w(list_width)
        .h(height)
        .min_h(0.0)
        .child(
            div()
                .class("fs-people-search")
                .flex_row()
                .items_center()
                .w(list_width)
                .h(SEARCH_HEIGHT)
                .child(areas.search),
        )
        .child(
            div()
                .w(list_width)
                .h(list_height)
                .min_h(0.0)
                .child(areas.list),
        );
    div()
        .class("fs-people-root")
        .flex_row()
        .w(width)
        .h(height)
        .child(side)
        .child(
            div()
                .class("fs-people-main")
                .flex_col()
                .w(detail_width(width))
                .h(height)
                .min_h(0.0)
                .child(areas.detail),
        )
}
