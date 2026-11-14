//! The block menu under a block: opens when the block's text is "/" and a filter, is kept by Up, Down, Enter and Esc and
//! by the mouse, and writes the chosen type into the block.
//!
//! Pattern: Observer — the menu of a block depends on a signal that is counted up when the menu opens, closes, is
//! filtered or moved.

use std::sync::{Arc, PoisonError};

use blinc_layout::div::div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::{Stateful, stateful};
use blinc_layout::text;
use blinc_layout::widgets::text_area::{SharedTextAreaState, focus_text_area_deferred};

use super::block_view::Blocks;
use super::slash::{Kind, matching, query};

/// The menu that is open: for which block, what is typed after "/", which entry is marked.
pub(super) struct Open {
    pub state: SharedTextAreaState,
    pub filter: String,
    pub selected: usize,
}

const UP: u32 = 38;
const DOWN: u32 = 40;
const ESC: u32 = 27;

impl Blocks {
    fn bump_menu(&self) {
        self.menu_signal.update(|v| v.wrapping_add(1));
    }

    fn words(&self) -> fs_components_widgets::SlashLabels {
        self.words
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Open, filter or close the menu of `state` after its text changed to `now`.
    pub(super) fn refresh_menu(&self, state: &SharedTextAreaState, now: &str) {
        let mut open = self.menu.lock().unwrap_or_else(PoisonError::into_inner);
        let before = open
            .as_ref()
            .map(|o| (Arc::as_ptr(&o.state), o.filter.clone(), o.selected));
        *open = match query(now) {
            Some(filter) => {
                let same = open.as_ref().filter(|o| Arc::ptr_eq(&o.state, state));
                let selected = same.map_or(0, |o| if o.filter == filter { o.selected } else { 0 });
                Some(Open {
                    state: Arc::clone(state),
                    filter: filter.to_owned(),
                    selected,
                })
            }
            None => open.take().filter(|o| !Arc::ptr_eq(&o.state, state)),
        };
        let after = open
            .as_ref()
            .map(|o| (Arc::as_ptr(&o.state), o.filter.clone(), o.selected));
        drop(open);
        if before != after {
            self.bump_menu();
        }
    }

    /// The type marked in the menu of `state`, if the menu is open there and something matches.
    pub(super) fn marked(&self, state: &SharedTextAreaState) -> Option<Kind> {
        let open = self.menu.lock().unwrap_or_else(PoisonError::into_inner);
        let o = open.as_ref().filter(|o| Arc::ptr_eq(&o.state, state))?;
        matching(&o.filter, &self.words()).get(o.selected).copied()
    }

    /// Write `kind` into the block of `state`, close the menu, and keep the keyboard there.
    pub(super) fn choose(&self, state: &SharedTextAreaState, kind: Kind) {
        *self.menu.lock().unwrap_or_else(PoisonError::into_inner) = None;
        let written = kind.written();
        super::block_text::set_text(state, &written.text, Some(written.at));
        self.remember(state);
        if written.block_after {
            self.insert_after_state(state, "");
        } else {
            focus_text_area_deferred(state);
        }
        self.bump_menu();
        self.announce();
    }

    /// Up, Down and Esc while a menu is open.
    pub(super) fn menu_key(&self, code: u32) {
        let mut open = self.menu.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(o) = open.as_mut() else { return };
        let count = matching(&o.filter, &self.words()).len().max(1);
        match code {
            UP => o.selected = (o.selected + count - 1) % count,
            DOWN => o.selected = (o.selected + 1) % count,
            ESC => {
                let state = Arc::clone(&o.state);
                *open = None;
                drop(open);
                focus_text_area_deferred(&state);
                self.bump_menu();
                return;
            }
            _ => return,
        }
        drop(open);
        self.bump_menu();
    }

    /// The menu under the block of `state`: empty unless it is open for this block.
    pub(super) fn menu_view(self: &Arc<Self>, state: &SharedTextAreaState) -> Stateful<NoState> {
        let (blocks, state) = (Arc::clone(self), Arc::clone(state));
        stateful::<NoState>()
            .deps([self.menu_signal.id()])
            .on_state(move |_| {
                let open = blocks.menu.lock().unwrap_or_else(PoisonError::into_inner);
                let Some(o) = open.as_ref().filter(|o| Arc::ptr_eq(&o.state, &state)) else {
                    return div();
                };
                let words = blocks.words();
                let kinds = matching(&o.filter, &words);
                let mut list = div()
                    .class("fs-wk-slash")
                    .flex_col()
                    .w(280.0)
                    .id("fs-wk-slash");
                if kinds.is_empty() {
                    list = list.child(
                        div()
                            .class("fs-wk-slash-item")
                            .child(text(&words.none).no_cursor()),
                    );
                }
                for (i, kind) in kinds.into_iter().enumerate() {
                    let (blocks, state) = (Arc::clone(&blocks), Arc::clone(&state));
                    let item = div()
                        .class("fs-wk-slash-item")
                        .id(format!("fs-wk-slash-{i}"))
                        .flex_row()
                        .items_center()
                        .on_click(move |_| blocks.choose(&state, kind))
                        .child(
                            div()
                                .class("fs-wk-slash-mark")
                                .child(text(kind.marker()).no_cursor()),
                        )
                        .child(text(kind.label(&words)).no_cursor());
                    list = list.child(if i == o.selected {
                        item.class("fs-wk-slash-on")
                    } else {
                        item
                    });
                }
                div().class("fs-wk-slash-place").child(list)
            })
    }

    /// Is a menu open (for a test of the host)?
    pub fn menu_open(&self) -> bool {
        self.menu
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }
}
