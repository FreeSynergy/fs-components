//! The chart as a widget: a Blinc canvas that draws the scene.
//!
//! Pattern: Adapter + Facade — `ChartView` holds the chart and the caller's labels, and returns a
//! Blinc element (behind `impl Element`) of the size the caller asks for. The layout is made when
//! the canvas first draws at a size and kept until the size changes.

use std::rc::Rc;
use std::sync::Arc;

use blinc_layout::canvas::canvas;
use blinc_layout::prelude::{div, text};

use super::draw::draw;
use super::look::Look;
use super::measure::BlincMeasure;
use super::rotated;
use crate::Element;
use crate::alt::TextAlternative;
use crate::chart::Chart;
use crate::labels::Labels;
use crate::scene::{Paint, Scene, TextSize};

/// How many rows the table of the text alternative lists by default.
pub const ALT_ROWS: usize = 200;

/// A chart ready to be placed in a layout.
#[derive(Clone)]
pub struct ChartView {
    chart: Arc<Chart>,
    labels: Labels,
}

impl ChartView {
    /// A view of `chart` with the caller's words.
    pub fn new(chart: Chart, labels: Labels) -> Self {
        Self {
            chart: Arc::new(chart),
            labels,
        }
    }

    /// The chart.
    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    /// The text alternative (summary and data table), for a screen reader or a "show data" view.
    pub fn text_alternative(&self) -> TextAlternative {
        self.chart.text_alternative(&self.labels, ALT_ROWS)
    }

    /// The size the spec asks for, with the theme's text sizes.
    pub fn natural_size(&self) -> (f32, f32) {
        let look = Look::current();
        self.chart
            .natural_size(&self.labels, &BlincMeasure, &look.metrics)
    }

    /// The layout at a size, with the theme's text sizes (for tests and hit testing).
    pub fn scene(&self, width: f32, height: f32) -> Scene {
        let look = Look::current();
        self.chart
            .scene(width, height, &self.labels, &BlincMeasure, &look.metrics)
    }

    /// The element of `width` x `height` pixels: a canvas, and the rotated labels over it.
    ///
    /// The layout is made now, with the theme's text sizes and the text measurer Blinc has
    /// installed; build again when the theme or the size changes.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let look = Look::current();
        let mut root = div().relative().w(width).h(height);
        let wanted = self.chart.series_names(&self.labels).len();
        if wanted > look.series.len() {
            let reason = format!(
                "{wanted} colour categories, but the theme offers {} colours that can be told apart",
                look.series.len()
            );
            let words = Labels::fill(&self.labels.cannot_draw, &[("reason", &reason)]);
            return root.child(
                text(words)
                    .size(look.px(TextSize::Label))
                    .color(look.colour(Paint::Muted)),
            );
        }
        let scene = Rc::new(self.scene(width, height));
        let drawn = Rc::clone(&scene);
        root = root.child(
            canvas(move |ctx, _| draw(ctx, &drawn, &Look::current(), &BlincMeasure))
                .redraw_while(|| false)
                .w(width)
                .h(height),
        );
        for e in rotated::elements(&scene, &look, &BlincMeasure) {
            root = root.child(e);
        }
        root
    }
}

/// The element shown in place of a chart that cannot be drawn: the reason, in the caller's words.
pub fn notice(
    labels: &Labels,
    reason: &str,
    width: f32,
    height: f32,
    look: &Look,
) -> impl Element + use<> {
    let words = Labels::fill(&labels.cannot_draw, &[("reason", reason)]);
    div().w(width).h(height).child(
        text(words)
            .size(look.px(TextSize::Label))
            .color(look.colour(Paint::Muted)),
    )
}
