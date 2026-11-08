//! The buttons above an item or a form.
//!
//! Pattern: none (pure functions of what is shown) — asking and confirming are two actions in different places: what lies
//! where Delete or Cancel was keeps what is there, and the deed is across the bar, so a second press at
//! the same point can never be it. Save and Cancel do not lie where Edit was.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};

use super::actions::CalendarActions;
use super::toolbar::button;
use crate::calendar::state::CalendarState;

/// The buttons that fit the state.
pub(super) fn bar(state: &CalendarState, name: &str, actions: &CalendarActions, back: bool) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-calendar-bar").flex_row().items_center();
    if let Some(edit) = state.editing() {
        // Across the bar from Edit and Delete, so the second press of a double-click on Edit lands on
        // nothing. The deed of a discard is across from what lies under the pointer after Cancel.
        // Tree order Save, Cancel, spacer, Discard; drawn in reverse (Tab walks Save before Cancel).
        bar = bar.flex_row_reverse();
        bar = bar
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(cancel_or_keep(state, name, actions))
            .child(div().flex_1());
        if edit.discard_asked {
            bar = bar.child(button(
                id("discard"),
                &labels.discard_really,
                ButtonVariant::Destructive,
                &actions.confirm_discard,
            ));
        }
        return bar;
    }
    if back {
        bar = bar.child(button(
            id("back"),
            &labels.back,
            ButtonVariant::Ghost,
            &actions.back,
        ));
    }
    if state.can_edit() {
        bar = bar.child(button(
            id("edit"),
            &labels.edit,
            ButtonVariant::Secondary,
            &actions.edit,
        ));
    }
    if state.can_delete() {
        if state.delete_asked() {
            bar = bar
                .child(button(
                    id("keep"),
                    &labels.delete_keep,
                    ButtonVariant::Secondary,
                    &actions.keep,
                ))
                .child(div().flex_1())
                .child(button(
                    id("really"),
                    &labels.delete_really,
                    ButtonVariant::Destructive,
                    &actions.confirm_delete,
                ));
        } else {
            bar = bar.child(button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.ask_delete,
            ));
        }
    }
    bar
}

/// What lies where Cancel is: after the question it keeps the edit.
fn cancel_or_keep(state: &CalendarState, name: &str, actions: &CalendarActions) -> Div {
    let labels = state.labels();
    if state.editing().is_some_and(|edit| edit.discard_asked) {
        button(
            format!("{name}-keep-editing"),
            &labels.discard_keep,
            ButtonVariant::Secondary,
            &actions.keep_editing,
        )
    } else {
        button(
            format!("{name}-cancel"),
            &labels.cancel,
            ButtonVariant::Secondary,
            &actions.cancel,
        )
    }
}
