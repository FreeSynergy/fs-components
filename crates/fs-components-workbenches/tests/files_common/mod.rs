//! A file source for the tests of the workbench.
//!
//! Tests: a test double — folders and files in memory; it stands for "any source", not for a
//! program's core (that is tested in `fs-files`, over the real core and real folders).
#![allow(dead_code)] // each test file uses a different subset

pub mod harness;
mod keys;
mod source;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use fs_components_workbenches::file_manager::{
    EntryKind, FileEntry, FileError, FileLabels, FileManagerWidget, FileProblem, NameRule,
    PendingUpload,
};

/// A thing in the shelf.
#[derive(Clone, Debug)]
pub struct Thing {
    pub name: String,
    pub folder: Option<String>,
    pub is_folder: bool,
    pub trashed: bool,
    pub marked: bool,
    pub bytes: Vec<u8>,
    pub changed_outside: bool,
}

/// Things per key.
#[derive(Default)]
pub struct Shelf {
    pub things: Mutex<BTreeMap<String, Thing>>,
    pub broken: Mutex<Option<String>>,
    pub writes: Mutex<u32>,
    pub pending: Mutex<Vec<PendingUpload>>,
    pub downloads: Mutex<Vec<(String, String, bool)>>,
    pub page_size: Mutex<usize>,
    pub unreadable: Mutex<Vec<String>>,
    /// Trashed things that cannot be purged.
    pub stays: Mutex<Vec<String>>,
}

fn not_found() -> FileError {
    FileError::new(FileProblem::NotFound, "gone")
}

impl Shelf {
    pub fn new() -> Self {
        let shelf = Self::default();
        *shelf.page_size.lock().unwrap() = 50;
        shelf
    }

    pub fn file(self, key: &str) -> Self {
        self.file_in(key, None)
    }

    pub fn file_in(self, key: &str, folder: Option<&str>) -> Self {
        self.things.lock().unwrap().insert(
            key.into(),
            Thing {
                name: format!("{key}.txt"),
                folder: folder.map(str::to_string),
                is_folder: false,
                trashed: false,
                marked: false,
                bytes: format!("the text of {key}").into_bytes(),
                changed_outside: false,
            },
        );
        self
    }

    pub fn folder(self, key: &str, parent: Option<&str>) -> Self {
        self.things.lock().unwrap().insert(
            key.into(),
            Thing {
                name: key.into(),
                folder: parent.map(str::to_string),
                is_folder: true,
                trashed: false,
                marked: false,
                bytes: Vec::new(),
                changed_outside: false,
            },
        );
        self
    }

    pub fn numbered(self, count: usize) -> Self {
        let mut this = self;
        for i in 0..count {
            this = this.file(&format!("f{i:05}"));
        }
        this
    }

    pub fn change(&self, key: &str, change: impl FnOnce(&mut Thing)) {
        change(self.things.lock().unwrap().get_mut(key).unwrap());
    }

    pub fn thing(&self, key: &str) -> Option<Thing> {
        self.things.lock().unwrap().get(key).cloned()
    }

    pub fn writes(&self) -> u32 {
        *self.writes.lock().unwrap()
    }

    pub fn wrote(&self) {
        *self.writes.lock().unwrap() += 1;
    }

    pub fn break_it(&self, why: Option<&str>) {
        *self.broken.lock().unwrap() = why.map(str::to_string);
    }

    pub(super) fn check(&self) -> Result<(), FileError> {
        match self.broken.lock().unwrap().clone() {
            Some(why) => Err(FileError::failed(why)),
            None => Ok(()),
        }
    }

    pub(super) fn entry_of(key: &str, t: &Thing) -> FileEntry {
        let mut entry = if t.is_folder {
            FileEntry::folder(key, t.name.clone())
        } else {
            FileEntry::file(key, t.name.clone(), t.bytes.len() as u64)
        };
        entry.media_type = if t.is_folder {
            String::new()
        } else {
            "text/plain".into()
        };
        entry.folder.clone_from(&t.folder);
        entry.trashed = t.trashed;
        if t.marked {
            entry.sensitive = vec!["dpv:Health".into()];
        }
        entry
    }

    fn taken(&self, folder: Option<&str>, name: &str, except: &str) -> bool {
        self.things.lock().unwrap().iter().any(|(k, t)| {
            k != except && t.folder.as_deref() == folder && t.name == name && !t.trashed
        })
    }

    pub(super) fn free_name(
        &self,
        folder: Option<&str>,
        name: &str,
        except: &str,
        rule: NameRule,
    ) -> Result<String, FileError> {
        if name.contains('/') || name.is_empty() || name.len() > 255 {
            return Err(FileError::new(
                FileProblem::InvalidName,
                format!("bad: {name}"),
            ));
        }
        if !self.taken(folder, name, except) {
            return Ok(name.to_string());
        }
        if rule == NameRule::Refuse {
            return Err(FileError::new(
                FileProblem::NameConflict,
                format!("{name} is taken"),
            ));
        }
        Ok((2..)
            .map(|n| format!("{name} ({n})"))
            .find(|n| !self.taken(folder, n, except))
            .unwrap())
    }
}

pub fn widget(shelf: Shelf) -> (Arc<Shelf>, FileManagerWidget) {
    let shelf = Arc::new(shelf);
    (
        shelf.clone(),
        FileManagerWidget::new(shelf, FileLabels::default()),
    )
}

pub fn kind_of(shelf: &Shelf, key: &str) -> EntryKind {
    if shelf.thing(key).is_some_and(|t| t.is_folder) {
        EntryKind::Folder
    } else {
        EntryKind::File
    }
}
