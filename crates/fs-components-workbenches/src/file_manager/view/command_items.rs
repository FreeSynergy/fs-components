//! What the command bar holds: one list of items, from which its height and its elements both come.
//!
//! Pattern: Adapter (state to data) — what the source or the variant cannot do is not listed, what is not built yet is
//! listed dimmed with its tooltip saying so (ADR 0066).

use fs_components_widgets::icon::Glyph;

use super::actions::{Act, FileActions, ICON_BUTTON};
use crate::file_manager::context::{FileView, Variant};
use crate::file_manager::state::FilesState;

/// One thing in the bar.
pub(super) enum Item {
    /// An icon button: id, glyph, tooltip, (pressed, usable), action.
    Icon(&'static str, Glyph, String, (bool, bool), Act),
    /// An icon button with a badge: a dot (count 0) or a number, and what a screen reader says of it.
    Marked(&'static str, Glyph, String, Act, (u32, String)),
    /// The "New" button with its menu arrow.
    New(String, bool, Act),
    /// A word button of the menu of "New", or of the trash.
    Word(&'static str, Glyph, String, Act),
    /// The room that pushes what follows to the right.
    Gap,
}

impl Item {
    pub(super) fn width(&self) -> f32 {
        match self {
            Item::Icon(..) | Item::Marked(..) => ICON_BUTTON + 8.0,
            Item::New(word, ..) => 70.0 + word_room(word),
            Item::Word(_, _, word, _) => 44.0 + word_room(word),
            Item::Gap => 0.0,
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn word_room(word: &str) -> f32 {
    word.chars().count() as f32 * 8.0
}

fn locked(word: &str, labels: &crate::file_manager::ChromeLabels) -> String {
    format!("{word} ({})", labels.coming_soon)
}

/// Below this width the less used commands hide behind one button, so the rest stays usable.
pub(super) const TOOLS_BELOW: f32 = 560.0;

// One list of everything the bar holds, in the order it is shown: splitting it would only scatter the order.
#[allow(clippy::too_many_lines)]
pub(super) fn items(state: &FilesState, actions: &FileActions, width: f32) -> Vec<Item> {
    let (labels, caps) = (state.labels(), state.capabilities());
    let chrome = &labels.chrome;
    let mut out = Vec::new();
    let narrow = width < TOOLS_BELOW && state.variant() == Variant::Full;
    let more = !narrow || state.tools_open();
    if narrow {
        out.push(Item::Icon(
            "tools",
            Glyph::More,
            labels.tools.clone(),
            (state.tools_open(), true),
            actions.plain(FilesState::toggle_tools),
        ));
    }
    if state.variant() == Variant::Full && more {
        if state.in_trash() {
            out.push(Item::Icon(
                "leave-trash",
                Glyph::Back,
                labels.leave_trash.clone(),
                (false, true),
                actions.with(|s| s.open_folder(None)),
            ));
            if caps.trash && !state.typing() {
                out.push(Item::Word(
                    "empty-trash",
                    Glyph::Trash,
                    labels.empty_trash.clone(),
                    actions.plain(FilesState::ask_empty_trash),
                ));
            }
        } else {
            if caps.change || caps.upload {
                out.push(Item::New(
                    chrome.new_menu.clone(),
                    state.new_menu_open(),
                    actions.plain(FilesState::toggle_new_menu),
                ));
            }
            if state.new_menu_open() {
                if caps.change {
                    out.push(Item::Word(
                        "new-folder",
                        Glyph::Folder,
                        labels.new_folder.clone(),
                        actions.plain(FilesState::begin_new_folder),
                    ));
                }
                if caps.upload {
                    out.push(Item::Word(
                        "upload",
                        Glyph::Upload,
                        labels.upload.clone(),
                        actions.plain(FilesState::begin_upload),
                    ));
                }
            }
            let selected = state.selected_key().is_some();
            let noop: Act = std::sync::Arc::new(|| {});
            for (id, glyph, word) in [
                ("act-cut", Glyph::Cut, &chrome.cut),
                ("act-copy", Glyph::Copy, &chrome.copy),
                ("act-paste", Glyph::Paste, &chrome.paste),
            ] {
                out.push(Item::Icon(
                    id,
                    glyph,
                    locked(word, chrome),
                    (false, false),
                    noop.clone(),
                ));
            }
            out.push(Item::Icon(
                "act-rename",
                Glyph::Rename,
                labels.rename.clone(),
                (false, selected && caps.change),
                actions.plain(FilesState::begin_rename),
            ));
            out.push(Item::Icon(
                "act-share",
                Glyph::Share,
                locked(&chrome.share, chrome),
                (false, false),
                noop,
            ));
            out.push(Item::Icon(
                "act-delete",
                Glyph::Trash,
                labels.delete.clone(),
                (false, selected && caps.trash),
                actions.plain(FilesState::ask_delete),
            ));
        }
        out.push(Item::Gap);
        if !state.in_trash() {
            if caps.rescan {
                let found = state.report().is_some_and(|r| {
                    !(r.added.is_empty() && r.updated.is_empty() && r.missing.is_empty())
                });
                let act = actions.plain(FilesState::rescan);
                out.push(if found {
                    Item::Marked(
                        "rescan",
                        Glyph::Drive,
                        labels.rescan.clone(),
                        act,
                        (0, chrome.changes_found.clone()),
                    )
                } else {
                    Item::Icon(
                        "rescan",
                        Glyph::Drive,
                        labels.rescan.clone(),
                        (false, true),
                        act,
                    )
                });
            }
            if caps.upload || caps.download {
                let open = state
                    .transfers()
                    .iter()
                    .filter(|t| t.pending.is_some() || t.path.is_some() || t.file.is_some())
                    .count();
                let act = actions.plain(FilesState::open_transfers);
                out.push(if open > 0 {
                    let said = crate::file_manager::FileLabels::fill(
                        &chrome.transfers_open,
                        &open.to_string(),
                    );
                    Item::Marked(
                        "transfers",
                        Glyph::Transfers,
                        labels.transfers.clone(),
                        act,
                        (u32::try_from(open).unwrap_or(u32::MAX), said),
                    )
                } else {
                    Item::Icon(
                        "transfers",
                        Glyph::Transfers,
                        labels.transfers.clone(),
                        (false, true),
                        act,
                    )
                });
            }
        }
        let mut right = |id, glyph, tip: &String, on: bool, act: Act| {
            out.push(Item::Icon(id, glyph, tip.clone(), (on, true), act));
        };
        if caps.search {
            right(
                "filter",
                Glyph::Filter,
                &labels.filter,
                state.filter().is_active(),
                actions.plain(FilesState::open_filter),
            );
        }
        if caps.trash && !state.in_trash() {
            right(
                "trash",
                Glyph::Trash,
                &labels.trash,
                false,
                actions.plain(FilesState::open_trash),
            );
        }
        if caps.sort {
            right(
                "sort",
                Glyph::Sort,
                &chrome.sort,
                false,
                actions.plain(FilesState::next_sort),
            );
        }
    }
    let mut right = |id, glyph, tip: &String, on: bool, act: Act| {
        out.push(Item::Icon(id, glyph, tip.clone(), (on, true), act));
    };
    right(
        "view-list",
        Glyph::ViewList,
        &labels.view_list,
        state.view() == FileView::List,
        actions.with(|s| s.set_view(FileView::List)),
    );
    right(
        "view-grid",
        Glyph::ViewGrid,
        &labels.view_grid,
        state.view() == FileView::Grid,
        actions.with(|s| s.set_view(FileView::Grid)),
    );
    if state.variant() == Variant::Full {
        right(
            "details",
            Glyph::Details,
            &chrome.details,
            state.details_on(),
            actions.plain(FilesState::toggle_details),
        );
    }
    out
}
