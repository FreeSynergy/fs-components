//! What to draw in a picture, and what comes back.
//!
//! Pattern: none — plain data (`Picture::new` plus assigning fields).

use std::sync::Arc;

use crate::labels::Labels;
use crate::reader::Reader;
use crate::source::DocumentSource;

/// One thing a person does, to be applied before the picture is taken.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Open the document at this path.
    Open(String),
    /// Follow this link in the document shown.
    Follow(String),
    /// Go back.
    Back,
    /// Search for this text.
    Search(String),
    /// Start editing the document shown.
    Edit,
    /// Replace the text being edited.
    Type(String),
    /// Save the edit.
    Save,
    /// Cancel the edit.
    Cancel,
    /// Export the document shown.
    Export,
    /// Scroll the content down by this many pixels (after the layout is known).
    Scroll(u32),
}

/// What the scroll area of the content looked like.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollInfo {
    /// Height of everything inside the scroll area.
    pub content_height: f32,
    /// Height of the visible part.
    pub viewport_height: f32,
    /// How far it is scrolled (0 = top).
    pub offset: f32,
}

/// What [`snapshot`] returns.
#[non_exhaustive]
pub struct Snapshot {
    /// The reader after the steps.
    pub reader: Reader,
    /// The scroll area of the content.
    pub scroll: ScrollInfo,
}

/// What to draw: the source, the words, the steps, the size.
#[non_exhaustive]
pub struct Picture {
    /// The documents.
    pub source: Arc<dyn DocumentSource>,
    /// The words of the interface.
    pub labels: Labels,
    /// Where Export writes to.
    pub export_target: Option<String>,
    /// What the person does first.
    pub steps: Vec<Step>,
    /// Width in pixels (not 0).
    pub width: u32,
    /// Height in pixels (not 0).
    pub height: u32,
}

impl ScrollInfo {
    /// Scroll bounds as read from the area.
    pub fn new(content_height: f32, viewport_height: f32, offset: f32) -> Self {
        Self {
            content_height,
            viewport_height,
            offset,
        }
    }
}

impl Picture {
    /// A picture of `source` with default words, no steps, `width` x `height`.
    pub fn new(source: Arc<dyn DocumentSource>, width: u32, height: u32) -> Self {
        Self {
            source,
            labels: Labels::default(),
            export_target: None,
            steps: Vec::new(),
            width,
            height,
        }
    }
}
