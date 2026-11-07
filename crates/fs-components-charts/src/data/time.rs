//! Reading dates and times written in ISO 8601, as the data wrote them.
//!
//! Pattern: Value object (`Parsed`, valid on construction; `parse` is its only maker, with a private `Cursor` that walks the text) — an instant in milliseconds on the UTC line, plus the offset
//! the text carried. A text without offset is a *wall-clock* time: it is never converted with
//! the machine's zone, so a chart looks the same on every computer.

use super::calendar::{days_from_civil, days_in_month};

/// Milliseconds per minute, hour and day.
pub const MIN: f64 = 60_000.0;
/// Milliseconds per hour.
pub const HOUR: f64 = 3_600_000.0;
/// Milliseconds per day.
pub const DAY: f64 = 86_400_000.0;

/// The zone a time was written in.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    /// No offset in the text: wall-clock time, no conversion.
    Naive,
    /// A fixed offset from UTC in minutes (`Z` is 0).
    Offset(i32),
}

/// A parsed time: milliseconds on the UTC line (for `Naive`: as if UTC) and the zone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parsed {
    /// Milliseconds since 1970-01-01T00:00 (UTC, or wall-clock for a naive time).
    pub ms: f64,
    /// How the text was written.
    pub zone: Zone,
}

/// Parse `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, or a date with `T` (or a space) and `HH:MM[:SS[.fff]]`,
/// optionally followed by `Z`, `+HH:MM`, `+HHMM` or `+HH`. Anything else, and impossible dates
/// (31 February, hour 24, second 60), give `None`.
pub fn parse(text: &str) -> Option<Parsed> {
    let b = text.trim().as_bytes();
    let mut p = Cursor { b, i: 0 };
    let year = i64::from(p.digits(4)?);
    let (mut month, mut day) = (1, 1);
    if p.eat(b'-') {
        month = p.digits(2)?;
        if p.eat(b'-') {
            day = p.digits(2)?;
        }
    }
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let (mut h, mut m, mut s, mut frac) = (0, 0, 0, 0.0);
    if p.eat(b'T') || p.eat(b' ') {
        h = p.digits(2)?;
        p.expect(b':')?;
        m = p.digits(2)?;
        if p.eat(b':') {
            s = p.digits(2)?;
            if p.eat(b'.') {
                frac = p.fraction()?;
            }
        }
        if h > 23 || m > 59 || s > 59 {
            return None;
        }
    }
    let zone = p.zone()?;
    if p.i != b.len() {
        return None;
    }
    let days = days_from_civil(year, month, day);
    #[allow(clippy::cast_precision_loss)] // far below 2^53
    let local =
        days as f64 * DAY + f64::from(h) * HOUR + f64::from(m) * MIN + f64::from(s) * 1000.0 + frac;
    let off = match zone {
        Zone::Naive => 0.0,
        Zone::Offset(o) => f64::from(o) * MIN,
    };
    Some(Parsed {
        ms: local - off,
        zone,
    })
}

struct Cursor<'a> {
    b: &'a [u8],
    i: usize,
}

impl Cursor<'_> {
    fn eat(&mut self, c: u8) -> bool {
        let hit = self.b.get(self.i) == Some(&c);
        self.i += usize::from(hit);
        hit
    }

    fn expect(&mut self, c: u8) -> Option<()> {
        self.eat(c).then_some(())
    }

    fn digits(&mut self, n: usize) -> Option<u32> {
        let s = self.b.get(self.i..self.i + n)?;
        if !s.iter().all(u8::is_ascii_digit) {
            return None;
        }
        self.i += n;
        Some(s.iter().fold(0, |a, d| a * 10 + u32::from(d - b'0')))
    }

    /// Fraction digits to milliseconds (extra digits are cut, not rounded).
    fn fraction(&mut self) -> Option<f64> {
        let start = self.i;
        while self.b.get(self.i).is_some_and(u8::is_ascii_digit) {
            self.i += 1;
        }
        if self.i == start {
            return None;
        }
        let digits = &self.b[start..self.i.min(start + 3)];
        let ms = digits.iter().enumerate().fold(0.0, |a, (k, d)| {
            a + f64::from(d - b'0') * [100.0, 10.0, 1.0][k]
        });
        Some(ms)
    }

    fn zone(&mut self) -> Option<Zone> {
        if self.i == self.b.len() {
            return Some(Zone::Naive);
        }
        if self.eat(b'Z') {
            return Some(Zone::Offset(0));
        }
        let sign = if self.eat(b'+') {
            1
        } else if self.eat(b'-') {
            -1
        } else {
            return None;
        };
        let h = self.digits(2)?;
        self.eat(b':');
        let m = if self.i == self.b.len() {
            0
        } else {
            self.digits(2)?
        };
        if h > 23 || m > 59 {
            return None;
        }
        Some(Zone::Offset(sign * i32::try_from(h * 60 + m).ok()?))
    }
}
