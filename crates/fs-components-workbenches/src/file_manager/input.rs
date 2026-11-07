//! A line of text being typed: a name or a path.
//!
//! Pattern: State — one input at a time. With something typed, the first Cancel only asks; the answer
//! that throws it away is another act at another place. A failed try keeps what was typed and says why.

use super::ask::{Ask, Confirm, InputAsk, InputKind};
use super::command::Command;
use super::labels::FileLabels;
use super::model::{EntryKind, EntryRef, NameRule};
use super::state::{FilesState, Notice};

impl FilesState {
    fn begin_input(&mut self, kind: InputKind, label: String, value: String) {
        self.input_counter += 1;
        self.notice = None;
        self.ask = Ask::Input(InputAsk {
            id: self.input_counter,
            kind,
            label,
            original: value.clone(),
            value,
            error: None,
            discard_asked: false,
        });
    }

    /// Would asking something else now throw what was typed away? Then that is refused, and the band
    /// asks whether to throw it away: that stays the person's second act.
    pub(super) fn typing(&mut self) -> bool {
        if let Ask::Input(input) = &mut self.ask
            && input.dirty()
        {
            input.discard_asked = true;
            return true;
        }
        false
    }

    /// Ask for the name of a new folder here.
    pub fn begin_new_folder(&mut self) {
        if self.capabilities.change && !self.in_trash() && !self.typing() {
            let label = self.labels.label_new_folder.clone();
            self.begin_input(InputKind::NewFolder, label, String::new());
        }
    }

    /// Ask for the new name of what is selected.
    pub fn begin_rename(&mut self) {
        if !self.capabilities.change || self.typing() {
            return;
        }
        if let Some(entry) = self.selected_entry().filter(|e| !e.trashed).cloned() {
            let label = FileLabels::fill(&self.labels.label_rename, &self.named(&entry));
            self.begin_input(InputKind::Rename(entry.reference()), label, entry.name);
        }
    }

    /// Ask for the path of a file to upload here.
    pub fn begin_upload(&mut self) {
        if self.capabilities.upload && !self.in_trash() && !self.typing() {
            let label = self.labels.label_upload.clone();
            self.begin_input(InputKind::UploadPath, label, String::new());
        }
    }

    /// Ask where to write the file selected.
    pub fn begin_download(&mut self) {
        if !self.capabilities.download || self.typing() {
            return;
        }
        if let Some(entry) = self
            .selected_entry()
            .filter(|e| !e.trashed && e.kind == EntryKind::File)
            .cloned()
        {
            let label = FileLabels::fill(&self.labels.label_download, &self.named(&entry));
            let kind = InputKind::DownloadPath {
                key: entry.key,
                name: entry.name,
            };
            self.begin_input(kind, label, String::new());
        }
    }

    pub(super) fn begin_download_of(&mut self, key: String, name: String) {
        if !self.typing() {
            let label = FileLabels::fill(&self.labels.label_download, &name);
            self.begin_input(InputKind::DownloadPath { key, name }, label, String::new());
        }
    }

    pub(super) fn begin_resume(&mut self, id: String, name: &str) {
        if !self.typing() {
            let label = FileLabels::fill(&self.labels.label_resume, name);
            self.begin_input(InputKind::ResumePath { id }, label, String::new());
        }
    }

    /// The person typed this. It takes back the question whether to throw it away.
    pub fn set_input(&mut self, text: &str) {
        if let Ask::Input(input) = &mut self.ask
            && input.value != text
        {
            input.value = text.to_string();
            input.discard_asked = false;
        }
    }

    /// Cancel. With something typed the first press only asks.
    pub fn cancel_input(&mut self) {
        if let Ask::Input(input) = &mut self.ask {
            if input.dirty() {
                input.discard_asked = true;
            } else {
                self.ask = Ask::Nothing;
            }
        }
    }

    /// The answer that keeps what was typed.
    pub fn keep_typing(&mut self) {
        if let Ask::Input(input) = &mut self.ask {
            input.discard_asked = false;
        }
    }

    /// The second act: what was typed is thrown away. Does nothing unless asked.
    pub fn discard_input(&mut self) {
        if matches!(&self.ask, Ask::Input(input) if input.discard_asked) {
            self.ask = Ask::Nothing;
        }
    }

    /// Leave an input whose try failed, keeping nothing typed; for a host's own way out. A dirty input
    /// is otherwise two acts.
    pub fn discard_after_failure(&mut self) {
        if matches!(&self.ask, Ask::Input(input) if input.error.is_some() || !input.dirty()) {
            self.ask = Ask::Nothing;
        }
    }

    /// Do what the input is for.
    pub fn submit_input(&mut self) {
        let Ask::Input(input) = &self.ask else {
            return;
        };
        let input = input.clone();
        let value = input.value.clone();
        match input.kind.clone() {
            InputKind::NewFolder => self.create_folder(input, &value),
            InputKind::Rename(entry) => {
                if value == input.original {
                    self.ask = Ask::Nothing;
                    return;
                }
                self.run(Command::Rename(entry, value), NameRule::Refuse, Some(input));
            }
            InputKind::UploadPath => {
                let here = self.folder().map(str::to_string);
                self.run(Command::Upload(value, here), NameRule::Refuse, Some(input));
            }
            InputKind::DownloadPath { key, name } => {
                self.submit_download(input, &key, &name, &value);
            }
            InputKind::ResumePath { id } => self.resume_now(input, &id, &value),
        }
    }

    fn create_folder(&mut self, input: InputAsk, name: &str) {
        let here = self.folder().map(str::to_string);
        match self.source.create_folder(here.as_deref(), name) {
            Ok(folder) => {
                self.ask = Ask::Nothing;
                self.after_folder_change();
                self.notice = Some(Notice::Info(FileLabels::fill(
                    &self.labels.created,
                    &folder.name,
                )));
            }
            // Folders have no keep-both in the contract: the input stays and says why.
            Err(e) => self.input_failed(input, &e),
        }
    }

    pub(super) fn input_failed(&mut self, mut input: InputAsk, e: &super::error::FileError) {
        input.error = Some(self.problem_text(e));
        self.ask = Ask::Input(input);
        self.after_change();
    }

    fn submit_download(&mut self, input: InputAsk, key: &str, name: &str, path: &str) {
        let reference = EntryRef {
            key: key.to_string(),
            kind: EntryKind::File,
        };
        match self.source.entry(&reference) {
            Ok(Some(entry)) if entry.is_marked() => {
                self.ask_download_marked(key, self.named(&entry), path);
            }
            Ok(Some(_)) => {
                if !self.download_now(key, name, path, false) {
                    // A source that knows better than its listing has the last word.
                    if !matches!(self.ask, Ask::Confirm(_)) {
                        self.ask = Ask::Input(self.input_with_transfer_error(input));
                    }
                }
            }
            Ok(None) => {
                self.ask = Ask::Nothing;
                self.after_change();
                self.notice = Some(Notice::Info(FileLabels::fill(&self.labels.gone, name)));
            }
            Err(e) => self.input_failed(input, &e),
        }
    }

    pub(super) fn ask_download_marked(&mut self, key: &str, name: String, path: &str) {
        self.ask = Ask::Confirm(Confirm::DownloadMarked {
            key: key.to_string(),
            name,
            path: path.to_string(),
        });
    }

    fn input_with_transfer_error(&self, mut input: InputAsk) -> InputAsk {
        input.error = self.notice.as_ref().map(|n| n.text().to_string());
        input
    }
}
