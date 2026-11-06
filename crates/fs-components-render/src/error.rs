//! What can go wrong when drawing a picture.
//!
//! Pattern: Value object — a plain error enum; the variants let a test tell "no graphics adapter
//! on this machine" from a real failure.

use std::fmt;

/// Why a picture could not be drawn.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// Width or height is 0.
    EmptyCanvas,
    /// No graphics adapter could be started.
    NoGraphics(String),
    /// The style sheet does not parse.
    Stylesheet(String),
    /// Drawing or reading back the picture failed.
    Draw(String),
    /// The file could not be written.
    Write(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCanvas => f.write_str("the picture needs a width and a height"),
            Self::NoGraphics(why) => write!(f, "no graphics: {why}"),
            Self::Stylesheet(why) => write!(f, "style sheet: {why}"),
            Self::Draw(why) => write!(f, "drawing: {why}"),
            Self::Write(why) => write!(f, "writing the picture: {why}"),
        }
    }
}

impl std::error::Error for RenderError {}
