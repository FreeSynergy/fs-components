//! The state of the shell, testable without a window.
//!
//! Pattern: State — which screen is shown and what the status line says, with its transitions.

use std::sync::Arc;

use crate::error::AppError;
use crate::surface::{Screen, Surface};

/// What the status line says.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Something a person may want to know.
    Info(String),
    /// Something failed; drawn so that it stands out, and it stays until replaced or cleared.
    Problem(String),
}

/// Title, screens, the selected screen and the status line.
pub struct Shell {
    title: String,
    screens: Vec<Screen>,
    current: usize,
    status: Option<Status>,
}

impl Shell {
    /// A shell showing the first screen.
    ///
    /// # Errors
    /// [`AppError::DuplicateScreen`] if two screens have the same id.
    pub fn new(title: impl Into<String>, screens: Vec<Screen>) -> Result<Self, AppError> {
        for (index, screen) in screens.iter().enumerate() {
            if screens[..index].iter().any(|s| s.id == screen.id) {
                return Err(AppError::DuplicateScreen(screen.id.clone()));
            }
        }
        Ok(Self {
            title: title.into(),
            screens,
            current: 0,
            status: None,
        })
    }

    /// The title of the program.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// All screens, in the order of the navigation.
    pub fn screens(&self) -> &[Screen] {
        &self.screens
    }

    /// The screen shown; `None` for a program without screens.
    pub fn current(&self) -> Option<&Screen> {
        self.screens.get(self.current)
    }

    /// Show the screen with this id. Returns `false` and changes nothing if there is none.
    pub fn select(&mut self, id: &str) -> bool {
        match self.screens.iter().position(|s| s.id == id) {
            Some(index) => {
                self.current = index;
                true
            }
            None => false,
        }
    }

    /// What the status line says.
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    /// Replace what the status line says; `None` empties it.
    pub fn set_status(&mut self, status: Option<Status>) {
        self.status = status;
    }

    /// All surfaces, to be used without holding the lock.
    pub(crate) fn surfaces(&self) -> Vec<Arc<dyn Surface>> {
        self.screens
            .iter()
            .map(|s| Arc::clone(&s.surface))
            .collect()
    }
}
