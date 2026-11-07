//! The linear scale: a domain of numbers mapped to a range of pixels.
//!
//! Pattern: Newtype (a validated value: a domain and a range, with the domain rules applied). The mapping halves its terms so that a domain such as
//! [-1.7e308, 1.7e308] does not overflow; the domain rules are those of Vega-Lite: `zero`
//! includes 0, `nice` extends to round values (count 10), an explicit domain is taken as given.

use super::ticks;

/// A linear scale.
#[non_exhaustive]
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
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub struct DomainRule {
    /// Include zero.
    pub zero: bool,
    /// Extend to round values.
    pub nice: bool,
    /// Fixed domain; `zero` and `nice` still apply to it when asked for (as in Vega-Lite).
    pub fixed: Option<[f64; 2]>,
}

impl DomainRule {
    /// A rule.
    pub fn new(zero: bool, nice: bool, fixed: Option<[f64; 2]>) -> Self {
        Self { zero, nice, fixed }
    }
}

/// Pixel positions are kept within this many pixels of the plot: farther is off screen anyway,
/// and a coordinate of 1e300 would break the drawing.
const FAR: f64 = 1e6;

impl Linear {
    /// A scale over the given domain and range.
    pub fn new(d0: f64, d1: f64, r0: f64, r1: f64) -> Self {
        Self { d0, d1, r0, r1 }
    }

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

    /// Pixel position of a value. A one-point domain (or one too narrow to divide by) maps to
    /// the middle of the range; the result is always finite and within a million pixels.
    pub fn map(&self, v: f64) -> f64 {
        let middle = self.r0 / 2.0 + self.r1 / 2.0;
        let span = self.d1 / 2.0 - self.d0 / 2.0;
        if span.is_nan() || span.abs() < f64::MIN_POSITIVE {
            return middle;
        }
        let t = (v / 2.0 - self.d0 / 2.0) / span;
        let r = self.r0 + (self.r1 - self.r0) * t;
        if r.is_finite() {
            r.clamp(-FAR, FAR)
        } else {
            middle
        }
    }

    /// Tick values for about `count` ticks.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks::ticks(self.d0, self.d1, count)
    }
}

/// The domain Vega-Lite would use: the explicit domain or the data extent, then zero, then
/// padding of a one-value data domain, then nice.
pub fn domain(min: f64, max: f64, rule: DomainRule) -> (f64, f64) {
    let (mut lo, mut hi) = match rule.fixed {
        Some([a, b]) => (a, b),
        None => (min.min(max), max.max(min)),
    };
    if rule.zero {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if lo == hi && rule.fixed.is_none() {
        let pad = if lo == 0.0 { 1.0 } else { lo.abs() / 2.0 };
        lo -= pad;
        hi += pad;
    }
    if rule.nice {
        return ticks::nice(lo, hi, 10);
    }
    (lo, hi)
}
