//! The linear scale: a domain of numbers mapped to a range of pixels.
//!
//! Pattern: Value object. The mapping halves its terms so that a domain such as
//! [-1.7e308, 1.7e308] does not overflow; the domain rules are those of Vega-Lite: `zero`
//! includes 0, `nice` extends to round values (count 10), an explicit domain is taken as given.

use super::ticks;

/// A linear scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    /// Domain start.
    pub d0: f64,
    /// Domain end.
    pub d1: f64,
    /// Range start (pixels).
    pub r0: f64,
    /// Range end (pixels).
    pub r1: f64,
}

/// How to derive a domain from data.
#[derive(Clone, Copy, Debug)]
pub struct DomainRule {
    /// Include zero.
    pub zero: bool,
    /// Extend to round values.
    pub nice: bool,
    /// Fixed domain; nothing else applies.
    pub fixed: Option<[f64; 2]>,
}

impl Linear {
    /// A scale for a domain found with `rule` from the data's `min` and `max`.
    pub fn from_data(min: f64, max: f64, rule: DomainRule, range: (f64, f64)) -> Self {
        let (d0, d1) = domain(min, max, rule);
        Self {
            d0,
            d1,
            r0: range.0,
            r1: range.1,
        }
    }

    /// Pixel position of a value. A one-point domain maps to the middle of the range.
    pub fn map(&self, v: f64) -> f64 {
        let span = self.d1 / 2.0 - self.d0 / 2.0;
        if span == 0.0 {
            return self.r0 / 2.0 + self.r1 / 2.0;
        }
        let t = (v / 2.0 - self.d0 / 2.0) / span;
        self.r0 + (self.r1 - self.r0) * t
    }

    /// Tick values for about `count` ticks.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks::ticks(self.d0, self.d1, count)
    }
}

/// The domain Vega-Lite would use: data extent, then zero, then padding of a one-value domain,
/// then nice.
pub fn domain(min: f64, max: f64, rule: DomainRule) -> (f64, f64) {
    if let Some([a, b]) = rule.fixed {
        return (a, b);
    }
    let (mut lo, mut hi) = (min.min(max), max.max(min));
    if rule.zero {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if lo == hi {
        let pad = if lo == 0.0 { 1.0 } else { lo.abs() / 2.0 };
        lo -= pad;
        hi += pad;
    }
    if rule.nice {
        return ticks::nice(lo, hi, 10);
    }
    (lo, hi)
}
