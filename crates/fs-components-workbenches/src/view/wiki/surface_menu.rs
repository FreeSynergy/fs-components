//! The block menu under the cursor of the surface: opens when the line the cursor is on is "/" and a filter, is kept by Up, Down,
//! Enter and Esc and by the mouse, and writes the chosen type in place of that line.
//!
//! Pattern: Observer — the menu depends on a signal that is counted up when it opens, closes, is filtered or moved.

use std::sync::{Arc, PoisonError};

use blinc_layout::div::div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::{Stateful, stateful};
use blinc_layout::text;
use blinc_layout::widgets::text_area::focus_text_area_deferred;

use super::slash::{Kind, matching, query};
use super::surface::Surface;
use super::surface_text::line_at;

/// The menu that is open: where its line starts, what is typed after "/", which entry is marked.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Open {
    pub at: usize,
    pub filter: String,
    pub selected: usize,
}

const UP: u32 = 38;
const DOWN: u32 = 40;
const ESC: u32 = 27;

impl Surface {
    fn bump_menu(&self) {
        self.menu_signal.update(|v| v.wrapping_add(1));
    }

    fn words(&self) -> fs_components_widgets::SlashLabels {
        self.words
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Open, filter or close the menu for the line the cursor is on.
    pub(super) fn refresh_menu(&self) {
        let (text, cursor) = (self.text(), self.cursor());
        let (start, end) = line_at(&text, cursor);
        let asked = (cursor == end).then(|| query(&text[start..end])).flatten();
        let mut open = self.menu.lock().unwrap_or_else(PoisonError::into_inner);
        let before = open.clone();
        *open = asked.map(|filter| {
            let same = open
                .as_ref()
                .filter(|o| o.at == start && o.filter == filter);
            Open {
                at: start,
                filter: filter.to_owned(),
                selected: same.map_or(0, |o| o.selected),
            }
        });
        let changed = *open != before;
        drop(open);
        if changed {
            self.bump_menu();
        }
    }

    /// The type marked in the open menu, if something matches.
    pub(super) fn marked(&self) -> Option<Kind> {
        let open = self.menu.lock().unwrap_or_else(PoisonError::into_inner);
        let o = open.as_ref()?;
        matching(&o.filter, &self.words()).get(o.selected).copied()
    }

    /// Write `kind` in place of the line of the open menu in `base` (the text without a line break Enter may have added), close
    /// the menu and keep the cursor in what was written.
    pub(super) fn choose(&self, kind: Kind, base: &str) {
        let at = self
            .menu
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .map_or(0, |o| o.at);
        let (start, end) = line_at(base, at);
        let written = kind.written();
        let (text, cursor) = if written.block_after {
            (
                format!("{}{}\n\n{}", &base[..start], written.text, &base[end..]),
                start + written.text.len() + 2,
            )
        } else {
            (
                format!("{}{}{}", &base[..start], written.text, &base[end..]),
                start + written.at,
            )
        };
        self.set(&text, cursor);
        self.remember();
        focus_text_area_deferred(&self.state);
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
                *open = None;
                drop(open);
                focus_text_area_deferred(&self.state);
                self.bump_menu();
                return;
            }
            _ => return,
        }
        drop(open);
        self.bump_menu();
    }

    /// Is a menu open?
    pub fn menu_open(&self) -> bool {
        self.menu
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// The menu under the cursor: nothing unless it is open.
    pub(super) fn menu_view(self: &Arc<Self>) -> Stateful<NoState> {
        let me = Arc::clone(self);
        stateful::<NoState>()
            .deps([self.menu_signal.id()])
            .on_state(move |_| {
                let open = me
                    .menu
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone();
                let Some(o) = open else {
                    return div();
                };
                let words = me.words();
                let kinds = matching(&o.filter, &words);
                let (x, y) = me
                    .state
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .cursor_position_from_visual_lines();
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
                    let surface = Arc::clone(&me);
                    let base = surface.text();
                    let item = div()
                        .class("fs-wk-slash-item")
                        .id(format!("fs-wk-slash-{i}"))
                        .flex_row()
                        .items_center()
                        .on_click(move |_| surface.choose(kind, &base))
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
                // Under the line the cursor is on: the text starts 12 px in, a line is about 22 px high.
                div()
                    .absolute()
                    .left((x + 12.0).max(12.0))
                    .top(y + 34.0)
                    .child(list)
            })
    }
}
