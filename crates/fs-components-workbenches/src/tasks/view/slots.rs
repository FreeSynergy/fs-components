//! The input states of one edit.
//!
//! Pattern: Memento — one input state per field for the whole edit, so a redraw (a failed save, a
//! pressed choice) keeps text and cursor. The lock of the slots is let go before an input's own lock is
//! taken. A third copy of what `people` and `calendar` have (see DESIGN.md).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input_data};

#[derive(Default)]
struct Slots {
    edit: u64,
    lines: HashMap<String, SharedTextInputData>,
    areas: HashMap<String, SharedTextAreaState>,
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
pub(in crate::tasks) struct InputSlots(Arc<Mutex<Slots>>);

impl InputSlots {
    /// The state of a one-line `field`, holding `value`.
    pub(super) fn line(&self, edit: u64, field: &str, value: &str) -> SharedTextInputData {
        let data = {
            let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            slots.for_edit(edit);
            Arc::clone(
                slots
                    .lines
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

    /// The state of a many-line `field`; holds `value` from when it is made, and after that what was
    /// typed into it.
    pub(super) fn area(&self, edit: u64, field: &str, value: &str) -> SharedTextAreaState {
        let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        slots.for_edit(edit);
        Arc::clone(slots.areas.entry(field.to_string()).or_insert_with(|| {
            let state = text_area_state();
            state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .set_value(value);
            state
        }))
    }
}
