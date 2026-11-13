//! What can go wrong in the shell.
//!
//! Pattern: none — a plain error enum.

use std::fmt;

use fs_components_render::RenderError;

/// Why the shell could not be built or shown.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
    /// Two screens have this id.
    DuplicateScreen(String),
    /// Two menu entries have this id.
    DuplicateMenuEntry(String),
    /// `set_frontends` or `set_active_frontend` before `set_standard_menus`.
    NoStandardMenus,
    /// The window or its graphics could not be started.
    Window(String),
    /// The picture could not be drawn.
    Picture(RenderError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateScreen(id) => write!(f, "two screens have the id {id:?}"),
            Self::DuplicateMenuEntry(id) => write!(f, "two menu entries have the id {id:?}"),
            Self::NoStandardMenus => write!(f, "the standard menus are not set"),
            Self::Window(why) => write!(f, "window: {why}"),
            Self::Picture(why) => write!(f, "picture: {why}"),
        }
    }
}

impl std::error::Error for AppError {}
