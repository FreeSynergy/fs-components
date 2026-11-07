//! The callbacks of the widget.
//!
//! Pattern: Observer — the callbacks the view calls when the person acts (plain closures, no operation enum); each changes the shared state under a short lock, then
//! asks for a redraw. No lock is held when the redraw is asked for.

use std::sync::Arc;

use fs_components_widgets::ViewRange;

use super::state::CalendarState;
use super::view::{Act, ActOn, CalendarActions};
use super::widget::{CalendarWidget, locked, rebuild};

impl CalendarWidget {
    pub(super) fn actions(&self) -> CalendarActions {
        let simple = |change: fn(&mut CalendarState)| -> Act {
            let state = Arc::clone(&self.state);
            Arc::new(move || {
                change(&mut locked(&state));
                rebuild();
            })
        };
        let on_key = |change: fn(&mut CalendarState, &str)| -> ActOn {
            let state = Arc::clone(&self.state);
            Arc::new(move |key| {
                change(&mut locked(&state), key);
                rebuild();
            })
        };
        let view = |range: ViewRange| -> Act {
            let state = Arc::clone(&self.state);
            Arc::new(move || {
                locked(&state).set_view(range);
                rebuild();
            })
        };
        let (pick_s, typed_s) = (Arc::clone(&self.state), Arc::clone(&self.state));
        CalendarActions {
            today: simple(CalendarState::go_today),
            previous: simple(CalendarState::previous),
            next: simple(CalendarState::next),
            month: view(ViewRange::Month),
            week: view(ViewRange::Week),
            day: view(ViewRange::Day),
            agenda: view(ViewRange::Agenda),
            pick: Arc::new(move |picked| {
                locked(&pick_s).pick(&picked);
                rebuild();
            }),
            create: simple(|s| {
                s.begin_create();
            }),
            edit: simple(CalendarState::begin_edit),
            save: simple(CalendarState::save),
            cancel: simple(CalendarState::cancel_edit),
            keep_editing: simple(CalendarState::keep_editing),
            confirm_discard: simple(CalendarState::confirm_discard),
            back: simple(|s| {
                s.close();
            }),
            ask_delete: simple(CalendarState::ask_delete),
            confirm_delete: simple(CalendarState::confirm_delete),
            keep: simple(CalendarState::keep),
            toggle: on_key(CalendarState::toggle_field),
            // Typing is remembered but does not redraw: the text field keeps its own cursor.
            typed: Arc::new(move |field, text| locked(&typed_s).set_field(field, text)),
        }
    }
}
