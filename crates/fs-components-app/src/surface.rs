//! What a program puts into the shell.
//!
//! Pattern: Strategy — the shell draws any [`Surface`]; it does not know what a surface shows.

use std::sync::Arc;

use crate::Element;

/// A work area: usually a workbench of `fs-components`.
pub trait Surface: Send + Sync {
    /// The element of this surface, filling `width` x `height`. Called again whenever the screen is
    /// shown or the window changes size, so a surface keeps its state outside the element. `width` and
    /// `height` may be 0, never negative. No lock of the shell is held, so a surface may use its
    /// `AppWidget`.
    fn build(&self, width: f32, height: f32) -> Box<dyn Element>;

    /// The style sheet this surface needs (theme variables only). Default: none.
    fn styles(&self) -> String {
        String::new()
    }
}

/// One entry of the navigation and the surface behind it.
#[non_exhaustive]
#[derive(Clone)]
pub struct Screen {
    /// Stable name of the screen, unique within the program.
    pub id: String,
    /// What the navigation shows; comes from the program.
    pub label: String,
    /// What is drawn when this screen is selected.
    pub surface: Arc<dyn Surface>,
}

impl Screen {
    /// A screen with this id, label and surface.
    pub fn new(id: impl Into<String>, label: impl Into<String>, surface: Arc<dyn Surface>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            surface,
        }
    }
}
