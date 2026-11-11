//! Search, views, order, tools.
//!
//! Pattern: Adapter (state to elements) — what the source or the variant cannot do is not offered, and in the
//! trash other things are offered. At narrow widths the tools hide behind one button, so the rest stays
//! usable; their height is worked out from their words, since what wraps must still fit.

use blinc_cn::components::button::ButtonVariant;

use super::actions::{Act, FileActions};
use crate::file_manager::ask::Ask;
use crate::file_manager::context::{FileView, Variant};
use crate::file_manager::model::SortKey;
use crate::file_manager::state::FilesState;

/// Below this width the tools hide behind a button.
pub(super) const TOOLS_BELOW: f32 = 560.0;
/// Height of a line of buttons.
const LINE: f32 = 40.0;
/// Padding of a bar.
pub(super) const PADDING: f32 = 24.0;
/// About how wide a character of a button is, and what a button has besides its words.
pub(super) const CHAR_WIDTH: f32 = 8.0;
pub(super) const BUTTON_ROOM: f32 = 40.0;

/// One button of a bar: its name, its words, how it looks, what it does.
pub(super) type Press = (&'static str, String, ButtonVariant, Act);

/// How high a bar with buttons of these words is in `width`.
#[allow(clippy::cast_precision_loss)]
pub(in crate::file_manager) fn buttons_height(words: &[&str], first: f32, width: f32) -> f32 {
    let room = width - PADDING;
    let (mut lines, mut used) = (1.0_f32, first);
    for word in words {
        let wide = word.chars().count() as f32 * CHAR_WIDTH + BUTTON_ROOM;
        if used > 0.0 && used + wide > room {
            lines += 1.0;
            used = 0.0;
        }
        used += wide + 8.0;
    }
    lines * LINE + PADDING
}

fn sort_word(state: &FilesState, key: SortKey) -> &str {
    let labels = state.labels();
    match key {
        SortKey::Name => &labels.sort_name,
        SortKey::Modified => &labels.sort_modified,
        SortKey::Size => &labels.sort_size,
        _ => &labels.sort_type,
    }
}

fn pressed(on: bool) -> ButtonVariant {
    if on {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Secondary
    }
}

pub(super) fn main_presses(state: &FilesState, actions: &FileActions, narrow: bool) -> Vec<Press> {
    let labels = state.labels();
    let mut out = Vec::new();
    if state.capabilities().search {
        let word = if state.filter().is_active() {
            labels.filter_on.clone()
        } else {
            labels.filter.clone()
        };
        out.push((
            "filter",
            word,
            pressed(state.filter().is_active()),
            actions.plain(FilesState::open_filter),
        ));
    }
    out.push((
        "view-list",
        labels.view_list.clone(),
        pressed(state.view() == FileView::List),
        actions.with(|s| s.set_view(FileView::List)),
    ));
    out.push((
        "view-grid",
        labels.view_grid.clone(),
        pressed(state.view() == FileView::Grid),
        actions.with(|s| s.set_view(FileView::Grid)),
    ));
    if state.capabilities().trash {
        if state.in_trash() {
            out.push((
                "leave-trash",
                labels.leave_trash.clone(),
                ButtonVariant::Secondary,
                actions.with(|s| s.open_folder(None)),
            ));
        } else {
            out.push((
                "trash",
                labels.trash.clone(),
                ButtonVariant::Secondary,
                actions.plain(FilesState::open_trash),
            ));
        }
    }
    if narrow {
        out.push((
            "tools",
            labels.tools.clone(),
            pressed(state.tools_open()),
            actions.plain(FilesState::toggle_tools),
        ));
    }
    out
}

pub(super) fn tool_presses(state: &FilesState, actions: &FileActions) -> Vec<Press> {
    let labels = state.labels();
    let caps = state.capabilities();
    let mut out: Vec<Press> = Vec::new();
    if caps.sort {
        for (name, key) in [
            ("sort-name", SortKey::Name),
            ("sort-modified", SortKey::Modified),
            ("sort-size", SortKey::Size),
            ("sort-type", SortKey::Type),
        ] {
            out.push((
                name,
                sort_word(state, key).to_string(),
                if state.sort().key == key {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                },
                actions.with(move |s| s.set_sort(key)),
            ));
        }
        if state.can_toggle_direction() {
            let word = if state.sort().descending {
                &labels.descending
            } else {
                &labels.ascending
            };
            out.push((
                "direction",
                word.clone(),
                ButtonVariant::Ghost,
                actions.plain(FilesState::toggle_direction),
            ));
        }
    }
    if state.in_trash() {
        if caps.trash && !state.typing() {
            out.push((
                "empty-trash",
                labels.empty_trash.clone(),
                ButtonVariant::Ghost,
                actions.plain(FilesState::ask_empty_trash),
            ));
        }
        return out;
    }
    let mut add = |on: bool, name, word: &String, change: fn(&mut FilesState)| {
        if on {
            out.push((
                name,
                word.clone(),
                ButtonVariant::Secondary,
                actions.plain(change),
            ));
        }
    };
    add(
        caps.change,
        "new-folder",
        &labels.new_folder,
        FilesState::begin_new_folder,
    );
    add(
        caps.upload,
        "upload",
        &labels.upload,
        FilesState::begin_upload,
    );
    add(caps.rescan, "rescan", &labels.rescan, FilesState::rescan);
    add(
        caps.upload || caps.download,
        "transfers",
        &labels.transfers,
        FilesState::open_transfers,
    );
    out
}

pub(super) fn shows_tools(state: &FilesState, width: f32) -> bool {
    // At narrow widths a question takes the room of the tools: it must be seen.
    state.variant() == Variant::Full
        && (width >= TOOLS_BELOW || (state.tools_open() && *state.ask() == Ask::Nothing))
}
