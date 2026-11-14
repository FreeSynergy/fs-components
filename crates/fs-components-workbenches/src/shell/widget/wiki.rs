//! The wiki layouts of the reader widget: the same two live areas (side, main) in another frame.
//!
//! Pattern: Strategy — `build_wiki` is what `build` does for a layout other than the plain one; the state (reader, tree,
//! block editor) is the widget's, this file only draws and wires.

use std::sync::{Arc, PoisonError};

use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;

use super::{ReaderWidget, locked, state};
use crate::reader::Reader;
use crate::view::{BlockEditor, Layout, confluence, notion};

impl ReaderWidget {
    /// The wiki layouts: the same two live areas (side, main) in another frame.
    pub(super) fn build_wiki(&self, width: f32, height: f32) -> blinc_layout::div::Div {
        let (id, reader) = (self.id, &self.reader);
        let (page_v, side_v) = (state(id, "page"), state(id, "side"));
        let actions = self.actions();
        let labels = self.wiki_labels.clone();
        let layout = self.layout;
        let side = {
            let (reader, tree, actions, labels) = (
                Arc::clone(reader),
                Arc::clone(&self.tree),
                actions.clone(),
                labels.clone(),
            );
            stateful::<NoState>()
                .deps([side_v.id()])
                .on_state(move |_| {
                    let reader = locked(&reader);
                    let mut tree = tree.lock().unwrap_or_else(PoisonError::into_inner);
                    if let Some(path) = reader.current_path() {
                        tree.reveal_once(
                            &reader
                                .page_info()
                                .canonical
                                .unwrap_or_else(|| path.to_string()),
                        );
                    }
                    if layout == Layout::Confluence {
                        confluence::side(&reader, &tree, &labels, &actions)
                    } else {
                        notion::side(&reader, &tree, &labels, &actions)
                    }
                })
        };
        let main = {
            let (reader, actions, labels, this) = (
                Arc::clone(reader),
                actions.clone(),
                labels.clone(),
                self.clone(),
            );
            stateful::<NoState>()
                .deps([page_v.id()])
                .on_state(move |_| {
                    let reader = locked(&reader);
                    if layout == Layout::Confluence {
                        let area = this.sync_area(&reader);
                        confluence::main(
                            &reader,
                            &labels,
                            &actions,
                            None,
                            area.as_ref(),
                            (width, height),
                        )
                    } else {
                        this.sync_blocks(&reader);
                        let blocks = this.blocks.lock().unwrap_or_else(PoisonError::into_inner);
                        notion::main(
                            &reader,
                            &labels,
                            &actions,
                            None,
                            blocks.as_ref(),
                            &actions.draft,
                        )
                    }
                })
        };
        let (back, save) = (Arc::clone(&actions.back), Arc::clone(&actions.save));
        let search = self.field.build();
        let ui = if layout == Layout::Confluence {
            confluence::confluence(search, side, main, &labels, width, height)
        } else {
            notion::notion(search, side, main, width, height)
        };
        if self.keys {
            ui.on_key_down(move |e| match e.key_code {
                0x25 if e.alt => back(),
                0x53 if e.ctrl => save(),
                _ => {}
            })
        } else {
            ui
        }
    }

    /// Keep the block editor of the edit in progress (made when an edit starts, dropped when it ends).
    pub(super) fn sync_blocks(&self, reader: &Reader) {
        let mut slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(editing) = reader.editing() else {
            *slot = None;
            return;
        };
        if slot.as_ref().is_none_or(|b| b.edit != editing.id) {
            *slot = Some(BlockEditor::new(
                editing.id,
                &editing.draft,
                &reader.page_title(),
            ));
        }
    }

    /// The text of every block of the Notion-like editor now (empty when no block editor is open): for hosts that drive
    /// the window from outside, and for their tests.
    pub fn block_texts(&self) -> Vec<String> {
        let slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        slot.as_ref()
            .map(BlockEditor::block_texts)
            .unwrap_or_default()
    }

    /// Which block of the Notion-like editor has the keyboard.
    pub fn focused_block(&self) -> Option<usize> {
        let slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        slot.as_ref().and_then(BlockEditor::focused)
    }

    /// Give the keyboard to block `i`, the cursor `column` characters into its first line (a test of a host types
    /// there; nothing happens when there is no such block).
    pub fn focus_block(&self, i: usize, column: usize) {
        let slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(editor) = slot.as_ref().filter(|e| i < e.block_texts().len()) {
            editor.put_cursor(i, column);
        }
    }

    /// Type `text` (no line break) into block `i` of the Notion-like editor, `at` bytes into it, as the keyboard would.
    pub fn type_in_block(&self, i: usize, at: usize, text: &str) {
        let slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(editor) = slot.as_ref().filter(|e| i < e.block_texts().len()) {
            editor.type_in(i, at, text);
        }
    }

    /// Is the block menu of the Notion-like editor open?
    pub fn block_menu_open(&self) -> bool {
        let slot = self.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        slot.as_ref().is_some_and(BlockEditor::menu_open)
    }
}
