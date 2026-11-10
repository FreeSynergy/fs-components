//! The plain data of the grid: items, ranges, slots, messages.
//!
//! Pattern: Value object — small immutable types the host fills; no program or JSCalendar type
//! appears here, times are already in the zone the reader sees.

use crate::day::{Day, Moment};

/// Which grid is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Six weeks of day cells.
    Month,
    /// Seven day columns with a time axis.
    Week,
    /// One day column with a time axis.
    Day,
}

impl Mode {
    /// All modes, in the order of the toolbar.
    pub const ALL: [Mode; 3] = [Mode::Month, Mode::Week, Mode::Day];

    /// Name used in translation keys and element ids.
    pub fn name(self) -> &'static str {
        match self {
            Mode::Month => "month",
            Mode::Week => "week",
            Mode::Day => "day",
        }
    }
}

/// Colour slot of an item (the style decides the colour).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    /// The accent colour.
    #[default]
    Accent,
    /// Success colour.
    Success,
    /// Error colour.
    Error,
    /// Neutral, weak.
    Neutral,
}

/// One thing on the calendar. All-day items cover `start.day..=end.day`; timed items end at
/// `end` (exclusive), so one ending at midnight does not touch the next day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridItem {
    /// Identity, handed back by `select_item`.
    pub key: String,
    /// Title, already text.
    pub title: String,
    /// Start.
    pub start: Moment,
    /// End.
    pub end: Moment,
    /// Covers whole days.
    pub all_day: bool,
    /// Colour slot.
    pub tone: Tone,
}

impl GridItem {
    /// A timed item.
    pub fn timed(key: &str, title: &str, start: Moment, end: Moment) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            start,
            end,
            all_day: false,
            tone: Tone::Accent,
        }
    }

    /// An all-day item over `first..=last`.
    pub fn all_day(key: &str, title: &str, first: Day, last: Day) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            start: Moment::new(first, 0),
            end: Moment::new(last, 0),
            all_day: true,
            tone: Tone::Accent,
        }
    }

    /// The same item in another colour slot.
    #[must_use]
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// The last day the item touches.
    pub fn last_day(&self) -> Day {
        if self.all_day || self.end.minute > 0 || self.end.day == self.start.day {
            self.end.day.max(self.start.day)
        } else {
            self.end.day.add_days(-1)
        }
    }

    /// Lasts more than one day, or covers whole days: drawn as a bar, not in a time column.
    pub fn is_span(&self) -> bool {
        self.all_day || self.last_day() > self.start.day
    }
}

/// A place the reader chose: a day, and a minute when it was on a time axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The day.
    pub day: Day,
    /// Minute of the day; `None` for a day cell or the all-day row.
    pub minute: Option<u16>,
}

/// A navigation message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    /// One step back in the current mode.
    Previous,
    /// One step on.
    Next,
    /// Jump to today.
    Today,
}
