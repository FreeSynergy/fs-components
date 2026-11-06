//! Deleting a person.
//!
//! Pattern: State — deleting needs a second press, tied to that person; anything that shows another
//! person drops the question. A failure is kept and shown, and the person stays.

use super::labels::PeopleLabels;
use super::shown::Shown;
use super::state::PeopleState;

impl PeopleState {
    /// Is a person shown who can be deleted (show a Delete button)?
    pub fn can_delete(&self) -> bool {
        self.capabilities.delete && self.editing.is_none() && self.selected().is_some()
    }

    /// Has the person pressed Delete once, for the one selected?
    pub fn delete_asked(&self) -> bool {
        self.delete_asked.is_some() && self.delete_asked.as_deref() == self.selected()
    }

    /// The first call only asks; the second, for the same person, deletes. A failure is kept and
    /// shown, and the person stays.
    pub fn delete(&mut self) {
        if !self.can_delete() {
            return;
        }
        let Some(key) = self.selected().map(str::to_string) else {
            return;
        };
        if self.delete_asked.as_deref() != Some(key.as_str()) {
            self.delete_asked = Some(key);
            self.notice = Some(PeopleLabels::fill(
                &self.labels.delete_confirm,
                &self.title(),
            ));
            return;
        }
        self.delete_asked = None;
        let title = self.title();
        if let Err(e) = self.source.delete(&key) {
            self.notice = Some(PeopleLabels::fill(&self.labels.not_deleted, &e.message));
            return;
        }
        let place = self.rows().iter().position(|r| r.key == key).unwrap_or(0);
        self.reload(self.loaded());
        let next = self
            .rows()
            .get(place)
            .or_else(|| self.rows().last())
            .map(|r| r.key.clone());
        match next {
            Some(next) => self.show(&next),
            None => self.shown = Shown::Nothing,
        }
        self.notice = Some(PeopleLabels::fill(&self.labels.deleted, &title));
    }

    /// Answer the question of [`PeopleState::delete`] with no.
    pub fn keep(&mut self) {
        if self.delete_asked.take().is_some() {
            self.notice = None;
        }
    }

    fn title(&self) -> String {
        match &self.shown {
            Shown::Person { sheet, .. } => sheet.title.clone(),
            Shown::Failed { key, .. } => key.clone(),
            Shown::Nothing => String::new(),
        }
    }
}
