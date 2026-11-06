//! Going elsewhere in the calendar.
//!
//! Pattern: State — each transition is one method. Moving the calendar leaves an edit alone, because
//! its form is not part of the calendar; choosing another item does not, and is refused while there
//! are unsaved changes.

use fs_components_widgets::calendar_view::{AGENDA_DAYS, step_from};
use fs_components_widgets::{Day, Picked, ViewRange};

use super::labels::CalendarLabels;
use super::shown::{Outcome, Shown};
use super::state::{CalendarState, FIRST_YEAR, LAST_YEAR};

impl CalendarState {
    /// Show a month, a week, a day or the agenda.
    pub fn set_view(&mut self, view: ViewRange) {
        if self.view != view {
            self.view = view;
            self.reload();
        }
    }

    /// One step on.
    pub fn next(&mut self) {
        self.go_to(step_from(self.view, self.anchor, true, AGENDA_DAYS));
    }

    /// One step back.
    pub fn previous(&mut self) {
        self.go_to(step_from(self.view, self.anchor, false, AGENDA_DAYS));
    }

    /// Go to today.
    pub fn go_today(&mut self) {
        let today = self.source.now().day;
        self.selected_day = Some(today);
        self.go_to(today);
    }

    /// Show the span around `day`. An edit in progress stays: its form is not part of the calendar.
    pub fn go_to(&mut self, day: Day) {
        if !(FIRST_YEAR..=LAST_YEAR).contains(&day.year()) {
            let edge = day.year().clamp(FIRST_YEAR, LAST_YEAR).to_string();
            self.notice = Some(CalendarLabels::fill(&self.labels.edge, &edge));
            return;
        }
        self.anchor = day;
        self.reload();
    }

    /// Take what a click on the calendar reported.
    pub fn pick(&mut self, picked: &Picked) -> Outcome {
        match picked {
            Picked::Item(key) => self.select(key),
            Picked::More(day) => {
                self.selected_day = Some(*day);
                self.view = if self.view == ViewRange::Day {
                    ViewRange::Agenda
                } else {
                    ViewRange::Day
                };
                self.go_to(*day);
                Outcome::Done
            }
            Picked::Day(day) => {
                self.selected_day = Some(*day);
                Outcome::Done
            }
            _ => Outcome::Done,
        }
    }

    /// Show this item. With unsaved changes this is refused and says why.
    pub fn select(&mut self, key: &str) -> Outcome {
        if self.selected() == Some(key) && self.editing.is_none() {
            return Outcome::Done;
        }
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.show(key);
        Outcome::Done
    }

    /// Show nothing. With unsaved changes this is refused and says why.
    pub fn close(&mut self) -> Outcome {
        if self.blocked_by_edit() {
            return Outcome::Blocked;
        }
        self.delete_asked = None;
        self.notice = None;
        self.shown = Shown::Nothing;
        Outcome::Done
    }
}
