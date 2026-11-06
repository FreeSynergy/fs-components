//! What the workbench is waiting for from the person.
//!
//! Pattern: State — one question at a time. A destructive act is a question first and an answer at
//! another place; pressing the first again only asks again, so a double-click can never do it.

use super::command::Command;
use super::model::EntryRef;

/// What a line of text is for.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputKind {
    /// The name of a new folder in the current one.
    NewFolder,
    /// The new name of this.
    Rename(EntryRef),
    /// The path of a file to upload.
    UploadPath,
    /// The path to write this file to; `release` is set once the person agreed to write a marked file.
    DownloadPath {
        /// The key of the file.
        key: String,
        /// Its name.
        name: String,
    },
    /// The path of the file to go on with for this unfinished upload.
    ResumePath {
        /// The id of the upload.
        id: String,
    },
}

/// A line of text being typed.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputAsk {
    /// Which input this is (counts up; a view ties its text field to it).
    pub id: u64,
    /// What it is for.
    pub kind: InputKind,
    /// The words above it.
    pub label: String,
    /// What it held at the start.
    pub original: String,
    /// What it holds now.
    pub value: String,
    /// Why the last try failed.
    pub error: Option<String>,
    /// Cancel was pressed once with something typed.
    pub discard_asked: bool,
}

impl InputAsk {
    /// Has something been typed?
    pub fn dirty(&self) -> bool {
        self.value != self.original
    }
}

/// A destructive act that has been asked for once.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirm {
    /// Put this file in the trash.
    Trash {
        /// The key.
        key: String,
        /// The words of the question.
        name: String,
    },
    /// Delete this folder.
    DeleteFolder {
        /// The key.
        key: String,
        /// The words of the question.
        name: String,
    },
    /// Delete this file in the trash for good.
    Purge {
        /// The key.
        key: String,
        /// The words of the question.
        name: String,
    },
    /// Delete all in the trash for good; what the person was told.
    EmptyTrash {
        /// Files.
        files: usize,
        /// Of these, marked.
        marked: usize,
    },
    /// Write this marked file out of the program, to this path.
    DownloadMarked {
        /// The key.
        key: String,
        /// The name.
        name: String,
        /// The path typed.
        path: String,
    },
    /// Throw this unfinished upload away.
    DropUpload {
        /// The id.
        id: String,
        /// The name.
        name: String,
    },
    /// Give this marked file to the program that asked.
    ChooseMarked {
        /// The key.
        key: String,
        /// The name.
        name: String,
    },
}

/// A name is taken: the person says what to do. There is no replace.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameTaken {
    /// What was tried; it runs again with keep-both if the person wants.
    pub command: Command,
    /// What the source said.
    pub message: String,
    /// The input to go back to, if it came from one.
    pub back: Option<InputAsk>,
}

/// What the workbench is waiting for.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Ask {
    /// Nothing.
    #[default]
    Nothing,
    /// A line of text.
    Input(InputAsk),
    /// The second act of something destructive.
    Confirm(Confirm),
    /// What to do about a taken name.
    NameTaken(NameTaken),
}
