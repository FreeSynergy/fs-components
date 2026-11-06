//! The outer layout: the toolbar above, the work area and the task beside each other.
//!
//! Pattern: Template — fixed skeleton; the areas are passed in. Sizes are worked out here because the
//! list must be told its height in pixels. At narrow widths the toolbar takes more lines and the
//! work area stands above the task, so everything stays reachable.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

/// Below this width the work area stands above the task, each with the whole width.
const STACK_BELOW: f32 = 720.0;
/// The part of the width the work area takes.
const WORK_SHARE: f32 = 0.62;

/// The areas of the frame, ready made.
pub(in crate::tasks) struct Areas<T, W, D> {
    /// Search, views, New.
    pub toolbar: T,
    /// The list, or the columns.
    pub work: W,
    /// The task.
    pub detail: D,
}

fn stacked(width: f32) -> bool {
    width < STACK_BELOW
}

/// How high the toolbar is: its parts wrap, and what wraps must still fit.
fn toolbar_height(width: f32) -> f32 {
    if width >= 560.0 {
        60.0
    } else if width >= 300.0 {
        104.0
    } else {
        148.0
    }
}

/// Width and height of the work area inside `width` x `height`.
pub(in crate::tasks) fn work_size(width: f32, height: f32) -> (f32, f32) {
    let rest = (height - toolbar_height(width)).max(0.0);
    if stacked(width) {
        (width.max(0.0), rest / 2.0)
    } else {
        ((width * WORK_SHARE).max(0.0), rest)
    }
}

fn detail_size(width: f32, height: f32) -> (f32, f32) {
    let rest = (height - toolbar_height(width)).max(0.0);
    if stacked(width) {
        (width.max(0.0), rest / 2.0)
    } else {
        ((width * (1.0 - WORK_SHARE)).max(0.0), rest)
    }
}

/// Everything, filling `width` x `height`.
pub(in crate::tasks) fn frame<T, W, D>(areas: Areas<T, W, D>, width: f32, height: f32) -> Div
where
    T: ElementBuilder + 'static,
    W: ElementBuilder + 'static,
    D: ElementBuilder + 'static,
{
    let (work_w, work_h) = work_size(width, height);
    let (main_w, main_h) = detail_size(width, height);
    let body = div().class("fs-tasks-body");
    let body = if stacked(width) {
        body.flex_col()
    } else {
        body.flex_row()
    };
    div()
        .class("fs-tasks-root")
        .flex_col()
        .w(width)
        .h(height)
        .child(
            div()
                .class("fs-tasks-toolbar")
                .w(width)
                .h(toolbar_height(width))
                .child(areas.toolbar),
        )
        .child(
            body.w(width)
                .child(
                    div()
                        .class("fs-tasks-work")
                        .w(work_w)
                        .h(work_h)
                        .min_h(0.0)
                        .overflow_clip()
                        .child(areas.work),
                )
                .child(
                    div()
                        .class("fs-tasks-main")
                        .flex_col()
                        .w(main_w)
                        .h(main_h)
                        .min_h(0.0)
                        .child(areas.detail),
                ),
        )
}
