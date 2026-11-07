//! A calendar day and a wall-clock moment.
//!
//! Pattern: none of the table fits (value types, `Day` and `Moment`, with their arithmetic) — days count from 1970-01-01 in the proleptic Gregorian calendar,
//! so arithmetic never passes through a clock time and daylight saving cannot move it. The widget
//! crate takes no date library for this; these few lines are all it needs.

use std::fmt;

/// Minutes in a wall-clock day.
pub const MINUTES_PER_DAY: u16 = 1440;

/// A date in the calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(clippy::struct_field_names)]
pub struct Day {
    year: i32,
    month: u8,
    day: u8,
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 31,
    }
}

impl Day {
    /// This date, if it exists.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }

    /// `YYYY-MM-DD`, if that is a date.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.splitn(3, '-');
        let year = parts.next()?;
        let month = parts.next()?;
        let day = parts.next()?;
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return None;
        }
        Self::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
    }

    /// The year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, from 1.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Days since 1970-01-01 (negative before it).
    pub fn number(self) -> i64 {
        let month = i64::from(self.month);
        let year = i64::from(self.year) - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let of_era = year.rem_euclid(400);
        let of_year = (153 * ((month + 9) % 12) + 2) / 5 + i64::from(self.day) - 1;
        let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
        era * 146_097 + of_cycle - 719_468
    }

    /// The date with this [`Day::number`].
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_number(number: i64) -> Self {
        let shifted = number + 719_468;
        let era = shifted.div_euclid(146_097);
        let of_cycle = shifted.rem_euclid(146_097);
        let of_era = (of_cycle - of_cycle / 1460 + of_cycle / 36_524 - of_cycle / 146_096) / 365;
        let of_year = of_cycle - (365 * of_era + of_era / 4 - of_era / 100);
        let index = (5 * of_year + 2) / 153;
        let month = if index < 10 { index + 3 } else { index - 9 };
        let year = of_era + era * 400 + i64::from(month <= 2);
        Self {
            year: year as i32,
            month: month as u8,
            day: (of_year - (153 * index + 2) / 5 + 1) as u8,
        }
    }

    /// The date `count` days on (back, if negative).
    #[must_use]
    pub fn add_days(self, count: i64) -> Self {
        Self::from_number(self.number() + count)
    }

    /// The same day `count` months on, the last of the month where a month is shorter.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn add_months(self, count: i32) -> Self {
        let months = i64::from(self.year) * 12 + i64::from(self.month) - 1 + i64::from(count);
        let year = months.div_euclid(12) as i32;
        let month = (months.rem_euclid(12) + 1) as u8;
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    /// The first of this month.
    #[must_use]
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// The day of the week: 0 is Monday, 6 is Sunday.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn weekday(self) -> u8 {
        (self.number() + 3).rem_euclid(7) as u8
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A wall-clock time on a day, in whatever zone the caller shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment {
    /// The day.
    pub day: Day,
    /// Minutes since the start of the day, below [`MINUTES_PER_DAY`].
    pub minute: u16,
}

impl Moment {
    /// `hour:minute` on `day`.
    pub fn at(day: Day, hour: u16, minute: u16) -> Self {
        Self {
            day,
            minute: (hour * 60 + minute).min(MINUTES_PER_DAY - 1),
        }
    }

    /// The start of `day`.
    pub fn start_of(day: Day) -> Self {
        Self { day, minute: 0 }
    }

    /// The time of day as `HH:MM`.
    pub fn clock(self) -> String {
        format!("{:02}:{:02}", self.minute / 60, self.minute % 60)
    }
}
