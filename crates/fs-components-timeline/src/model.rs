//! The plain data of the timeline: items, dependencies, zoom.
//!
//! Pattern: Value object — small immutable types the host fills; days are already in the zone the
//! reader sees (`Day` and `Tone` are those of `fs-components-calendar-grid`, same rev).

pub use fs_components_calendar_grid::{Day, Tone};

/// What an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A bar from `start` to `end` (both days inclusive), `progress` percent done (0..=100).
    Bar {
        /// First day.
        start: Day,
        /// Last day.
        end: Day,
        /// Percent done.
        progress: u8,
    },
    /// A mark on one day.
    Milestone {
        /// The day.
        day: Day,
    },
}

/// One row of the timeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// Identity, handed back by `select_item`.
    pub key: String,
    /// Title, already text.
    pub title: String,
    /// Bar or milestone.
    pub kind: Kind,
    /// Colour slot.
    pub tone: Tone,
}

impl Item {
    /// A bar.
    pub fn bar(key: &str, title: &str, start: Day, end: Day, progress: u8) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            kind: Kind::Bar {
                start,
                end: end.max(start),
                progress: progress.min(100),
            },
            tone: Tone::Accent,
        }
    }

    /// A milestone.
    pub fn milestone(key: &str, title: &str, day: Day) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            kind: Kind::Milestone { day },
            tone: Tone::Accent,
        }
    }

    /// The same item in another colour slot.
    #[must_use]
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// First and last day the item covers.
    pub fn span(&self) -> (Day, Day) {
        match self.kind {
            Kind::Bar { start, end, .. } => (start, end),
            Kind::Milestone { day } => (day, day),
        }
    }
}

/// `to` can start only after `from` ends (finish to start); drawn as an arrow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    /// Key of the item that comes first.
    pub from: String,
    /// Key of the item that follows.
    pub to: String,
}

/// How much one day is worth in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zoom {
    /// One day is wide enough to name.
    Day,
    /// Weeks are named.
    Week,
    /// Months are named.
    Month,
}

impl Zoom {
    /// All levels, fine to coarse.
    pub const ALL: [Zoom; 3] = [Zoom::Day, Zoom::Week, Zoom::Month];

    /// Name for translation keys and element ids.
    pub fn name(self) -> &'static str {
        match self {
            Zoom::Day => "day",
            Zoom::Week => "week",
            Zoom::Month => "month",
        }
    }

    /// Pixels per day.
    pub fn px_per_day(self) -> f32 {
        match self {
            Zoom::Day => 40.0,
            Zoom::Week => 12.0,
            Zoom::Month => 4.0,
        }
    }

    /// One level finer (or the same at the end).
    #[must_use]
    pub fn finer(self) -> Self {
        match self {
            Zoom::Month => Zoom::Week,
            _ => Zoom::Day,
        }
    }

    /// One level coarser (or the same at the end).
    #[must_use]
    pub fn coarser(self) -> Self {
        match self {
            Zoom::Day => Zoom::Week,
            _ => Zoom::Month,
        }
    }
}
