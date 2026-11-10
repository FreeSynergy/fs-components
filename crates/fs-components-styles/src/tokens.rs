//! Sizes and the palette of a style.
//!
//! Pattern: none (plain data, constants, pure functions). All lengths are logical pixels (1/96
//! inch at scale 1), as Blinc takes them.

use crate::color::Rgba;

/// Gaps between things.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    /// Smallest gap (icon to text).
    pub xs: f32,
    /// Small gap (inside a control).
    pub s: f32,
    /// Normal gap (between controls).
    pub m: f32,
    /// Large gap (between groups).
    pub l: f32,
    /// Page padding.
    pub xl: f32,
}

/// Heights of things that repeat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heights {
    /// A one-line control (input, button).
    pub control: f32,
    /// The same, in the dense variant.
    pub control_dense: f32,
    /// A row of a list.
    pub row: f32,
    /// A row of a list, dense.
    pub row_dense: f32,
    /// The row of headings above a list.
    pub head: f32,
}

/// Font sizes and the line height that goes with each (pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Type {
    /// Small print, captions, hints.
    pub caption: f32,
    /// Running text and field content.
    pub body: f32,
    /// Line height of `body`.
    pub body_line: f32,
    /// Section headings.
    pub title: f32,
    /// Page headings.
    pub headline: f32,
    /// Weight of labels (CSS 100..900).
    pub label_weight: u16,
}

/// Corner radii.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    /// Controls (input, button).
    pub control: f32,
    /// Cards and panels.
    pub card: f32,
    /// Windows, sheets, dialogs.
    pub overlay: f32,
}

/// Colours of one appearance (light or dark).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Ground of the window.
    pub ground: Rgba,
    /// Ground of a panel or card.
    pub panel: Rgba,
    /// Ground of a field.
    pub field: Rgba,
    /// Text.
    pub text: Rgba,
    /// Secondary text (hints, captions).
    pub text_weak: Rgba,
    /// Border of a field or panel.
    pub border: Rgba,
    /// Accent (selection, primary action, focus).
    pub accent: Rgba,
    /// Text on the accent.
    pub on_accent: Rgba,
    /// The ring of the control that has the keyboard.
    pub focus: Rgba,
    /// Hover ground of a row or control.
    pub hover: Rgba,
    /// Ground of a selected row.
    pub selected: Rgba,
    /// Error.
    pub error: Rgba,
    /// Success.
    pub success: Rgba,
}

impl Palette {
    /// Pairs of (name, text colour, ground) that must stay readable (WCAG 4.5:1).
    pub fn readable_pairs(&self) -> [(&'static str, Rgba, Rgba); 5] {
        [
            ("text on ground", self.text, self.ground),
            ("text on panel", self.text, self.panel),
            ("text in field", self.text, self.field),
            ("weak text on panel", self.text_weak, self.panel),
            ("text on accent", self.on_accent, self.accent),
        ]
    }
}
