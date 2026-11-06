//! The form of an edit: one text field per value of the draft.
//!
//! Pattern: Adapter over `blinc_cn`'s input + Memento — [`InputSlots`] keeps one input state per
//! field for the whole edit, so a redraw (a failed save, a field added) keeps text and cursor.
//! Typing reports to the draft and does not redraw.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_cn::components::button::ButtonVariant;
use blinc_cn::components::input::input;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input_data};

use super::actions::PeopleActions;
use super::detail::button;
use crate::people::edit::PersonEdit;
use crate::people::labels::PeopleLabels;

/// The input states of the edit in progress, tied to that edit by its id.
#[derive(Clone, Default)]
pub(in crate::people) struct InputSlots(Arc<Mutex<(u64, HashMap<String, SharedTextInputData>)>>);

impl InputSlots {
    /// The input state of `field` in edit `edit`, holding `value`. The lock of the slots is let go
    /// before the input's own lock is taken.
    fn slot(&self, edit: u64, field: &str, value: &str) -> SharedTextInputData {
        let data = {
            let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            if slots.0 != edit {
                *slots = (edit, HashMap::new());
            }
            Arc::clone(
                slots
                    .1
                    .entry(field.to_string())
                    .or_insert_with(text_input_data),
            )
        };
        {
            let mut held = data.lock().unwrap_or_else(PoisonError::into_inner);
            if held.value != value {
                held.value = value.to_string();
                held.cursor = value.chars().count();
                held.selection_start = None;
            }
        }
        data
    }
}

/// The form for `edit`, added to `page`.
pub(super) fn form(
    edit: &PersonEdit,
    labels: &PeopleLabels,
    name: &str,
    actions: &PeopleActions,
    slots: &InputSlots,
    mut page: Div,
) -> Div {
    if let Some(error) = &edit.error {
        page = page.child(div().class("fs-people-failed").child(text(error)));
    }
    for group in &edit.draft.groups {
        let mut section = div().class("fs-people-section").flex_col().w_full();
        for field in &group.fields {
            let data = slots.slot(edit.id, &field.key, &field.value);
            let (typed, key) = (Arc::clone(&actions.typed), field.key.clone());
            section = section.child(
                div().class("fs-people-input").w_full().child(
                    input(&data)
                        .label(field.label.clone())
                        .w_full()
                        .on_change(move |value| typed(&key, value)),
                ),
            );
        }
        if group.can_add {
            let (add, key) = (Arc::clone(&actions.add_field), group.key.clone());
            let act: super::actions::Act = Arc::new(move || add(&key));
            section = section.child(button(
                format!("{name}-add-{}", group.key),
                &PeopleLabels::fill(&labels.add_field, &group.label),
                ButtonVariant::Ghost,
                &act,
            ));
        }
        page = page.child(section);
    }
    page
}
