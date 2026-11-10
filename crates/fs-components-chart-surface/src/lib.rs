//! `fs-components-chart-surface`: a screen of the window shell (ADR 0065) that shows a chart.
//!
//! A program (history: entries per day, digest per kind) owns data and the trait [`ChartSource`]; it gives a Vega-Lite
//! spec as text, and this crate draws it with `fs-components-charts` (ADR 0066: no render code in a program repo). The
//! spec is read again each time the screen is built, so a refreshed source shows new data at the next draw.
//!
//! Pattern: Strategy (the source is behind a trait) + Adapter (to the shell's `Surface`).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

use std::sync::Arc;

use blinc_layout::prelude::{div, text};
use fs_components_app::Surface;
use fs_components_charts::{Chart, ChartView, Labels};
use fs_components_widgets::Element;

/// What a program gives to a chart screen: the chart as a Vega-Lite spec (the validated subset of
/// `fs-components-charts`), as text.
pub trait ChartSource: Send + Sync {
    /// The spec now, with the data in it.
    fn spec(&self) -> String;
}

/// A chart in the window shell.
pub struct ChartSurface {
    source: Arc<dyn ChartSource>,
    labels: Labels,
}

impl ChartSurface {
    /// A chart screen over `source`, with the caller's words for chart notes.
    pub fn new(source: Arc<dyn ChartSource>, labels: Labels) -> Self {
        Self { source, labels }
    }

    /// The chart for the source as it is now.
    ///
    /// # Errors
    /// The reason (English, for the log) when the spec is outside the subset or its data cannot be drawn.
    pub fn view(&self) -> Result<ChartView, String> {
        Chart::from_json(&self.source.spec())
            .map(|chart| ChartView::new(chart, self.labels.clone()))
            .map_err(|e| e.to_string())
    }
}

impl Surface for ChartSurface {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        match self.view() {
            Ok(view) => Box::new(view.build(width, height)),
            Err(reason) => Box::new(div().w(width).h(height).child(text(Labels::fill(
                &self.labels.cannot_draw,
                &[("reason", &reason)],
            )))),
        }
    }
}
