//! Editing the document that is shown.
//!
//! Pattern: State — `Reader::editing` is `Some` while the person edits; the transitions begin,
//! change, save, cancel. Saving goes through the source with the revision the text came from, so
//! another person's change is a conflict, never silently overwritten. Exporting is a separate,
//! explicit action (ADR 0042, 0046).

use super::model::Reader;
use crate::error::ErrorKind;
use crate::labels::Labels;

/// An edit in progress.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editing {
    /// Which edit this is (counts up with every edit begun; a shell ties its text area to it).
    pub id: u64,
    /// The text when the edit began.
    pub original: String,
    /// The text now, as it will be saved (line ends and final newline of the original kept).
    pub draft: String,
    /// The revision the text came from.
    pub revision: Option<String>,
    /// Why the last save failed, if it did.
    pub error: Option<String>,
    /// The person pressed Cancel once with unsaved changes.
    pub discard_asked: bool,
}

impl Editing {
    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.original != self.draft
    }

    /// `typed` as the text area gives it (it drops a final newline and turns `\r\n` into `\n`),
    /// put back into the form of the original, so that saving does not rewrite what was not
    /// touched.
    pub(crate) fn restore_form(&self, typed: &str) -> String {
        let mut text = typed.to_string();
        if self.original.ends_with('\n') && !text.ends_with('\n') {
            text.push('\n');
        }
        if self.original.contains("\r\n") && !text.contains("\r\n") {
            text = text.replace('\n', "\r\n");
        }
        text
    }
}

impl Reader {
    /// The edit in progress, if any.
    pub fn editing(&self) -> Option<&Editing> {
        self.editing.as_ref()
    }

    /// Can the document shown be changed (show an Edit button)?
    pub fn can_edit(&self) -> bool {
        self.current_path().is_some_and(|p| self.source.can_edit(p))
    }

    /// Can the document shown be exported (show an Export button)?
    pub fn can_export(&self) -> bool {
        self.export_target.is_some()
            && self
                .current_path()
                .is_some_and(|p| self.source.can_export(p))
    }

    /// Start editing the document shown. Does nothing while an edit is in progress.
    pub fn begin_edit(&mut self) {
        if self.editing.is_some() {
            return;
        }
        let Some(path) = self.current_path().map(str::to_string) else {
            return;
        };
        match self.source.source_text(&path) {
            Ok(editable) => {
                self.notice = None;
                self.edit_counter += 1;
                self.editing = Some(Editing {
                    id: self.edit_counter,
                    original: editable.text.clone(),
                    draft: editable.text,
                    revision: editable.revision,
                    error: None,
                    discard_asked: false,
                });
            }
            Err(e) => self.notice = Some(Labels::fill(&self.labels.cannot_edit, &e.message)),
        }
    }

    /// The person typed: remember the text.
    pub fn set_draft(&mut self, typed: &str) {
        if let Some(editing) = &mut self.editing {
            editing.draft = editing.restore_form(typed);
            editing.discard_asked = false;
        }
    }

    /// Save the draft. Without changes nothing is written. On success the document is read again
    /// and shown; on failure (also a conflict) the draft stays and the reason is shown.
    pub fn save(&mut self) {
        let (Some(path), Some(editing)) = (self.current_path().map(str::to_string), &self.editing)
        else {
            return;
        };
        if !editing.changed() {
            self.editing = None;
            self.notice = Some(self.labels.no_changes.clone());
            return;
        }
        let (draft, revision) = (editing.draft.clone(), editing.revision.clone());
        match self.source.save(&path, &draft, revision.as_deref()) {
            Ok(()) => {
                self.show(&path);
                self.refresh_entries();
                let term = self.term().to_string();
                self.search(&term);
                self.notice = Some(self.labels.saved.clone());
            }
            Err(e) => {
                let reason = if e.kind == ErrorKind::Conflict {
                    e.message.clone()
                } else {
                    Labels::fill(&self.labels.not_saved, &e.message)
                };
                if let Some(editing) = &mut self.editing {
                    editing.error = Some(reason);
                }
            }
        }
    }

    /// Stop editing. With unsaved changes the first call only asks; the second discards.
    pub fn cancel_edit(&mut self) {
        let Some(editing) = &mut self.editing else {
            return;
        };
        if editing.changed() && !editing.discard_asked {
            editing.discard_asked = true;
            self.notice = Some(self.labels.discard_confirm.clone());
            return;
        }
        self.editing = None;
        self.notice = None;
    }

    /// Export the document shown to the target (an explicit action; saving never writes files).
    /// A target that is already taken is reported; calling again confirms the replacement.
    pub fn export(&mut self) {
        let (Some(path), Some(target)) = (
            self.current_path().map(str::to_string),
            self.export_target.clone(),
        ) else {
            return;
        };
        let overwrite = self.overwrite_ok.as_deref() == Some(path.as_str());
        self.overwrite_ok = None;
        self.notice = Some(match self.source.export(&path, &target, overwrite) {
            Ok(message) => message,
            Err(e) if e.kind == ErrorKind::Conflict => {
                self.overwrite_ok = Some(path);
                Labels::fill(&self.labels.overwrite_confirm, &e.message)
            }
            Err(e) => Labels::fill(&self.labels.not_exported, &e.message),
        });
    }

    /// While there are unsaved changes, going elsewhere is refused with a notice.
    pub(super) fn blocked_by_edit(&mut self) -> bool {
        if self.editing.as_ref().is_some_and(Editing::changed) {
            self.notice = Some(self.labels.unsaved.clone());
            return true;
        }
        self.editing = None;
        false
    }
}
