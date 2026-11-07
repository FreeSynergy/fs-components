//! Deleting a place.
//!
//! Pattern: State — two different actions: asking and confirming. Pressing the first again only asks
//! again, so a double-click can never delete, whatever is drawn under the pointer by then. The question
//! is tied to that place; anything that shows another place drops it. A failure is kept and shown, and
//! the place stays; one someone else deleted just leaves the view. Afterwards nothing opens on its own.

use super::labels::MapLabels;
use super::shown::Shown;
use super::state::MapState;
use crate::error::ErrorKind;

/// The most of a title that goes into a sentence, in characters.
const LONGEST_IN_A_SENTENCE: usize = 80;

impl MapState {
    /// Is a place shown that can be deleted (show a Delete button)?
    pub fn can_delete(&self) -> bool {
        self.capabilities.delete
            && self.editing.is_none()
            && match &self.shown {
                Shown::Place { sheet, .. } => sheet.can_delete,
                Shown::Failed { .. } => true,
                Shown::Nothing => false,
            }
    }

    /// Has the person pressed Delete, for the one selected?
    pub fn delete_asked(&self) -> bool {
        self.delete_asked.is_some() && self.delete_asked.as_deref() == self.selected()
    }

    /// The first press: asks whether the place selected should really go, and says what goes with it.
    pub fn ask_delete(&mut self) {
        if !self.can_delete() {
            return;
        }
        self.delete_asked = self.selected().map(str::to_string);
        let mut question = MapLabels::fill(&self.labels.delete_confirm, &self.title());
        let warning = match &self.shown {
            Shown::Place { sheet, .. } => sheet.delete_warning.clone(),
            Shown::Failed { key, .. } => self.source.delete_warning(key),
            Shown::Nothing => None,
        };
        if let Some(warning) = warning {
            question.push(' ');
            question.push_str(&warning);
        }
        self.notice = Some(question);
    }

    /// The second action: deletes the one asked about. Does nothing unless asked.
    pub fn confirm_delete(&mut self) {
        if !self.can_delete() || !self.delete_asked() {
            return;
        }
        let Some(key) = self.selected().map(str::to_string) else {
            return;
        };
        self.delete_asked = None;
        let marked = matches!(&self.shown, Shown::Place { sheet, .. } if !sheet.marks.is_empty());
        let title = self.title();
        if let Err(e) = self.source.delete(&key) {
            if e.kind == ErrorKind::NotFound {
                self.place_gone();
            } else {
                self.notice = Some(MapLabels::fill(&self.labels.not_deleted, &e.message));
            }
            return;
        }
        self.reload_places();
        self.shown = Shown::Nothing;
        // The title of something marked does not stay on the screen after it has gone.
        self.notice = Some(if marked {
            self.labels.deleted_marked.clone()
        } else {
            MapLabels::fill(&self.labels.deleted, &title)
        });
    }

    /// Answer the question of [`MapState::ask_delete`] with no.
    pub fn keep(&mut self) {
        if self.delete_asked.take().is_some() {
            self.notice = None;
        }
    }

    /// What to call it in a sentence; a very long title is cut, so the sentence can still be read.
    fn title(&self) -> String {
        match &self.shown {
            Shown::Place { sheet, .. } if sheet.title.chars().count() > LONGEST_IN_A_SENTENCE => {
                let mut cut: String = sheet.title.chars().take(LONGEST_IN_A_SENTENCE).collect();
                cut.push('\u{2026}');
                cut
            }
            Shown::Place { sheet, .. } => sheet.title.clone(),
            Shown::Failed { key, .. } => key.clone(),
            Shown::Nothing => String::new(),
        }
    }
}
