//! The thing selected, and what is shown of it.
//!
//! Pattern: State — nothing is selected by itself. What the preview shows is decided
//! here: a marked file is not read until the person asks (ADR 0029); a file in the trash cannot be read;
//! a file that changed outside is shown as refused, not hidden; text and pictures go through the checks
//! of `file-preview`. A press on the folder already selected opens it: so a double-click goes one level.

use fs_components_widgets::file_preview::{
    Fact, FilePreview, PreviewContent, picture_address, picture_fits, safe_text,
};
use fs_components_widgets::transfer_list::format_bytes;

use super::error::FileProblem;
use super::labels::FileLabels;
use super::model::{EntryKind, FileEntry};
use super::side::Side;
use super::state::{FilesState, Notice};

/// Bytes read for a preview of text.
const TEXT_BYTES: usize = 64 * 1024;
/// Largest picture read for a preview.
const PICTURE_BYTES: u64 = 8 * 1024 * 1024;
/// Widest and highest picture handed to Blinc, which decodes at full size.
const PICTURE_SIDE: u32 = 4096;

/// What is selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Selected {
    pub entry: FileEntry,
    pub content: PreviewContent,
    pub released: bool,
}

impl FilesState {
    /// Select this row. With the row already selected, a folder opens.
    pub fn select(&mut self, key: &str) {
        if self.selected_key() == Some(key) {
            self.side = Side::Item;
            self.detail_open = true;
            if let Some(entry) = self
                .selected_entry()
                .filter(|e| e.kind == EntryKind::Folder)
                && !self.in_trash()
            {
                let key = entry.key.clone();
                self.open_folder(Some(&key));
            }
            return;
        }
        let Some(entry) = self.rows.iter().find(|r| r.key == key).cloned() else {
            return;
        };
        self.drop_question_about_selected();
        self.notice = None;
        self.side = Side::Item;
        self.detail_open = true;
        self.show(entry, false);
    }

    /// Open the folder selected.
    pub fn open_selected(&mut self) {
        if let Some(entry) = self
            .selected_entry()
            .filter(|e| e.kind == EntryKind::Folder)
        {
            let key = entry.key.clone();
            self.open_folder(Some(&key));
        }
    }

    /// The key of what is selected.
    pub fn selected_key(&self) -> Option<&str> {
        self.selected.as_ref().map(|s| s.entry.key.as_str())
    }

    /// What is selected.
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.selected.as_ref().map(|s| &s.entry)
    }

    /// What the preview shows, if something is selected.
    pub fn content(&self) -> Option<&PreviewContent> {
        self.selected.as_ref().map(|s| &s.content)
    }

    /// Is a marked file selected whose content the person has not asked for?
    pub fn preview_held_back(&self) -> bool {
        self.selected.as_ref().is_some_and(|s| {
            s.entry.is_marked()
                && !s.released
                && s.entry.kind == EntryKind::File
                && !s.entry.trashed
        })
    }

    /// The person asks to see this marked file. Only for this selection; it does not stick.
    pub fn release_preview(&mut self) {
        if self.preview_held_back()
            && let Some(selected) = &self.selected
        {
            let entry = selected.entry.clone();
            self.show(entry, true);
        }
    }

    /// The preview as the widget wants it.
    pub fn preview(&self) -> Option<FilePreview> {
        let selected = self.selected.as_ref()?;
        let entry = &selected.entry;
        let labels = &self.labels;
        let mut preview = FilePreview::new(entry.name.clone());
        if entry.is_marked() {
            preview.marks.push(labels.marked.clone());
        }
        if entry.trashed {
            preview.marks.push(labels.in_trash.clone());
        }
        preview
            .facts
            .push(Fact::new(labels.fact_type.clone(), self.type_word(entry)));
        if entry.kind == EntryKind::File {
            preview.facts.push(Fact::new(
                labels.fact_size.clone(),
                format_bytes(entry.size),
            ));
        }
        if let Some(when) = &entry.modified {
            preview
                .facts
                .push(Fact::new(labels.fact_modified.clone(), when.clone()));
        }
        if entry.is_marked() {
            preview.facts.push(Fact::new(
                labels.fact_marks.clone(),
                entry.sensitive.join(", "),
            ));
        }
        preview.content = selected.content.clone();
        Some(preview)
    }

    /// The word for what kind of thing it is.
    pub fn type_word(&self, entry: &FileEntry) -> String {
        if entry.kind == EntryKind::Folder {
            self.labels.folder_word.clone()
        } else if entry.media_type.is_empty() || entry.media_type == "application/octet-stream" {
            self.labels.unknown_type.clone()
        } else {
            short_type(&entry.media_type)
        }
    }

    pub(super) fn show(&mut self, entry: FileEntry, released: bool) {
        let content = self.content_of(&entry, released);
        self.selected = Some(Selected {
            entry,
            content,
            released,
        });
    }

    /// The thing selected, as it is now. If it is gone it leaves the view and that is said.
    pub(super) fn refresh_selected(&mut self) {
        let Some(selected) = self.selected.clone() else {
            return;
        };
        match self.source.entry(&selected.entry.reference()) {
            Ok(Some(entry)) => {
                let moved_out = self.in_trash() != entry.trashed;
                if moved_out {
                    self.selected = None;
                } else {
                    self.show(entry, selected.released);
                }
            }
            Ok(None) => {
                self.selected = None;
                self.notice = Some(Notice::Info(FileLabels::fill(
                    &self.labels.gone,
                    &selected.entry.name,
                )));
            }
            Err(_) => {} // What is drawn stays: a failed read says nothing about the file.
        }
    }

    fn content_of(&self, entry: &FileEntry, released: bool) -> PreviewContent {
        let labels = &self.labels;
        if entry.kind == EntryKind::Folder {
            return PreviewContent::Nothing;
        }
        if entry.trashed {
            return PreviewContent::Note(labels.preview_trash.clone());
        }
        if entry.is_marked() && !released {
            return PreviewContent::Note(labels.preview_marked.clone());
        }
        let picture = matches!(entry.media_type.as_str(), "image/png" | "image/jpeg");
        if picture && entry.size > PICTURE_BYTES {
            return PreviewContent::Note(labels.preview_too_big.clone());
        }
        #[allow(clippy::cast_possible_truncation)] // bounded by PICTURE_BYTES
        let limit = if picture {
            entry.size as usize
        } else {
            TEXT_BYTES
        };
        let read = match self.source.preview(&entry.key, limit) {
            Ok(read) => read,
            Err(e) if e.problem == FileProblem::ChangedOutside => {
                return PreviewContent::Refused(self.problem_text(&e));
            }
            Err(e) => return PreviewContent::Refused(self.problem_text(&e)),
        };
        if picture {
            return if picture_fits(&read.bytes, PICTURE_SIDE) {
                PreviewContent::Picture(picture_address(&entry.media_type, &read.bytes))
            } else {
                PreviewContent::Note(labels.preview_too_big.clone())
            };
        }
        let more = (read.bytes.len() as u64) < read.total;
        match safe_text(&read.bytes, more) {
            Some(text) => PreviewContent::Text(text, labels.preview_cut.clone()),
            None => PreviewContent::Note(labels.preview_binary.clone()),
        }
    }
}

/// A media type as a short word: the part after the slash without `x-`, `vnd.` and `+suffix`
/// (`image/jpeg` is `JPEG`, `application/x-tar` is `TAR`).
fn short_type(media_type: &str) -> String {
    let sub = media_type.split_once('/').map_or(media_type, |(_, s)| s);
    let sub = sub.split(['+', ';']).next().unwrap_or(sub);
    let sub = sub.trim_start_matches("x-").trim_start_matches("vnd.");
    let sub = sub.rsplit('.').next().unwrap_or(sub);
    if sub.chars().count() <= 5 {
        sub.to_uppercase()
    } else {
        let mut chars = sub.chars();
        chars
            .next()
            .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
    }
}
