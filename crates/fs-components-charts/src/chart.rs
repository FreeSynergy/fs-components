//! A chart: a spec with its data read and cleaned, ready to lay out and to describe.
//!
//! Pattern: Facade — the one type a program uses without a GUI: read, size, lay out, describe.
//! Everything it returns is plain data (a scene, a text alternative, a report).

use crate::alt::{TextAlternative, text_alternative};
use crate::data::{Prepared, Report, prepare};
use crate::labels::Labels;
use crate::layout::{MAX_SERIES, Measure, Metrics, build, series};
use crate::scene::Scene;
use crate::spec::{Spec, SpecError, parse};

/// Default size of a plot when the spec gives none (Vega-Lite: 300 for continuous axes).
const CONTINUOUS: f32 = 300.0;
/// The same for Vega-Lite 5 (`$schema` v5), where it was 200.
const CONTINUOUS_V5: f32 = 200.0;
/// Default width of one category (Vega-Lite's `step`).
const STEP: f32 = 20.0;

/// A validated, prepared chart.
#[derive(Clone, Debug)]
pub struct Chart {
    spec: Spec,
    data: Prepared,
}

impl Chart {
    /// Read a Vega-Lite spec given as JSON text.
    ///
    /// # Errors
    /// [`SpecError`] if the spec is not valid JSON, uses anything outside the subset, or its
    /// data cannot be put on one line of time or in the colours of the palette.
    pub fn from_json(text: &str) -> Result<Self, SpecError> {
        Self::from_spec(parse(text)?)
    }

    /// Prepare a spec that was read before.
    ///
    /// # Errors
    /// As [`Chart::from_json`], without the JSON error.
    pub fn from_spec(spec: Spec) -> Result<Self, SpecError> {
        let data = prepare(&spec)?;
        let n = series(&spec, &data, &Labels::default()).len();
        if n > MAX_SERIES {
            return Err(SpecError::outside(
                "/encoding/color",
                format!(
                    "{n} colour categories; at most {MAX_SERIES} can be told apart. Aggregate or filter the data first"
                ),
            ));
        }
        Ok(Self { spec, data })
    }

    /// The validated spec.
    pub fn spec(&self) -> &Spec {
        &self.spec
    }

    /// What was dropped, broken or odd in the data.
    pub fn report(&self) -> &Report {
        &self.data.report
    }

    /// The prepared data (what is drawn).
    pub fn data(&self) -> &Prepared {
        &self.data
    }

    /// The names of the colour categories (series), in legend order; empty without colour.
    pub fn series_names(&self, labels: &Labels) -> Vec<String> {
        series(&self.spec, &self.data, labels).names
    }

    /// Lay the chart out in a box of `width` x `height` pixels.
    pub fn scene(
        &self,
        width: f32,
        height: f32,
        labels: &Labels,
        measure: &dyn Measure,
        metrics: &Metrics,
    ) -> Scene {
        build(
            &self.spec,
            &self.data,
            labels,
            measure,
            metrics,
            width.max(1.0),
            height.max(1.0),
        )
    }

    /// The size the spec asks for, plot plus axes, legend and title: `width` and `height` of the
    /// spec if given, else 300 pixels for a number or time axis and 20 pixels per category.
    pub fn natural_size(
        &self,
        labels: &Labels,
        measure: &dyn Measure,
        metrics: &Metrics,
    ) -> (f32, f32) {
        let enc = &self.spec.encoding;
        let cats = |d: &Option<crate::spec::ChannelDef>, ch| {
            d.as_ref()
                .filter(|d| d.ftype.is_discrete())
                .map(|d| crate::layout::categories(d, ch, &self.data.data, labels, None).len())
        };
        let continuous = if self.spec.schema_major == 5 {
            CONTINUOUS_V5
        } else {
            CONTINUOUS
        };
        let step = |n: Option<usize>, asked: Option<f64>| {
            asked.map_or_else(
                || n.map_or(continuous, |n| n.max(1) as f32 * STEP),
                |v| v as f32,
            )
        };
        let (pw, ph) = (
            step(cats(&enc.x, crate::spec::Channel::X), self.spec.width),
            step(cats(&enc.y, crate::spec::Channel::Y), self.spec.height),
        );
        // the margins depend on the labels, which depend on the plot size: two rounds settle it
        let mut extra = (300.0, 200.0);
        for _ in 0..2 {
            let probe = self.scene(pw + extra.0, ph + extra.1, labels, measure, metrics);
            extra = (probe.width - probe.plot[2], probe.height - probe.plot[3]);
        }
        (pw + extra.0, ph + extra.1)
    }

    /// The text alternative: a summary and the drawn data as a table of at most `max_rows` rows.
    pub fn text_alternative(&self, labels: &Labels, max_rows: usize) -> TextAlternative {
        text_alternative(&self.spec, &self.data, labels, max_rows)
    }
}
