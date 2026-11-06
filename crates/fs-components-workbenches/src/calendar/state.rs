//! The state of the calendar workbench, testable without a window.
//!
//! Pattern: State machine over a Strategy (`CalendarSource`): every transition is one method; the view
//! only reads. Going elsewhere is in `navigate.rs`, editing and deleting in `edit.rs` and `delete.rs`. `Clone` is a cheap snapshot (the
//! items are shared), so a view draws from a copy and holds no lock while it builds elements. Nothing
//! here opens an item on its own: only a person's choice does (ADR 0029).

use std::sync::Arc;

use fs_components_widgets::calendar_view::{AGENDA_DAYS, span_of};
use fs_components_widgets::{CalendarItem, Day, Moment, ViewRange};

use super::edit::ItemEdit;
use super::labels::CalendarLabels;
use super::model::CalendarCapabilities;
use super::shown::{Shown, Variant};
use super::source::CalendarSource;
use crate::error::ErrorKind;

/// The calendar goes back no further than this year.
pub const FIRST_YEAR: i32 = 1;
/// The calendar goes no further than this year.
pub const LAST_YEAR: i32 = 9000;

/// View, day, items, the item shown, and the edit in progress.
#[derive(Clone)]
pub struct CalendarState {
    pub(super) source: Arc<dyn CalendarSource>,
    pub(super) labels: Arc<CalendarLabels>,
    pub(super) capabilities: CalendarCapabilities,
    variant: Variant,
    pub(super) view: ViewRange,
    pub(super) anchor: Day,
    week_start: u8,
    now: Moment,
    pub(super) selected_day: Option<Day>,
    items: Arc<Vec<CalendarItem>>,
    problem: Option<String>,
    pub(super) shown: Shown,
    pub(super) notice: Option<String>,
    pub(super) editing: Option<ItemEdit>,
    pub(super) edit_counter: u64,
    pub(super) delete_asked: Option<String>,
}

impl CalendarState {
    /// The workbench over `source`, at today. Nothing is opened.
    pub fn new(source: Arc<dyn CalendarSource>, labels: CalendarLabels, variant: Variant) -> Self {
        let (capabilities, view) = match variant {
            Variant::Compact => (CalendarCapabilities::read_only(), ViewRange::Agenda),
            _ => (source.capabilities(), ViewRange::Month),
        };
        let now = source.now();
        let mut state = Self {
            week_start: source.week_start() % 7,
            source,
            labels: Arc::new(labels),
            capabilities,
            variant,
            view,
            anchor: now.day,
            now,
            selected_day: None,
            items: Arc::new(Vec::new()),
            problem: None,
            shown: Shown::Nothing,
            notice: None,
            editing: None,
            edit_counter: 0,
            delete_asked: None,
        };
        state.reload();
        state
    }

    /// The words.
    pub fn labels(&self) -> &CalendarLabels {
        &self.labels
    }

    /// Full or compact.
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// What is shown: a month, a week, a day or the agenda.
    pub fn view(&self) -> ViewRange {
        self.view
    }

    /// The day the view is around.
    pub fn anchor(&self) -> Day {
        self.anchor
    }

    /// The first day of a week.
    pub fn week_start(&self) -> u8 {
        self.week_start
    }

    /// The moment now, as of the last reading.
    pub fn now(&self) -> Moment {
        self.now
    }

    /// The day chosen last, if any.
    pub fn selected_day(&self) -> Option<Day> {
        self.selected_day
    }

    /// What is on the days shown.
    pub fn items(&self) -> &Arc<Vec<CalendarItem>> {
        &self.items
    }

    /// Why the items could not be read, if they could not.
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    /// First and last day shown.
    pub fn span(&self) -> (Day, Day) {
        span_of(self.view, self.anchor, self.week_start, AGENDA_DAYS)
    }

    /// What the detail area shows.
    pub fn shown(&self) -> &Shown {
        &self.shown
    }

    /// The key of the item selected, if any.
    pub fn selected(&self) -> Option<&str> {
        match &self.shown {
            Shown::Item { key, .. } | Shown::Failed { key, .. } => Some(key),
            Shown::Nothing => None,
        }
    }

    /// A short message for the person, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Read the items of the span again, and the time.
    pub fn reload(&mut self) {
        self.now = self.source.now();
        let (first, last) = self.span();
        match self.source.items(first, last) {
            Ok(items) => {
                self.items = Arc::new(items);
                self.problem = None;
            }
            Err(e) => {
                self.items = Arc::new(Vec::new());
                self.problem = Some(CalendarLabels::fill(&self.labels.items_failed, &e.message));
            }
        }
    }

    /// Show `key` as it is now, ending any edit and any question.
    pub(super) fn show(&mut self, key: &str) {
        self.editing = None;
        self.delete_asked = None;
        self.notice = None;
        self.shown = match self.source.read(key) {
            Ok(sheet) => Shown::Item {
                key: key.to_string(),
                sheet: Box::new(sheet),
            },
            Err(e) if e.kind == ErrorKind::NotFound => {
                self.item_gone();
                return;
            }
            Err(e) => Shown::Failed {
                key: key.to_string(),
                reason: CalendarLabels::fill(&self.labels.unreadable, &e.message),
            },
        };
    }

    /// The item shown no longer exists: it leaves the view and the detail area says so.
    pub(super) fn item_gone(&mut self) {
        self.editing = None;
        self.delete_asked = None;
        self.reload();
        self.shown = Shown::Nothing;
        self.notice = Some(self.labels.gone.clone());
    }
}
