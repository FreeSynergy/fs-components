//! Calendar days and moments as plain numbers.
//!
//! Pattern: Value object — a [`Day`] is a count of days since 1970-01-01 (proleptic Gregorian), a
//! [`Moment`] a day plus minutes. No time zone: the caller hands in already localised values.

/// Minutes in a day.
pub const MINUTES_PER_DAY: u16 = 1440;

/// A day, as days since 1970-01-01.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day(pub i64);

impl Day {
    /// The day of a civil date (month 1..=12, day 1..=31).
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Self {
        let y = i64::from(year) - i64::from(month <= 2);
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let m = i64::from(month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Day(era * 146_097 + doe - 719_468)
    }

    /// Year, month (1..=12), day of month.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // ranges are bounded by the algorithm
    pub fn ymd(self) -> (i32, u8, u8) {
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let y = (yoe + era * 400 + i64::from(m <= 2)) as i32;
        (y, m, d)
    }

    /// Weekday, 0 is Monday .. 6 is Sunday.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn weekday(self) -> u8 {
        (self.0 + 3).rem_euclid(7) as u8
    }

    /// This day plus `n` days (minus when negative).
    #[must_use]
    pub fn add_days(self, n: i64) -> Self {
        Day(self.0 + n)
    }

    /// Days in the month of this day.
    pub fn days_in_month(self) -> u8 {
        let (y, m, _) = self.ymd();
        let next = if m == 12 {
            Day::from_ymd(y + 1, 1, 1)
        } else {
            Day::from_ymd(y, m + 1, 1)
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (next.0 - Day::from_ymd(y, m, 1).0) as u8;
        n
    }

    /// The first day of this day's month.
    #[must_use]
    pub fn first_of_month(self) -> Self {
        let (y, m, _) = self.ymd();
        Day::from_ymd(y, m, 1)
    }

    /// `n` months on, the day of month kept or cut to the month's end.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn add_months(self, n: i32) -> Self {
        let (y, m, d) = self.ymd();
        let total = y * 12 + i32::from(m) - 1 + n;
        let (ny, nm) = (total.div_euclid(12), (total.rem_euclid(12) + 1) as u8);
        let last = Day::from_ymd(ny, nm, 1).days_in_month();
        Day::from_ymd(ny, nm, d.min(last))
    }
}

/// A moment: a day and the minute of that day (0..1440), in the zone the caller works in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment {
    /// The day.
    pub day: Day,
    /// Minutes since midnight.
    pub minute: u16,
}

impl Moment {
    /// A moment on `day` at `minute`.
    pub fn new(day: Day, minute: u16) -> Self {
        Self { day, minute }
    }
}
