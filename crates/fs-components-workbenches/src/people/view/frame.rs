//! The outer layout: search and list on the left, the person on the right.
//!
//! Pattern: none — fixed skeleton; the areas are passed in. Sizes are worked out here because
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

/// Below this width the list stands above the person, each with the whole width.
const STACK_BELOW: f32 = 560.0;

/// Do the two areas stand one above the other?
fn stacked(width: f32) -> bool {
    width < STACK_BELOW
}

/// Width of the column of the list, and its height.
fn side_size(width: f32, height: f32) -> (f32, f32) {
    if stacked(width) {
        (width.max(0.0), (height / 2.0).max(0.0))
    } else {
        ((width * LIST_SHARE).max(0.0), height.max(0.0))
    }
}

/// Width and height of the area of the list inside `width` x `height`.
pub(in crate::people) fn list_size(width: f32, height: f32) -> (f32, f32) {
    let (side_w, side_h) = side_size(width, height);
    (side_w, (side_h - SEARCH_HEIGHT).max(0.0))
}

/// Width and height of the person's area.
fn detail_size(width: f32, height: f32) -> (f32, f32) {
    if stacked(width) {
        (width.max(0.0), (height / 2.0).max(0.0))
    } else {
        ((width * (1.0 - LIST_SHARE)).max(0.0), height.max(0.0))
    }
}

/// Everything, filling `width` x `height`.
pub(in crate::people) fn frame<S, L, D>(areas: Areas<S, L, D>, width: f32, height: f32) -> Div
where
    S: ElementBuilder + 'static,
    L: ElementBuilder + 'static,
    D: ElementBuilder + 'static,
{
    let (list_width, list_height) = list_size(width, height);
    let (_, side_height) = side_size(width, height);
    let (main_width, main_height) = detail_size(width, height);
    let side = div()
        .class("fs-people-side")
        .flex_col()
        .w(list_width)
        .h(side_height)
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
    let root = div().class("fs-people-root");
    let root = if stacked(width) {
        root.flex_col()
    } else {
        root.flex_row()
    };
    root.w(width).h(height).child(side).child(
        div()
            .class("fs-people-main")
            .flex_col()
            .w(main_width)
            .h(main_height)
            .min_h(0.0)
            .child(areas.detail),
    )
}
