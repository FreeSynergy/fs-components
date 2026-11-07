//! Band and point scales for categories.
//!
//! Pattern: Newtype (a validated value: the counts and paddings are clamped on construction). A band scale gives each category a slot of one step with a gap
//! (`padding_inner`) and space at both ends (`padding_outer`), centred. A point scale is a band
//! scale with no width, as Vega-Lite uses for categories on a line or a point mark.

/// A scale of `n` categories over `[r0, r1]`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    /// Number of categories.
    pub n: usize,
    start: f64,
    step: f64,
    bandwidth: f64,
}

/// Vega-Lite's defaults: bars have inner padding 0.1 and outer padding half of it.
pub const BAR_PADDING_INNER: f64 = 0.1;
/// Outer padding of a point scale.
pub const POINT_PADDING: f64 = 0.5;

impl Band {
    /// A band scale. `inner` is the gap between bands, `outer` the space at both ends, both as
    /// a fraction of a step.
    pub fn new(n: usize, range: (f64, f64), inner: f64, outer: f64) -> Self {
        let (lo, hi) = (range.0.min(range.1), range.0.max(range.1));
        #[allow(clippy::cast_precision_loss)]
        let count = n as f64;
        let steps = (count - inner + 2.0 * outer).max(1.0);
        let step = (hi - lo) / steps;
        let start = lo + (hi - lo - step * (count - inner)) * 0.5;
        Self {
            n,
            start,
            step,
            bandwidth: step * (1.0 - inner),
        }
    }

    /// A band scale for bars.
    pub fn bars(n: usize, range: (f64, f64)) -> Self {
        Self::new(n, range, BAR_PADDING_INNER, BAR_PADDING_INNER / 2.0)
    }

    /// A point scale.
    pub fn points(n: usize, range: (f64, f64)) -> Self {
        Self::new(n, range, 1.0, POINT_PADDING)
    }

    /// Where category `i` starts.
    pub fn start(&self, i: usize) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let i = i as f64;
        self.start + self.step * i
    }

    /// Width of one band (0 for a point scale).
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// The centre of category `i`.
    pub fn centre(&self, i: usize) -> f64 {
        self.start(i) + self.bandwidth / 2.0
    }

    /// Distance between two categories.
    pub fn step(&self) -> f64 {
        self.step
    }
}
