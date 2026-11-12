//! A chip: a short word in a coloured pill (a due date, a label, a mark such as BLOCKED).
//!
//! Pattern: Value object + Adapter — the host picks a tone, the sheet decides the colours, so a theme can change
//! them all. A chip is a plain element with no click of its own.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

/// What a chip means; the sheet gives each tone its colours.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// A label.
    #[default]
    Neutral,
    /// Something to look at (today).
    Accent,
    /// Careful: waiting, blocked.
    Warning,
    /// Wrong or late.
    Danger,
    /// Fine: ready.
    Success,
}

impl Tone {
    /// The class of the tone in the sheet.
    pub fn class(self) -> &'static str {
        match self {
            Self::Neutral => "fs-plan-chip-neutral",
            Self::Accent => "fs-plan-chip-accent",
            Self::Warning => "fs-plan-chip-warning",
            Self::Danger => "fs-plan-chip-danger",
            Self::Success => "fs-plan-chip-success",
        }
    }
}

/// One chip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    /// The word.
    pub text: String,
    /// What it means.
    pub tone: Tone,
}

impl Chip {
    /// A chip.
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
}

/// The element of `chip`.
pub fn chip_element(chip: &Chip) -> Div {
    div()
        .class("fs-plan-chip")
        .class(chip.tone.class())
        .child(text(chip.text.clone()).class("fs-plan-chip-text"))
}
