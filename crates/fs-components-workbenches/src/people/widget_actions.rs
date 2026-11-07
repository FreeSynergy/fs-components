//! The callbacks of the widget.
//!
//! Pattern: Command — each changes the shared state under a short lock,
//! then asks for a redraw. No lock is held when the redraw is asked for.

use std::sync::Arc;

use super::state::PeopleState;
use super::view::PeopleActions;
use super::widget::{PeopleWidget, locked, rebuild};

impl PeopleWidget {
    pub(super) fn actions(&self) -> PeopleActions {
        let simple = |change: fn(&mut PeopleState)| -> Arc<dyn Fn() + Send + Sync> {
            let state = Arc::clone(&self.state);
            Arc::new(move || {
                change(&mut locked(&state));
                rebuild();
            })
        };
        let (select_s, add_s, typed_s, create_s) = (
            Arc::clone(&self.state),
            Arc::clone(&self.state),
            Arc::clone(&self.state),
            Arc::clone(&self.state),
        );
        PeopleActions {
            select: Arc::new(move |key| {
                locked(&select_s).select(key);
                rebuild();
            }),
            more: simple(PeopleState::load_more),
            create: Arc::new(move || {
                locked(&create_s).begin_create();
                rebuild();
            }),
            edit: simple(PeopleState::begin_edit),
            save: simple(PeopleState::save),
            cancel: simple(PeopleState::cancel_edit),
            keep_editing: simple(PeopleState::keep_editing),
            confirm_discard: simple(PeopleState::confirm_discard),
            ask_delete: simple(PeopleState::ask_delete),
            confirm_delete: simple(PeopleState::confirm_delete),
            keep: simple(PeopleState::keep),
            add_field: Arc::new(move |group| {
                locked(&add_s).add_field(group);
                rebuild();
            }),
            // Typing is remembered but does not redraw: the text field keeps its own cursor.
            typed: Arc::new(move |field, text| locked(&typed_s).set_field(field, text)),
        }
    }
}
