//! The outer layout: bars above, the tree, the list and the side area beside each other, the band of
//! questions below.
//!
//! Pattern: Adapter (state to elements) — fixed skeleton; the areas are passed in. Sizes are worked out here because the
//! list must be told its height in pixels. Questions stand at the bottom so nothing above them moves
//! when one appears: the second act of something destructive can never be under the first press. At
//! narrow widths the tree gives way to the breadcrumb, and the side area takes the place of the list
//! while it is open.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

use super::band_size::band_height;
use super::commands::commands_height;
use super::nav::NAV_HEIGHT;
use super::tabs::TABS_HEIGHT;
use crate::file_manager::state::FilesState;

/// Below this width one pane is shown at a time.
pub(in crate::file_manager) const ONE_PANE_BELOW: f32 = 720.0;
/// Below this width the tree gives way to the breadcrumb.
const TREE_BELOW: f32 = 960.0;
/// Width of the tree.
const TREE_WIDTH: f32 = 220.0;
/// The part of what is left that the list takes.
const WORK_SHARE: f32 = 0.6;
/// Height of the line with the breadcrumb.
/// About how wide a character of a message is.
const CHAR_WIDTH: f32 = 7.5;
/// Height of a line of a message.
const LINE_HEIGHT: f32 = 22.0;
/// The body never gets less than this, so the bars above and the band below cannot overlap.
pub(super) const STATUS_HEIGHT: f32 = 28.0;
const LEAST_BODY: f32 = 96.0;

/// What is worked out.
#[derive(Clone, Copy, Debug)]
pub(in crate::file_manager) struct Sizes {
    pub width: f32,
    pub bar: f32,
    pub commands: f32,
    pub tabs: f32,
    pub notice: f32,
    pub body: f32,
    pub tree: f32,
    pub work: f32,
    pub side: f32,
}

/// How high a text of `chars` characters is in `width`.
#[allow(clippy::cast_precision_loss)]
pub(super) fn text_height(chars: usize, width: f32) -> f32 {
    let per_line = ((width - 32.0).max(40.0) / CHAR_WIDTH).floor().max(1.0);
    (chars as f32 / per_line).ceil().max(1.0) * LINE_HEIGHT
}

/// The sizes for `state` in `width` x `height`.
pub(in crate::file_manager) fn sizes(state: &FilesState, width: f32, height: f32) -> Sizes {
    let bar = NAV_HEIGHT;
    let commands = commands_height(state, width);
    let tabs = if state.variant() == crate::file_manager::context::Variant::Picker {
        0.0
    } else {
        TABS_HEIGHT
    };
    let notice = state
        .notice()
        .map_or(0.0, |n| text_height(n.text().chars().count(), width) + 16.0);
    let band = band_height(state, width);
    let status = if state.variant() == crate::file_manager::context::Variant::Picker {
        0.0
    } else {
        STATUS_HEIGHT
    };
    let body = (height - tabs - bar - commands - notice - band - status).max(LEAST_BODY);
    let one_pane = width < ONE_PANE_BELOW;
    let picker = state.variant() == crate::file_manager::context::Variant::Picker;
    let tree = if width < TREE_BELOW || picker {
        0.0
    } else {
        TREE_WIDTH
    };
    let rest = width - tree;
    let (work, side) = if picker {
        (width, 0.0)
    } else if one_pane {
        if state.detail_open() {
            (0.0, width)
        } else {
            (width, 0.0)
        }
    } else if (state.selected_key().is_none() || !state.details_on())
        && matches!(state.side(), super::super::side::Side::Item)
    {
        // nothing to show beside the list: the list takes the room
        (rest, 0.0)
    } else {
        (rest * WORK_SHARE, rest * (1.0 - WORK_SHARE))
    };
    Sizes {
        width,
        bar,
        commands,
        tabs,
        notice,
        body,
        tree,
        work,
        side,
    }
}

/// Everything. The areas are ready made; one with no room is not added.
#[allow(clippy::too_many_lines)] // one column of areas, each with its box: splitting scatters the order
pub(in crate::file_manager) fn frame<A, B, C, D, E, F, G, H, I>(
    (sizes, height): (Sizes, f32),
    (tabs, bar, crumbs, notice): (Option<I>, A, B, Option<C>),
    (tree, work, side): (D, E, F),
    (status, band): (Option<H>, Option<G>),
) -> Div
where
    A: ElementBuilder + 'static,
    B: ElementBuilder + 'static,
    C: ElementBuilder + 'static,
    D: ElementBuilder + 'static,
    E: ElementBuilder + 'static,
    F: ElementBuilder + 'static,
    G: ElementBuilder + 'static,
    H: ElementBuilder + 'static,
    I: ElementBuilder + 'static,
{
    let Sizes { width, body, .. } = sizes;
    let mut root = div()
        .class("fs-files-root")
        .flex_col()
        .w(width)
        .h(height)
        .overflow_clip();
    if let Some(tabs) = tabs {
        root = root.child(
            div()
                .class("fs-files-tabs-row")
                .flex_shrink_0()
                .w(width)
                .h(sizes.tabs)
                .child(tabs),
        );
    }
    let mut root = root
        .child(
            div()
                .class("fs-files-toolbar")
                .flex_shrink_0()
                .w(width)
                .h(sizes.bar)
                .child(bar),
        )
        .child(
            div()
                .class("fs-files-crumbs")
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .w(width)
                .h(sizes.commands)
                .overflow_clip()
                .child(crumbs),
        );
    if let Some(notice) = notice {
        root = root.child(
            div()
                .flex_shrink_0()
                .w(width)
                .h(sizes.notice)
                .overflow_clip()
                .child(notice),
        );
    }
    let mut middle = div()
        .class("fs-files-body")
        .flex_row()
        .flex_shrink_0()
        .w(width)
        .h(body);
    if sizes.tree > 0.0 {
        middle = middle.child(
            div()
                .class("fs-files-tree")
                .flex_shrink_0()
                .w(sizes.tree)
                .h(body)
                .overflow_clip()
                .child(tree),
        );
    }
    if sizes.work > 0.0 {
        middle = middle.child(
            div()
                .class("fs-files-work")
                .flex_shrink_0()
                .w(sizes.work)
                .h(body)
                .overflow_clip()
                .child(work),
        );
    }
    if sizes.side > 0.0 {
        middle = middle.child(
            div()
                .class("fs-files-side")
                .flex_shrink_0()
                .w(sizes.side)
                .h(body)
                .overflow_clip()
                .child(side),
        );
    }
    root = root.child(middle);
    if let Some(status) = status {
        root = root.child(
            div()
                .class("fs-files-status")
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .w(width)
                .h(STATUS_HEIGHT)
                .overflow_clip()
                .child(status),
        );
    }
    if let Some(band) = band {
        root = root.child(band); // the band is its own box, of the height `band_height` says (see `band`)
    }
    root
}
