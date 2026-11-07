//! The changes that a taken name can stop, and carrying something to another folder.
//!
//! Pattern: Command — a change is a value, so it can run again with the keep-both rule when the person
//! says so. A taken name asks; there is no rule that replaces. A move, copy or restore to a place is
//! carried while the person goes to the folder, and done there by another press.

use super::ask::{Ask, InputAsk, NameTaken};
use super::error::{FileError, FileProblem};
use super::labels::FileLabels;
use super::model::{EntryKind, EntryRef, FileEntry, NameRule};
use super::state::{FilesState, Notice};

/// A change that can meet a taken name.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Give it another name.
    Rename(EntryRef, String),
    /// Move it to this folder.
    Move(EntryRef, Option<String>),
    /// Copy this file to this folder.
    Copy(String, Option<String>),
    /// Bring this file back to where it was.
    Restore(String),
    /// Bring this file back to this folder.
    RestoreTo(String, Option<String>),
    /// Upload the file at this path into this folder.
    Upload(String, Option<String>),
}

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
    /// Run a change. A taken name asks what to do and changes nothing; a bad name goes back to the
    /// input it came from, which stays as typed; anything else says why and reads again.
    pub(super) fn run(&mut self, command: Command, rule: NameRule, from: Option<InputAsk>) {
        let source = self.source.clone();
        let done = match &command {
            Command::Rename(entry, name) => source
                .rename(entry, name, rule)
                .map(|n| FileLabels::fill(&self.labels.renamed, &n)),
            Command::Move(entry, to) => source
                .move_to(entry, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&self.labels.moved, &n)),
            Command::Copy(key, to) => source
                .copy_to(key, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&self.labels.copied, &n)),
            Command::Restore(key) => source
                .restore(key, rule)
                .map(|n| FileLabels::fill(&self.labels.restored, &n)),
            Command::RestoreTo(key, to) => source
                .restore_to(key, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&self.labels.restored, &n)),
            Command::Upload(path, to) => source
                .upload(path, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&self.labels.uploaded, &n)),
        };
        match done {
            Ok(said) => {
                self.ask = Ask::Nothing;
                self.carrying = None;
                self.transfer_done(&command, &said);
                self.after_change();
                self.notice = Some(Notice::Info(said));
            }
            Err(e) => self.not_run(command, rule, from, &e),
        }
    }

    fn not_run(&mut self, command: Command, rule: NameRule, from: Option<InputAsk>, e: &FileError) {
        self.transfer_failed(&command, e);
        if e.problem == FileProblem::NameConflict
            && rule == NameRule::Refuse
            && self.capabilities.keep_both
        {
            self.after_change();
            self.notice = None;
            self.ask = Ask::NameTaken(NameTaken {
                command,
                message: self.problem_text(e),
                back: from,
            });
            return;
        }
        self.ask = Ask::Nothing;
        if let Some(mut input) = from {
            input.error = Some(self.problem_text(e));
            self.ask = Ask::Input(input);
            self.after_change();
            return;
        }
        self.carrying = None;
        self.failed(e);
    }

    /// The name is taken and the person wants both: the incoming one gets a free name.
    pub fn keep_both(&mut self) {
        if let Ask::NameTaken(question) = &self.ask {
            let command = question.command.clone();
            self.run(command, NameRule::KeepBoth, None);
        }
    }

    /// The name is taken and the person does not want both: back to the input, or to where it was.
    pub fn name_taken_back(&mut self) {
        if let Ask::NameTaken(question) = &self.ask {
            self.ask = match &question.back {
                Some(input) => Ask::Input(input.clone()),
                None => Ask::Nothing,
            };
        }
    }

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

    /// Bring what is selected back from the trash to where it was.
    pub fn restore_selected(&mut self) {
        if !self.capabilities.trash || self.typing() {
            return;
        }
        if let Some(entry) = self.selected_entry().filter(|e| e.trashed) {
            let command = Command::Restore(entry.key.clone());
            self.run(command, NameRule::Refuse, None);
        }
    }
}
