//! The widget: what it is given and how it picks the drawing.
//!
//! Pattern: Adapter, part of `CalendarView` (the others are `layout.rs`, `agenda.rs`, `day.rs`): the widget turns what it was given into Blinc elements; no lock and no `stateful`: it draws what it was given, and
//! whoever holds it builds it again after a change. The one reactive part is the agenda's window of lines
//! in view, a `Computed` over the scroll offset kept in the position: it says which lines exist, while
//! what a line shows comes from the items of this build. Below [`NARROW_WIDTH`] a month or a week
//! becomes the agenda (docs: on narrow screens).

use std::sync::Arc;

use blinc_layout::div::Div;

use super::agenda::agenda;
use super::column::HOUR_HEIGHT;
use super::day::{Day, Moment};
use super::item::Drawing;
use super::layout::{span_of, week_of};
use super::model::{CalendarItem, CalendarNames, OnPick, ViewRange};
use super::month::month;
use super::position::CalendarPosition;
use super::timeline::{Axis, timeline};

/// How many days an agenda shows.
pub const AGENDA_DAYS: u16 = 30;
/// Narrower than this a month or a week is drawn as the agenda.
pub const NARROW_WIDTH: f32 = 420.0;

/// A calendar: range, day and items from the caller. See the module docs.
pub struct CalendarView {
    range: ViewRange,
    anchor: Day,
    week_start: u8,
    now: Option<Moment>,
    today: Option<Day>,
    selected_day: Option<Day>,
    selected_item: Option<String>,
    items: Arc<Vec<CalendarItem>>,
    names: Arc<CalendarNames>,
    name: Option<String>,
    on_pick: Option<OnPick>,
    position: CalendarPosition,
    start_hour: Option<u16>,
}

impl CalendarView {
    /// `range` around `anchor`, with these words and items. The week starts on Monday, nothing is
    /// selected, and nothing is today.
    pub fn new(
        range: ViewRange,
        anchor: Day,
        names: Arc<CalendarNames>,
        items: Arc<Vec<CalendarItem>>,
    ) -> Self {
        Self {
            range,
            anchor,
            week_start: 0,
            now: None,
            today: None,
            selected_day: None,
            selected_item: None,
            items,
            names,
            name: None,
            on_pick: None,
            position: CalendarPosition::new(),
            start_hour: None,
        }
    }

    /// The first day of a week: 0 is Monday, 6 is Sunday.
    #[must_use]
    pub fn week_start(mut self, weekday: u8) -> Self {
        self.week_start = weekday % 7;
        self
    }

    /// Today, marked as such.
    #[must_use]
    pub fn today(mut self, today: Day) -> Self {
        self.today = Some(today);
        self
    }

    /// The moment now: a line across the axis, if its day is shown.
    #[must_use]
    pub fn now(mut self, now: Moment) -> Self {
        self.now = Some(now);
        self.today = Some(now.day);
        self
    }

    /// The day shown as selected.
    #[must_use]
    pub fn selected_day(mut self, day: Option<Day>) -> Self {
        self.selected_day = day;
        self
    }

    /// The item shown as selected, by key.
    #[must_use]
    pub fn selected_item(mut self, key: Option<String>) -> Self {
        self.selected_item = key;
        self
    }

    /// Element ids begin with this name: `<name>-day-<date>`, `<name>-item-<key>@<date>`,
    /// `<name>-more-<date>`, `<name>-head-<date>`.
    #[must_use]
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Who hears of a click.
    #[must_use]
    pub fn on_pick(mut self, on_pick: OnPick) -> Self {
        self.on_pick = Some(on_pick);
        self
    }

    /// Keeps the scroll position across builds. The owner keeps the handle.
    #[must_use]
    pub fn position(mut self, position: &CalendarPosition) -> Self {
        self.position = position.clone();
        self
    }

    /// The hour at the top of the axis the first time a position is used here.
    #[must_use]
    pub fn start_hour(mut self, hour: u16) -> Self {
        self.start_hour = Some(hour.min(23));
        self
    }

    /// The range drawn at this width.
    pub fn range_at(&self, width: f32) -> ViewRange {
        if width < NARROW_WIDTH && matches!(self.range, ViewRange::Month | ViewRange::Week) {
            ViewRange::Agenda
        } else {
            self.range
        }
    }

    /// The element, filling `width` x `height`.
    pub fn build(&self, width: f32, height: f32) -> Div {
        let size = (width.max(0.0), height.max(0.0));
        let drawing = Drawing {
            names: Arc::clone(&self.names),
            name: self.name.clone(),
            on_pick: self.on_pick.clone(),
            selected_item: self.selected_item.clone(),
            selected_day: self.selected_day,
            today: self.today,
        };
        let items = self.items.as_slice();
        match self.range_at(width) {
            ViewRange::Week => {
                let first = week_of(self.anchor, self.week_start);
                let days: Vec<Day> = (0..7).map(|i| first.add_days(i)).collect();
                timeline(&drawing, items, &days, &self.axis(), size)
            }
            ViewRange::Agenda => {
                // A narrow month or week lists the same span the caller has items for.
                let span = span_of(self.range, self.anchor, self.week_start, AGENDA_DAYS);
                agenda(
                    &drawing,
                    &self.items,
                    span,
                    self.position.list(span.0),
                    size,
                )
            }
            ViewRange::Month => month(&drawing, items, self.anchor, self.week_start, size),
            _ => timeline(&drawing, items, &[self.anchor], &self.axis(), size),
        }
    }

    fn axis(&self) -> Axis<'_> {
        let offset = self.start_hour.map(|hour| f32::from(hour) * HOUR_HEIGHT);
        Axis {
            position: self.position.axis(offset),
            now: self.now,
        }
    }
}
