//! The form of an edit: one text field per value of the draft.
//!
//! Pattern: Adapter over `blinc_cn`'s input. [`InputSlots`] keeps one input state per field for the
//! whole edit. Typing reports to the draft and does not redraw.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_cn::components::input::{InputSize, input};
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::PeopleActions;
use super::detail::{button, with_marks};
use crate::input_slots::InputSlots;
use crate::people::edit::PersonEdit;
use crate::people::labels::PeopleLabels;

/// The form for `edit`, added to `page`.
pub(super) fn form(
    edit: &PersonEdit,
    labels: &PeopleLabels,
    name: &str,
    actions: &PeopleActions,
    slots: &InputSlots,
    mut page: Div,
) -> Div {
    if !edit.draft.marks.is_empty() {
        page = page.child(with_marks(
            div().class("fs-people-head").flex_row(),
            &edit.draft.marks,
        ));
    }
    if let Some(error) = &edit.error {
        page = page.child(div().class("fs-people-failed").child(text(error)));
    }
    for group in &edit.draft.groups {
        let mut section = div().class("fs-people-section").flex_col().w_full();
        for field in &group.fields {
            let data = slots.line(edit.id, &field.key, &field.value);
            let (typed, key) = (Arc::clone(&actions.typed), field.key.clone());
            section = section.child(
                div().class("fs-people-input").w_full().child(
                    input(&data)
                        .size(InputSize::Small)
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
