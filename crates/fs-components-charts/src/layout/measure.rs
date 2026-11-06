//! What the layout needs to know about text, and the sizes it uses.
//!
//! Pattern: Strategy (`Measure`) + Value object (`Metrics`). The view measures with Blinc's text
//! engine; tests measure with a fixed advance, so layout numbers are exact and reproducible.

use crate::scene::TextSize;

/// Width of a text in pixels.
pub trait Measure {
    /// The width of `text` at `px` pixels size.
    fn width(&self, text: &str, px: f32, bold: bool) -> f32;
}

/// Every character is `0.6 * size` wide (tests and the text-only fallback).
#[derive(Clone, Copy, Debug, Default)]
pub struct Fixed;

impl Measure for Fixed {
    fn width(&self, text: &str, px: f32, _bold: bool) -> f32 {
        text.chars().count() as f32 * px * 0.6
    }
}

/// Pixel sizes of chart furniture. The view reads the text sizes from the theme.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Chart title.
    pub title: f32,
    /// Axis and legend titles.
    pub axis_title: f32,
    /// Labels.
    pub label: f32,
    /// Length of a tick.
    pub tick: f32,
    /// Space between tick and label.
    pub label_gap: f32,
    /// Space between labels and title.
    pub title_gap: f32,
    /// Space around the chart.
    pub pad: f32,
}

impl Default for Metrics {
    /// Vega-Lite's sizes: title 13, axis title 11, labels 10, ticks 5.
    fn default() -> Self {
        Self {
            title: 13.0,
            axis_title: 11.0,
            label: 10.0,
            tick: 5.0,
            label_gap: 2.0,
            title_gap: 4.0,
            pad: 5.0,
        }
    }
}

impl Metrics {
    /// Pixel size of a text class.
    pub fn px(&self, size: TextSize) -> f32 {
        match size {
            TextSize::Title => self.title,
            TextSize::AxisTitle => self.axis_title,
            TextSize::Label => self.label,
        }
    }

    /// Line height of a text class.
    pub fn line(&self, size: TextSize) -> f32 {
        (self.px(size) * 1.25).ceil()
    }
}
