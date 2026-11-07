//! Round tick values for a number axis: steps of 1, 2 or 5 times a power of ten.
//!
//! Where the thresholds come from, plainly: choosing a step of 1, 2, 5 or 10 times a power of ten
//! with the cut points sqrt(2), sqrt(10) and sqrt(50) (the geometric means between 1, 2, 5, 10) is
//! how d3's tick function does it. The author of this code, an AI model, wrote it from memory of
//! that rule, not by reading or pasting the library's source in this session; it may reflect the
//! source it learned from. Whether that is a copy in the sense of ADR 0013 is for the owner to
//! judge. The results are checked against Vega-Lite 6.5.0 in `tests/golden`, and the step could be
//! replaced by any other way of choosing round steps.
//!
//! Pattern: none — a `Step` and pure functions. A step is kept as `mult` and `pow` so that
//! ticks like 0.3 are computed as 3 / 10, the closest double to the decimal, not 0.1 + 0.1 + 0.1.

/// A tick step `mult * 10^pow`, `mult` in {1, 2, 5, 10}.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    mult: f64,
    pow: i32,
}

impl Step {
    /// The value of tick number `k` (an integer count of steps).
    pub fn value(self, k: f64) -> f64 {
        if self.pow >= 0 {
            k * self.mult * 10f64.powi(self.pow)
        } else {
            k * self.mult / 10f64.powi(-self.pow)
        }
    }

    /// The step as a number.
    pub fn size(self) -> f64 {
        self.value(1.0)
    }

    /// Number of decimals a label needs to tell ticks of this step apart.
    pub fn decimals(self) -> usize {
        usize::try_from(-self.pow).unwrap_or(0)
    }

    /// The step number at or below / above a value.
    fn floor(self, v: f64) -> f64 {
        (v / self.size()).floor()
    }

    fn ceil(self, v: f64) -> f64 {
        (v / self.size()).ceil()
    }
}

/// The step that gives about `count` ticks between `start` and `stop` (`start < stop`).
pub fn step_for(start: f64, stop: f64, count: usize) -> Option<Step> {
    #[allow(clippy::cast_precision_loss)]
    let n = count.max(1) as f64;
    let raw = if (stop - start).is_finite() {
        (stop - start) / n
    } else {
        stop / n - start / n
    };
    if !(raw.is_finite() && raw > 0.0) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    let pow = raw.log10().floor() as i32;
    if pow < -300 {
        return None; // steps this small are below what a double can tell apart
    }
    let err = raw / 10f64.powi(pow);
    let mult = if err >= 50f64.sqrt() {
        10.0
    } else if err >= 10f64.sqrt() {
        5.0
    } else if err >= 2f64.sqrt() {
        2.0
    } else {
        1.0
    };
    Some(Step { mult, pow })
}

/// Round tick values inside `[start, stop]`, about `count` of them. `start == stop` gives that
/// one value; an infinite or reversed range gives none.
pub fn ticks(start: f64, stop: f64, count: usize) -> Vec<f64> {
    if start == stop && start.is_finite() {
        return vec![start];
    }
    let Some(step) = step_for(start, stop, count) else {
        return Vec::new();
    };
    let (k0, k1) = (step.ceil(start), step.floor(stop));
    let mut out = Vec::new();
    let mut k = k0;
    while k <= k1 && out.len() < 10_000 {
        out.push(step.value(k) + 0.0);
        k += 1.0;
    }
    out
}

/// Extend `[start, stop]` outwards to round values, as a scale's `nice` does. Returns the input
/// when no round extension fits in a double.
pub fn nice(start: f64, stop: f64, count: usize) -> (f64, f64) {
    let (mut lo, mut hi) = (start, stop);
    let mut last = None;
    for _ in 0..10 {
        let Some(step) = step_for(lo, hi, count) else {
            break;
        };
        if last == Some(step) {
            break;
        }
        let (a, b) = (step.value(step.floor(lo)), step.value(step.ceil(hi)));
        if !(a.is_finite() && b.is_finite()) {
            break;
        }
        (lo, hi) = (a, b);
        last = Some(step);
    }
    (lo + 0.0, hi + 0.0)
}
