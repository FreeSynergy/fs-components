//! The changes that a taken name can stop.
//!
//! Pattern: Command — a change is a value, so it can run again with the keep-both rule when the person
//! says so. A taken name asks; there is no rule that replaces.

use super::ask::{Ask, InputAsk, NameTaken};
use super::error::{FileError, FileProblem};
use super::labels::FileLabels;
use super::model::{EntryRef, NameRule};
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

impl FilesState {
    /// Run a change. A taken name asks what to do and changes nothing; a bad name goes back to the
    /// input it came from, which stays as typed; anything else says why and reads again.
    pub(super) fn run(&mut self, command: Command, rule: NameRule, from: Option<InputAsk>) {
        if matches!(command, Command::Upload(..)) {
            // The long one: on a worker, the answer comes back through `finish`.
            self.start_upload(command, rule, from);
            return;
        }
        let done = self.call_now(&command, rule);
        self.settled(command, rule, from, done);
    }

    /// The short changes: the call is made here, with the source's own time limit.
    fn call_now(&self, command: &Command, rule: NameRule) -> Result<String, FileError> {
        let source = &self.source;
        let labels = &self.labels;
        match command {
            Command::Rename(entry, name) => source
                .rename(entry, name, rule)
                .map(|n| FileLabels::fill(&labels.renamed, &n)),
            Command::Move(entry, to) => source
                .move_to(entry, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&labels.moved, &n)),
            Command::Copy(key, to) => source
                .copy_to(key, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&labels.copied, &n)),
            Command::Restore(key) => source
                .restore(key, rule)
                .map(|n| FileLabels::fill(&labels.restored, &n)),
            Command::RestoreTo(key, to) => source
                .restore_to(key, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&labels.restored, &n)),
            Command::Upload(path, to) => source
                .upload(path, to.as_deref(), rule)
                .map(|n| FileLabels::fill(&labels.uploaded, &n)),
        }
    }

    /// What came of a change: said, or asked, or given back to the input it came from.
    pub(super) fn settled(
        &mut self,
        command: Command,
        rule: NameRule,
        from: Option<InputAsk>,
        done: Result<String, FileError>,
    ) {
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
