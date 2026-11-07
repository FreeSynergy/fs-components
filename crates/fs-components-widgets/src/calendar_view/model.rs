//! What the calendar view shows and what it reports.
//!
//! Pattern: none — neutral items and the words the caller gives; the widget does not know
//! what an item stands for.

use std::sync::Arc;

use super::day::{Day, Moment};

/// Which span of time is shown.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ViewRange {
    /// A month: six weeks of days.
    #[default]
    Month,
    /// A week on a time axis.
    Week,
    /// One day on a time axis.
    Day,
    /// A list by day.
    Agenda,
}

/// A thing on the calendar. Times are wall-clock in the zone the caller shows; `end` is exclusive.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarItem {
    /// Names the item for the caller; opaque to the widget. Reported when it is clicked.
    pub key: String,
    /// What it is called.
    pub title: String,
    /// Where it begins.
    pub start: Moment,
    /// Where it ends; not before `start`.
    pub end: Moment,
    /// Without a time: shown above the axis.
    pub all_day: bool,
    /// Short markings to be seen wherever the item appears, in the caller's words.
    pub marks: Vec<String>,
    /// Which of the six category colours of the theme, 0 to 5.
    pub category: u8,
}

impl CalendarItem {
    /// An item with a time.
    pub fn timed(
        key: impl Into<String>,
        title: impl Into<String>,
        start: Moment,
        end: Moment,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            start,
            end: end.max(start),
            all_day: false,
            marks: Vec::new(),
            category: 0,
        }
    }

    /// An item without a time, from `first` through `last` (both shown).
    pub fn all_day(
        key: impl Into<String>,
        title: impl Into<String>,
        first: Day,
        last: Day,
    ) -> Self {
        let mut item = Self::timed(
            key,
            title,
            Moment::start_of(first),
            Moment::start_of(last.max(first).add_days(1)),
        );
        item.all_day = true;
        item
    }

    /// With this marking.
    #[must_use]
    pub fn marked(mut self, mark: impl Into<String>) -> Self {
        self.marks.push(mark.into());
        self
    }

    /// In this category colour (taken modulo six).
    #[must_use]
    pub fn in_category(mut self, category: u8) -> Self {
        self.category = category % 6;
        self
    }
}

/// The words of the calendar view, all from the caller.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarNames {
    /// Short names of the days, Monday first.
    pub weekdays: [String; 7],
    /// Names of the months, January first.
    pub months: [String; 12],
    /// Says an item has no time.
    pub all_day: String,
    /// Shown when there is nothing to show.
    pub empty: String,
    /// How more items than fit are counted; `{}` = how many.
    pub more: String,
    /// A date; `{weekday}`, `{day}`, `{month}` and `{year}` are filled in.
    pub date: String,
}

impl CalendarNames {
    /// These words, with `+{}` for the count and `{weekday} {day} {month} {year}` for a date.
    pub fn new(
        weekdays: [String; 7],
        months: [String; 12],
        all_day: impl Into<String>,
        empty: impl Into<String>,
    ) -> Self {
        Self {
            weekdays,
            months,
            all_day: all_day.into(),
            empty: empty.into(),
            more: "+{}".into(),
            date: "{weekday} {day} {month} {year}".into(),
        }
    }

    /// `day` in words.
    pub fn date_text(&self, day: Day) -> String {
        self.date
            .replace("{weekday}", &self.weekdays[usize::from(day.weekday())])
            .replace("{day}", &day.day().to_string())
            .replace("{month}", &self.months[usize::from(day.month()) - 1])
            .replace("{year}", &day.year().to_string())
    }

    /// The name of the month of `day`.
    pub fn month_of(&self, day: Day) -> &str {
        &self.months[usize::from(day.month()) - 1]
    }
}

/// What a click reports.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Picked {
    /// A day.
    Day(Day),
    /// The count of a day's items that did not fit.
    More(Day),
    /// An item, by its key.
    Item(String),
}

/// Receives what was clicked.
pub type OnPick = Arc<dyn Fn(Picked) + Send + Sync>;
