//! What a program puts into the shell.
//!
//! Pattern: Strategy — the shell draws any [`Surface`]; it does not know what a surface shows.

use std::sync::Arc;

use blinc_layout::div::Div;

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

    /// A bar of its own in place of the shell's title bar, so the window has one top line that holds the program's own bar
    /// and its menus. It gets the row of menus the shell would have put into the title bar (`None` when the program has
    /// no menu; place it, Tab and the arrow keys reach it there) and the width of the window, and returns the bar and
    /// its height. Default: `None`, the shell draws its title bar.
    fn bar(&self, _menus: Option<Div>, _width: f32) -> Option<(Div, f32)> {
        None
    }

    /// Whether the shell draws its status line below the surface. A surface that has a footer of its own says `false`
    /// and gets the room. Default: `true`.
    fn status_line(&self) -> bool {
        true
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
