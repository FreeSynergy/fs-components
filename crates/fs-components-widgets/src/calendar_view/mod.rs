//! `calendar-view`: a month, a week, a day or an agenda of items given by the caller.
//!
//! Pattern: Adapter (pure layout in `layout.rs`, then plain Blinc elements) — the widget of ADR 0005:
//! the calendar app embeds exactly this and draws no second calendar. It is neutral: an item is a key,
//! a title, a wall-clock start and end and an all-day flag; the caller has already put them into the
//! zone it shows. It has no words and no colours of its own, and a click reports a value ([`Picked`]).
//!
//! Why not `blinc_cn`'s calendar: that is the date picker of the docs. Its grid is a fixed 252 px, the
//! month and weekday names are English constants, a day holds at most three dots without titles, it
//! knows neither weeks nor days nor times, and it reports only a day.

mod agenda;
mod column;
mod day;
mod item;
mod layout;
mod model;
mod month;
mod position;
mod styles;
mod timeline;
mod view;

pub use day::{Day, MINUTES_PER_DAY, Moment};
pub use layout::{Placed, Segment, lanes, month_days, segment_on, span_of, step_from, week_of};
pub use model::{CalendarItem, CalendarNames, OnPick, Picked, ViewRange};
pub use position::CalendarPosition;
pub use styles::STYLES;
pub use view::{AGENDA_DAYS, CalendarView, NARROW_WIDTH};
