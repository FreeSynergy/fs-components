//! The editor of the page: a text area with Save and Cancel.
//!
//! Pattern: Adapter over Blinc's text area. The text is the draft kept by the reader; typing
//! reports to it without redrawing, so the area keeps its own cursor.

use blinc_cn::cn;
use blinc_cn::components::textarea::textarea;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::text_area::text_area_state;

use super::actions::Actions;
use crate::reader::Editing;

/// The editor for `editing`.
pub(crate) fn editor(editing: &Editing, actions: &Actions) -> Div {
    let state = text_area_state();
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_value(&editing.draft);
    let draft = actions.draft.clone();
    let area = textarea(&state)
        .w_full()
        .rows(30)
        .on_change(move |value| draft(value));
    let (save, cancel) = (actions.save.clone(), actions.cancel.clone());
    let mut buttons = div()
        .class("fs-rd-bar")
        .child(cn::button("Save").on_click(move |_| save()))
        .child(cn::button("Cancel").on_click(move |_| cancel()));
    if let Some(error) = &editing.error {
        buttons = buttons.child(div().class("fs-rd-failed").child(text(error)));
    }
    div()
        .flex_col()
        .w_full()
        .flex_1()
        .child(buttons)
        .child(div().class("fs-rd-editor").flex_1().w_full().child(area))
}
