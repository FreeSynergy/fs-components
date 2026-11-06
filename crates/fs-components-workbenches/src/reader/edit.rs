//! Editing the page that is shown.
//!
//! Pattern: State — `Reader::editing` is `Some` while the person edits; the transitions begin,
//! change, save, cancel. Saving goes through the data source (for the wiki: `Texts::edit`, the
//! database is the truth); exporting is a separate, explicit action (ADR 0042, 0046).

use super::model::Reader;

/// An edit in progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editing {
    /// The text when the edit began.
    pub original: String,
    /// The text now.
    pub draft: String,
    /// Why the last save failed, if it did.
    pub error: Option<String>,
}

impl Editing {
    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.original != self.draft
    }
}

impl Reader {
    /// The edit in progress, if any.
    pub fn editing(&self) -> Option<&Editing> {
        self.editing.as_ref()
    }

    /// Can the source change documents (show an Edit button)?
    pub fn can_edit(&self) -> bool {
        self.source.can_edit()
    }

    /// Can the source export documents (show an Export button)?
    pub fn can_export(&self) -> bool {
        self.source.can_export()
    }

    /// Start editing the page shown.
    pub fn begin_edit(&mut self) {
        let Some(path) = self.current_path().map(str::to_string) else {
            return;
        };
        match self.source.source_text(&path) {
            Ok(text) => {
                self.notice = None;
                self.editing = Some(Editing {
                    original: text.clone(),
                    draft: text,
                    error: None,
                });
            }
            Err(e) => self.notice = Some(format!("Cannot edit this page: {e}")),
        }
    }

    /// The person typed: remember the text.
    pub fn set_draft(&mut self, text: &str) {
        if let Some(editing) = &mut self.editing {
            editing.draft = text.to_string();
        }
    }

    /// Save the draft. On success the page is read again and shown; on failure the draft stays.
    pub fn save(&mut self) {
        let (Some(path), Some(editing)) = (self.current_path().map(str::to_string), &self.editing)
        else {
            return;
        };
        let draft = editing.draft.clone();
        match self.source.save(&path, &draft) {
            Ok(()) => {
                self.show(&path);
                self.notice = Some("Saved.".into());
            }
            Err(e) => {
                if let Some(editing) = &mut self.editing {
                    editing.error = Some(format!("Not saved: {e}"));
                }
            }
        }
    }

    /// Stop editing and drop the draft.
    pub fn cancel_edit(&mut self) {
        self.editing = None;
    }

    /// Export the page shown (an explicit action; saving never writes files).
    pub fn export(&mut self) {
        let Some(path) = self.current_path().map(str::to_string) else {
            return;
        };
        self.notice = Some(match self.source.export(&path) {
            Ok(message) => message,
            Err(e) => format!("Not exported: {e}"),
        });
    }

    /// While there are unsaved changes, going elsewhere is refused with a notice.
    pub(super) fn blocked_by_edit(&mut self) -> bool {
        if self.editing.as_ref().is_some_and(super::Editing::changed) {
            self.notice = Some("Save or cancel your changes first.".into());
            return true;
        }
        self.editing = None;
        false
    }
}
