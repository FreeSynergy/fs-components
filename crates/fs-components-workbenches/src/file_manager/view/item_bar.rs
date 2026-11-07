//! The buttons of the thing selected.
//!
//! Pattern: Strategy by state — what the source or the state does not allow is not offered. The first
//! press of Delete is here, its answer in the band; once asked, Keep stands where Delete stood.

use blinc_cn::components::button::ButtonVariant;

use super::actions::FileActions;
use super::toolbar::Press;
use crate::file_manager::carry::CarryKind;
use crate::file_manager::context::Variant;
use crate::file_manager::state::FilesState;

/// The presses of the thing selected.
pub(super) fn item_presses(state: &FilesState, actions: &FileActions) -> Vec<Press> {
    let labels = state.labels();
    let caps = state.capabilities();
    let Some(entry) = state.selected_entry() else {
        return Vec::new();
    };
    let folder = entry.kind == crate::file_manager::model::EntryKind::Folder;
    let plain = |name, word: &String, change: fn(&mut FilesState)| -> Press {
        (
            name,
            word.clone(),
            ButtonVariant::Secondary,
            actions.plain(change),
        )
    };
    let mut out = Vec::new();
    if state.variant() == Variant::Picker {
        return out;
    }
    // While a name or path is being typed, what would throw it away is not offered.
    let typing = state.typing();
    if entry.trashed {
        if caps.trash && !typing {
            out.push(plain(
                "restore",
                &labels.restore,
                FilesState::restore_selected,
            ));
            if caps.restore_to {
                let carry = actions.with(|s| s.carry(CarryKind::Restore));
                out.push((
                    "restore-to",
                    labels.restore_to.clone(),
                    ButtonVariant::Secondary,
                    carry,
                ));
            }
        }
    } else {
        if folder {
            out.push(plain("open", &labels.open, FilesState::open_selected));
        }
        if caps.change && !typing {
            out.push(plain("rename", &labels.rename, FilesState::begin_rename));
            let carry = actions.with(|s| s.carry(CarryKind::Move));
            out.push((
                "move",
                labels.move_to.clone(),
                ButtonVariant::Secondary,
                carry,
            ));
            if !folder {
                let carry = actions.with(|s| s.carry(CarryKind::Copy));
                out.push((
                    "copy",
                    labels.copy_to.clone(),
                    ButtonVariant::Secondary,
                    carry,
                ));
            }
        }
        if caps.download && !folder {
            out.push(plain(
                "download",
                &labels.download,
                FilesState::begin_download,
            ));
        }
    }
    if caps.trash && !typing {
        let word = if entry.trashed {
            &labels.purge
        } else if folder {
            &labels.delete_folder
        } else {
            &labels.delete
        };
        // The answer stands in the band; here the first press only asks, again and again.
        let (name, word, variant, act) = if state.confirm_asked().is_some() {
            (
                "keep",
                &labels.keep,
                ButtonVariant::Secondary,
                actions.plain(FilesState::keep),
            )
        } else {
            (
                "delete",
                word,
                ButtonVariant::Ghost,
                actions.plain(FilesState::ask_delete),
            )
        };
        out.push((name, word.clone(), variant, act));
    }
    out
}
