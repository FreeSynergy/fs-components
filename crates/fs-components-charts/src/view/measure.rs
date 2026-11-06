//! Measuring text with Blinc's own text engine, so layout matches what is drawn.
//!
//! Pattern: Adapter — Blinc's `measure_text` behind our `Measure` trait.

use crate::layout::Measure;

/// Measures with the text measurer Blinc has installed (or its estimate before there is one).
#[derive(Clone, Copy, Debug, Default)]
pub struct BlincMeasure;

impl Measure for BlincMeasure {
    fn width(&self, text: &str, px: f32, bold: bool) -> f32 {
        let w = blinc_layout::measure_text(text, px).width;
        // Blinc measures one weight; bold text is a little wider.
        if bold { w * 1.06 } else { w }
    }
}
