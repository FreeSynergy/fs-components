//! The callbacks of the widget.
//!
//! Pattern: Observer — the callbacks the view calls when the person acts (plain closures, no operation enum); each changes the shared state under a short lock, then
//! brings the status view up to date and asks for a redraw. No lock is held for either. What the
//! status view's list reports comes back here as a value: a move is decided by the source, and a move
//! that did not happen puts the card back where the source has it.

use std::sync::Arc;

use fs_components_widgets::sortable_list::{Moved, Outcome, Refusal};

use super::context::TaskView;
use super::state::TasksState;
use super::view::TaskActions;
use super::widget::{TasksWidget, bump_list, locked, rebuild};

impl TasksWidget {
    /// Change the state, take the status view along, draw again.
    fn apply(&self, change: impl FnOnce(&mut TasksState)) {
        let snapshot = {
            let mut held = locked(&self.state);
            change(&mut held);
            held.clone()
        };
        self.board.sync(&snapshot);
        rebuild();
    }

    pub(super) fn actions(&self) -> TaskActions {
        let plain = |change: fn(&mut TasksState)| -> Arc<dyn Fn() + Send + Sync> {
            let this = self.clone();
            Arc::new(move || this.apply(change))
        };
        let on_key = |change: fn(&mut TasksState, &str)| -> Arc<dyn Fn(&str) + Send + Sync> {
            let this = self.clone();
            Arc::new(move |key| this.apply(|s| change(s, key)))
        };
        let (typed, cursor) = (Arc::clone(&self.state), self.clone());
        TaskActions {
            select: on_key(|s, key| {
                s.select(key);
            }),
            walk: {
                let (state, id) = (Arc::clone(&self.state), self.id);
                // A live area: the list keeps the keyboard.
                Arc::new(move |key| {
                    locked(&state).walk(key);
                    bump_list(id);
                })
            },
            more: plain(TasksState::load_more),
            create: plain(|s| {
                s.begin_create();
            }),
            edit: plain(TasksState::begin_edit),
            save: plain(TasksState::save),
            cancel: plain(TasksState::cancel_edit),
            keep_editing: plain(TasksState::keep_editing),
            confirm_discard: plain(TasksState::confirm_discard),
            ask_delete: plain(TasksState::ask_delete),
            confirm_delete: plain(TasksState::confirm_delete),
            keep: plain(TasksState::keep),
            view_list: plain(|s| s.set_view(TaskView::List)),
            view_status: plain(|s| s.set_view(TaskView::Status)),
            open_cursor: Arc::new(move || {
                if let Some(key) = cursor.board.list.cursor_key() {
                    cursor.apply(|s| {
                        s.select(&key);
                    });
                }
            }),
            move_selected: on_key(|s, column| {
                if let Some(key) = s.selected().map(str::to_string) {
                    s.move_task(&key, column);
                }
            }),
            choose: {
                let this = self.clone();
                Arc::new(move |field, value| this.apply(|s| s.set_field(field, value)))
            },
            // Typing is remembered but does not redraw: the text field keeps its own cursor.
            typed: Arc::new(move |field, text| locked(&typed).set_field(field, text)),
        }
    }

    /// What the status view's list reports. Runs with no lock of the list held.
    pub(super) fn listen_to_board(&self) {
        let this = self.clone();
        // The handler stays with the list for as long as the widget lives.
        let _ =
            self.board
                .list
                .clone()
                .on_outcome(Arc::new(move |outcome: &Outcome| match outcome {
                    Outcome::Moved(moved) => this.landed(moved),
                    Outcome::Chosen(chosen) => this.apply(|s| {
                        s.select(&chosen.item);
                    }),
                    Outcome::Refused(Refusal::Locked) => {
                        if let Some(key) = this.board.refused_card() {
                            this.apply(|s| s.refuse_move(&key));
                        }
                    }
                    Outcome::Unchanged | Outcome::Cancelled => this.apply(|_| {}),
                    _ => {}
                }));
    }

    fn landed(&self, moved: &Moved) {
        if moved.from.lane == moved.to.lane {
            self.apply(TasksState::refuse_order);
        } else {
            self.apply(|s| {
                s.move_task(&moved.item, &moved.to.lane);
            });
        }
    }
}
