//! The state of the shell, testable without a window.
//!
//! Pattern: State — which screen is shown, what the status line says, the menus and which of them is
//! open, with their transitions.

use std::collections::HashSet;
use std::sync::Arc;

use crate::error::AppError;
use crate::menu::{Menu, MenuHandler};
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

/// Title, screens, the selected screen, the status line and the menus.
pub struct Shell {
    title: String,
    screens: Vec<Screen>,
    current: usize,
    status: Option<Status>,
    menus: Vec<Menu>,
    handler: Option<MenuHandler>,
    open_menu: Option<usize>,
    overlay: Option<u64>,
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
            menus: Vec::new(),
            handler: None,
            open_menu: None,
            overlay: None,
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

    /// Replace the menus and the handler that receives the id of a chosen entry. An open menu closes.
    ///
    /// # Errors
    /// [`AppError::DuplicateMenuEntry`] if two entries have the same id; then nothing changes.
    pub fn set_menus(
        &mut self,
        menus: Vec<Menu>,
        handler: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<(), AppError> {
        let mut seen = HashSet::new();
        for entry in menus.iter().flat_map(|menu| &menu.entries) {
            if !seen.insert(entry.id.as_str()) {
                return Err(AppError::DuplicateMenuEntry(entry.id.clone()));
            }
        }
        self.menus = menus;
        self.handler = Some(Arc::new(handler));
        self.open_menu = None;
        Ok(())
    }

    /// The menus, in the order of the title bar.
    pub fn menus(&self) -> &[Menu] {
        &self.menus
    }

    /// The index of the open menu.
    pub fn open_menu(&self) -> Option<usize> {
        self.open_menu
    }

    /// Mark this menu as open. Returns `false` and changes nothing if there is none at this index or
    /// it has no entries.
    pub fn open(&mut self, index: usize) -> bool {
        let can = self.menus.get(index).is_some_and(|m| !m.entries.is_empty());
        if can {
            self.open_menu = Some(index);
        }
        can
    }

    /// Mark no menu as open.
    pub fn close_menu(&mut self) {
        self.open_menu = None;
    }

    /// The handler to run for this entry; `None` if the entry is unknown or disabled. Run it after
    /// letting go of the shell, so it may change the shell.
    pub fn handler_for(&self, id: &str) -> Option<MenuHandler> {
        self.menus
            .iter()
            .flat_map(|menu| &menu.entries)
            .any(|entry| entry.id == id && !entry.disabled)
            .then(|| self.handler.clone())
            .flatten()
    }

    /// The overlay that draws the open menu, as Blinc's raw handle.
    pub(crate) fn overlay(&self) -> Option<u64> {
        self.overlay
    }

    pub(crate) fn set_overlay(&mut self, raw: u64) {
        self.overlay = Some(raw);
    }

    pub(crate) fn take_overlay(&mut self) -> Option<u64> {
        self.overlay.take()
    }

    /// All surfaces, to be used without holding the lock.
    pub(crate) fn surfaces(&self) -> Vec<Arc<dyn Surface>> {
        self.screens
            .iter()
            .map(|s| Arc::clone(&s.surface))
            .collect()
    }
}
