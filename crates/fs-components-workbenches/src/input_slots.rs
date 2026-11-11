//! The input states of one edit, shared by the workbenches with a form.
//!
//! Pattern: State — one input state per field for the whole edit, so a redraw (a failed save, a
//! pressed choice, a field added) keeps text and cursor. The lock of the slots is let go before an
//! input's own lock is taken. A new `edit` id forgets what the previous edit held.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::widgets::text_area::SharedTextAreaState;
use blinc_layout::widgets::text_input::SharedTextInputData;
use fs_components_primitives::FieldData;

#[derive(Default)]
struct Slots {
    edit: u64,
    lines: HashMap<String, SharedTextInputData>,
    areas: HashMap<String, SharedTextAreaState>,
    /// Gives the signals of the states back when the edit is forgotten.
    keep: FieldData,
}

impl Slots {
    fn for_edit(&mut self, edit: u64) {
        if self.edit != edit {
            *self = Self {
                edit,
                ..Self::default()
            };
        }
    }
}

/// The input states of the edit in progress, tied to that edit by its id.
#[derive(Clone, Default)]
pub(crate) struct InputSlots(Arc<Mutex<Slots>>);

impl InputSlots {
    /// The state of a one-line `field`, holding `value`.
    pub(crate) fn line(&self, edit: u64, field: &str, value: &str) -> SharedTextInputData {
        let data = {
            let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            slots.for_edit(edit);
            let keep = slots.keep.clone();
            Arc::clone(
                slots
                    .lines
                    .entry(field.to_string())
                    .or_insert_with(|| keep.text_input()),
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

    /// The state of a many-line `field`; holds `value` from when it is made, and after that what was
    /// typed into it.
    pub(crate) fn area(&self, edit: u64, field: &str, value: &str) -> SharedTextAreaState {
        let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        slots.for_edit(edit);
        let keep = slots.keep.clone();
        Arc::clone(slots.areas.entry(field.to_string()).or_insert_with(|| {
            let state = keep.text_area();
            state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .set_value(value);
            state
        }))
    }
}
