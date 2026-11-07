//! The area beside the list: the thing selected, or what else was asked for.
//!
//! Pattern: none (pure functions of what is shown). Marks and buttons stand outside the scroller, and the preview's scroller
//! starts at the top for each build. The first press of something destructive is here; its answer is
//! in the band.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::{FileActions, button};
use super::frame::ONE_PANE_BELOW;
use super::item_bar::item_presses;
use super::side_more::{filter_panel, report_panel, transfers_panel};
use super::toolbar::buttons_height;
use crate::file_manager::context::Variant;
use crate::file_manager::side::Side;
use crate::file_manager::state::FilesState;
use crate::input_slots::InputSlots;

/// Height of the bar that leads back to the list at narrow widths.
const BACK_HEIGHT: f32 = 48.0;

/// The area, `width` x `height` (the whole window's width decides whether Back is needed).
pub(in crate::file_manager) fn side(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    slots: &InputSlots,
    (width, height, whole): (f32, f32, f32),
) -> Div {
    let mut area = div()
        .class("fs-files-side-inner")
        .flex_col()
        .w(width)
        .h(height);
    let mut left = height;
    if whole < ONE_PANE_BELOW && state.variant() == Variant::Full {
        left -= BACK_HEIGHT;
        area = area.child(
            div()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .h(BACK_HEIGHT)
                .child(button(
                    format!("{name}-close-detail"),
                    &state.labels().back_to_list,
                    ButtonVariant::Secondary,
                    &actions.plain(FilesState::close_detail),
                )),
        );
    }
    let left = left.max(0.0);
    match state.side() {
        Side::Transfers => area.child(transfers_panel(state, actions, width, left)),
        Side::Filter => area.child(filter_panel(state, name, actions, slots, width)),
        Side::Report => area.child(report_panel(state, name, width, left)),
        _ => area.child(item_panel(state, name, actions, width, left)),
    }
}

fn item_panel(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    width: f32,
    height: f32,
) -> Div {
    let Some(preview) = state.preview() else {
        return div()
            .id(format!("{name}-nothing-selected"))
            .class("fs-files-empty")
            .child(text(&state.labels().nothing_selected));
    };
    let presses = item_presses(state, actions);
    let words: Vec<&str> = presses.iter().map(|p| p.1.as_str()).collect();
    let bar_height = buttons_height(&words, 0.0, width);
    let mut bar = div()
        .class("fs-files-item-bar")
        .flex_row()
        .flex_wrap()
        .flex_shrink_0()
        .w(width)
        .h(bar_height);
    for (what, word, variant, act) in &presses {
        bar = bar.child(button(format!("{name}-{what}"), word, *variant, act));
    }
    let mut column = div().flex_col().w(width).h(height).child(bar);
    let mut left = height - bar_height;
    if state.preview_held_back() {
        left -= BACK_HEIGHT;
        column = column.child(
            div()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .h(BACK_HEIGHT)
                .child(button(
                    format!("{name}-show-preview"),
                    &state.labels().show_preview,
                    ButtonVariant::Secondary,
                    &actions.plain(FilesState::release_preview),
                )),
        );
    }
    column.child(
        div()
            .class("fs-files-preview-room")
            .w(width)
            .child(preview.build(
                &format!("{name}-preview"),
                width - 24.0,
                (left - 24.0).max(0.0),
            )),
    )
}
