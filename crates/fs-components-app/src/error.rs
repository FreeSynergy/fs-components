//! What can go wrong in the shell.
//!
//! Pattern: Value object — a plain error enum.

use std::fmt;

use fs_components_render::RenderError;

/// Why the shell could not be built or shown.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
    /// Two screens have this id.
    DuplicateScreen(String),
    /// The window or its graphics could not be started.
    Window(String),
    /// The picture could not be drawn.
    Picture(RenderError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateScreen(id) => write!(f, "two screens have the id {id:?}"),
            Self::Window(why) => write!(f, "window: {why}"),
            Self::Picture(why) => write!(f, "picture: {why}"),
        }
    }
}

impl std::error::Error for AppError {}
