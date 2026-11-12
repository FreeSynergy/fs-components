//! Destructive acts: asked in one place, done in another.
//!
//! Pattern: State — two different actions. The first only asks and says exactly what would go; pressing
//! it again only asks again, so a double-click can never do it, whatever is drawn under the pointer by
//! then. The answer is tied to what was asked about. Emptying the trash counts again before it does
//! anything: if the number told is no longer right, nothing is deleted and the person is told the new one.

use super::ask::{Ask, Confirm};
use super::labels::FileLabels;
use super::model::EntryKind;
use super::state::{FilesState, Notice};

impl FilesState {
    /// Is the question about this on the table?
    pub fn confirm_asked(&self) -> Option<&Confirm> {
        match &self.ask {
            Ask::Confirm(confirm) => Some(confirm),
            _ => None,
        }
    }

    /// A question about the thing selected no longer applies when another is selected.
    pub(super) fn drop_question_about_selected(&mut self) {
        if matches!(
            self.ask,
            Ask::Confirm(
                Confirm::Trash { .. }
                    | Confirm::DeleteFolder { .. }
                    | Confirm::Purge { .. }
                    | Confirm::ChooseMarked { .. }
            )
        ) {
            self.ask = Ask::Nothing;
        }
    }

    /// The first press of Delete: a file goes to the trash, a folder is deleted if empty, what is in the
    /// trash is gone for good. It only asks.
    pub fn ask_delete(&mut self) {
        let Some(entry) = self.selected_entry().cloned() else {
            return;
        };
        if !self.capabilities.trash || self.typing() {
            return;
        }
        let name = self.named(&entry);
        let key = entry.key;
        self.notice = None;
        self.ask = Ask::Confirm(if entry.trashed {
            Confirm::Purge { key, name }
        } else if entry.kind == EntryKind::Folder {
            Confirm::DeleteFolder { key, name }
        } else {
            Confirm::Trash { key, name }
        });
    }

    /// The first press of Empty trash: counts, and says how many. An empty trash does not ask.
    pub fn ask_empty_trash(&mut self) {
        if !self.capabilities.trash || !self.in_trash() || self.typing() {
            return;
        }
        match self.source.trash_count() {
            Ok(count) if count.files > 0 => {
                self.notice = None;
                self.ask = Ask::Confirm(Confirm::EmptyTrash {
                    files: count.files,
                    marked: count.marked,
                });
            }
            Ok(_) => {
                // read again first: what that finds out (the selected file is gone) is not said over
                // the better news
                self.after_change();
                self.notice = Some(Notice::Info(self.labels.trash_empty.clone()));
            }
            Err(e) => self.failed(&e),
        }
    }

    /// The answer no.
    pub fn keep(&mut self) {
        if matches!(self.ask, Ask::Confirm(_)) {
            self.ask = Ask::Nothing;
        }
    }

    /// The second act: does what was asked about. Does nothing unless asked.
    pub fn really(&mut self) {
        let Ask::Confirm(confirm) = std::mem::take(&mut self.ask) else {
            return;
        };
        let source = self.source.clone();
        let done = match &confirm {
            Confirm::Trash { key, name } => source
                .trash(key)
                .map(|()| FileLabels::fill(&self.labels.trashed, name)),
            Confirm::DeleteFolder { key, name } => source
                .delete_folder(key)
                .map(|()| FileLabels::fill(&self.labels.folder_deleted, name)),
            Confirm::Purge { key, name } => source
                .purge(key)
                .map(|()| FileLabels::fill(&self.labels.purged, name)),
            Confirm::EmptyTrash { files, marked } => {
                self.really_empty(*files, *marked);
                return;
            }
            Confirm::DownloadMarked { key, name, path } => {
                self.start_download((key, name, path), true, None);
                return;
            }
            Confirm::DropUpload { id, name } => {
                self.drop_now(id, name);
                return;
            }
            Confirm::ChooseMarked { key, name } => {
                self.picked_now(key, name, true);
                return;
            }
        };
        match done {
            Ok(said) => {
                self.selected = None;
                self.detail_open = false;
                self.after_change();
                self.notice = Some(Notice::Info(said));
            }
            Err(e) => self.failed(&e),
        }
    }

    fn really_empty(&mut self, files: usize, marked: usize) {
        match self.source.trash_count() {
            Ok(now) if now.files != files || now.marked != marked => {
                self.notice = Some(Notice::Info(self.labels.empty_changed.clone()));
                self.ask = Ask::Confirm(Confirm::EmptyTrash {
                    files: now.files,
                    marked: now.marked,
                });
                if now.files == 0 {
                    self.ask = Ask::Nothing;
                }
                self.after_change();
                return;
            }
            Ok(_) => {}
            Err(e) => {
                self.failed(&e);
                return;
            }
        }
        match self.source.empty_trash() {
            Ok(report) => {
                self.selected = None;
                self.detail_open = false;
                self.after_change();
                let text = if report.failed.is_empty() {
                    FileLabels::fill(&self.labels.emptied, &report.purged.to_string())
                } else {
                    // one whole sentence of its own, not two pieces put together
                    FileLabels::fill_named(
                        &self.labels.emptied_failed,
                        "names",
                        &report.failed.join(", "),
                    )
                };
                self.notice = Some(Notice::Info(text));
            }
            Err(e) => self.failed(&e),
        }
    }

    /// The words of the question on the table, and of the button that answers yes.
    pub fn confirm_words(&self) -> Option<(String, String)> {
        let labels = &self.labels;
        let fill = FileLabels::fill;
        Some(match self.confirm_asked()? {
            Confirm::Trash { name, .. } => (
                fill(&labels.trash_confirm, name),
                labels.trash_really.clone(),
            ),
            Confirm::DeleteFolder { name, .. } => (
                fill(&labels.delete_folder_confirm, name),
                labels.delete_folder_really.clone(),
            ),
            Confirm::Purge { name, .. } => (
                fill(&labels.purge_confirm, name),
                labels.purge_really.clone(),
            ),
            Confirm::EmptyTrash { files, marked } => {
                let text = if *marked > 0 {
                    &labels.empty_confirm_marked
                } else {
                    &labels.empty_confirm
                };
                (fill(text, &files.to_string()), labels.empty_really.clone())
            }
            Confirm::DownloadMarked { name, .. } => (
                fill(&labels.download_marked_confirm, name),
                labels.download_marked_really.clone(),
            ),
            Confirm::DropUpload { name, .. } => {
                (fill(&labels.drop_confirm, name), labels.drop_really.clone())
            }
            Confirm::ChooseMarked { name, .. } => (
                fill(&labels.choose_marked_confirm, name),
                labels.choose_marked.clone(),
            ),
        })
    }
}
