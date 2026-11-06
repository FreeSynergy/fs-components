//! The editor of the page: a text area with Save and Cancel.
//!
//! Pattern: Adapter over Blinc's text area. The text is the draft kept by the reader; typing
//! reports to it without redrawing, so the area keeps its own cursor.

use blinc_cn::cn;
use blinc_cn::components::textarea::textarea;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};

use super::actions::Actions;
use crate::labels::Labels;
use crate::reader::Editing;

/// The editor for `editing`.
pub(crate) fn editor(
    editing: &Editing,
    labels: &Labels,
    actions: &Actions,
    area: Option<&SharedTextAreaState>,
    rows: usize,
) -> Div {
    // The shell keeps one area for the whole edit, so a redraw (failed save, resize) keeps the
    // cursor and the focus; without one (a picture) a fresh area is made.
    let state = area.cloned().unwrap_or_else(|| new_area(&editing.draft));
    let draft = actions.draft.clone();
    let area = textarea(&state)
        .w_full()
        .rows(rows)
        .on_change(move |value| draft(value));
    let (save, cancel) = (actions.save.clone(), actions.cancel.clone());
    let mut buttons = div()
        .class("fs-rd-bar")
        .child(cn::button(labels.save.clone()).on_click(move |_| save()))
        .child(cn::button(labels.cancel.clone()).on_click(move |_| cancel()));
    if let Some(note) = &editing.note {
        buttons = buttons.child(div().class("fs-rd-notice").child(text(note)));
    }
    if let Some(error) = &editing.error {
        buttons = buttons.child(div().class("fs-rd-failed").child(text(error)));
    }
    div()
        .flex_col()
        .w_full()
        .flex_1()
        .min_h(0.0)
        .child(buttons)
        .child(
            div()
                .class("fs-rd-editor")
                .flex_1()
                .min_h(0.0)
                .w_full()
                .child(area),
        )
}

/// A text area holding `text`.
pub(crate) fn new_area(text: &str) -> SharedTextAreaState {
    let state = text_area_state();
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_value(text);
    state
}

/// How many text rows fit into a window `height` pixels high (bar, buttons and padding take
/// about 190 px; a row is about 20 px), at least 4.
pub(crate) fn editor_rows(height: f32) -> usize {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rows = ((height - 190.0) / 20.0).max(0.0) as usize;
    rows.clamp(4, 80)
}
