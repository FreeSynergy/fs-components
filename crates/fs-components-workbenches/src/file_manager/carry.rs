//! Carrying something to another folder: moved, copied or brought back from the trash.
//!
//! Pattern: State — what is carried waits while the person goes to the folder, and is put down there
//! by another press, as the [`Command`] it becomes. The source must allow that kind of change.

use super::ask::Ask;
use super::command::Command;
use super::model::{EntryKind, FileEntry, NameRule};
use super::state::FilesState;

/// What is being carried.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarryKind {
    /// It moves.
    Move,
    /// A copy goes.
    Copy,
    /// It comes back from the trash.
    Restore,
}

/// Something on its way to a folder the person is looking for.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carrying {
    /// How.
    pub kind: CarryKind,
    /// What.
    pub entry: FileEntry,
}

impl FilesState {
    /// Start carrying what is selected. A folder cannot be copied: the contract has no such call.
    pub fn carry(&mut self, kind: CarryKind) {
        let Some(entry) = self.selected_entry().cloned() else {
            return;
        };
        if !self.may_carry(kind) || self.typing() {
            return;
        }
        if kind == CarryKind::Copy && entry.kind == EntryKind::Folder {
            return;
        }
        self.notice = None;
        self.ask = Ask::Nothing;
        self.carrying = Some(Carrying { kind, entry });
        if kind == CarryKind::Restore {
            self.open_folder(None);
        }
    }

    /// Does the source allow this kind of carrying? Moving and copying change things; bringing
    /// back to another place needs the trash and its restore-to.
    fn may_carry(&self, kind: CarryKind) -> bool {
        let caps = &self.capabilities;
        match kind {
            CarryKind::Move | CarryKind::Copy => caps.change,
            CarryKind::Restore => caps.trash && caps.restore_to,
        }
    }

    /// Can what is carried be put down here?
    pub fn can_put_down(&self) -> bool {
        self.carrying
            .as_ref()
            .is_some_and(|c| self.may_carry(c.kind))
            && !self.in_trash()
            && !self.typing()
    }

    /// Put what is carried down in the current folder.
    pub fn put_down(&mut self) {
        if !self.can_put_down() {
            return;
        }
        let Some(carrying) = self.carrying.clone() else {
            return;
        };
        let here = self.folder().map(str::to_string);
        let key = carrying.entry.key.clone();
        let command = match carrying.kind {
            CarryKind::Move => Command::Move(carrying.entry.reference(), here),
            CarryKind::Copy => Command::Copy(key, here),
            CarryKind::Restore => Command::RestoreTo(key, here),
        };
        self.run(command, NameRule::Refuse, None);
    }

    /// Stop carrying; nothing happened.
    pub fn stop_carrying(&mut self) {
        self.carrying = None;
    }
}
