//! The buttons above a place or a form.
//!
//! Pattern: Strategy by state — asking and confirming are two actions in different places: what lies
//! where Delete or Cancel was keeps what is there, and the deed is across the bar, so a second press at
//! the same point can never be it. Save and Cancel do not lie where Edit was.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};

use super::actions::MapActions;
use super::toolbar::button;
use crate::maps::state::MapState;

/// The buttons that fit the state.
pub(super) fn bar(state: &MapState, name: &str, actions: &MapActions, back: bool) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-map-bar").flex_row().places_center();
    if let Some(edit) = state.editing() {
        // Across the bar from Edit and Delete, so the second press of a double-click on Edit lands on
        // nothing. The deed of a discard is across from what lies under the pointer after Cancel.
        if edit.discard_asked {
            bar = bar.child(button(
                id("discard"),
                &labels.discard_really,
                ButtonVariant::Destructive,
                &actions.confirm_discard,
            ));
        }
        return bar
            .child(div().flex_1())
            .child(cancel_or_keep(state, name, actions))
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ));
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
fn cancel_or_keep(state: &MapState, name: &str, actions: &MapActions) -> Div {
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
