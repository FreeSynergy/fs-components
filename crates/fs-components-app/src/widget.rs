//! The shell, ready to be drawn.
//!
//! Pattern: Facade + Observer. `AppWidget` holds the shared [`Shell`] and builds the element.
//! Changing the screen asks the window for a full rebuild: a surface built outside a build pass
//! would get new element keys on every change, and its old callbacks would keep running (see the
//! Blinc notes in `DESIGN.md`). The status line is a live area of its own, so a message rebuilds
//! nothing else. No lock is held while a surface or the menu handler runs, so both may use their
//! `AppWidget`.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::{BlincContextState, Signal};
use blinc_layout::div::Div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use fs_components_render::{Canvas, draw_to_png};

use crate::Element;
use crate::error::AppError;
use crate::menu_overlay;
use crate::menu_view::{OnTrigger, menu_bar};
use crate::shell::{Shell, Status};
use crate::status_panel;
use crate::styles::STYLES;
use crate::surface::{Screen, Surface};
use crate::view::{OnSelect, OnStatus, frame, has_nav, main_area, nav, status_line, title_bar};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The shell of one program. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct AppWidget {
    id: u64,
    pub(crate) shell: Arc<Mutex<Shell>>,
    /// Closes the window; set by `launch` (see `quit.rs`).
    pub(crate) quit: crate::quit::QuitSlot,
}

impl AppWidget {
    /// A shell with this title and these screens, showing the first.
    ///
    /// # Errors
    /// [`AppError::DuplicateScreen`] if two screens have the same id.
    pub fn new(title: impl Into<String>, screens: Vec<Screen>) -> Result<Self, AppError> {
        Ok(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            shell: Arc::new(Mutex::new(Shell::new(title, screens)?)),
            quit: crate::quit::QuitSlot::default(),
        })
    }

    /// Read the state. Change it through [`AppWidget::select`] and [`AppWidget::set_status`], which
    /// also draw again. Those two must not be called from inside `read`.
    pub fn with_shell<T>(&self, read: impl FnOnce(&Shell) -> T) -> T {
        read(&locked(&self.shell))
    }

    /// Show the screen with this id. Returns `false` and changes nothing if there is none. Selecting
    /// the screen already shown draws nothing. Call it on the interface thread.
    pub fn select(&self, id: &str) -> bool {
        let (found, changed) = {
            let mut shell = locked(&self.shell);
            let shown = shell.current().is_some_and(|s| s.id == id);
            let found = shell.select(id);
            (found, found && !shown)
        };
        if changed && BlincContextState::is_initialized() {
            BlincContextState::get().request_rebuild();
        }
        found
    }

    /// Replace what the status line says; `None` empties it. Call it on the interface thread: there
    /// is no way yet to post from a worker thread (`DESIGN.md`).
    pub fn set_status(&self, status: Option<Status>) {
        // A panel with the whole text must not go on showing the text that was replaced.
        status_panel::close(&self.shell);
        locked(&self.shell).set_status(status);
        if BlincContextState::is_initialized() {
            status_signal(self.id).update(|v| v + 1);
        }
    }

    /// The style sheet of the shell and all its surfaces; a sheet used by several comes once.
    pub fn styles(&self) -> String {
        let surfaces = self.with_shell(Shell::surfaces);
        let mut sheets: Vec<String> = vec![STYLES.to_owned()];
        for surface in surfaces {
            let sheet = surface.styles();
            if !sheet.is_empty() && !sheets.contains(&sheet) {
                sheets.push(sheet);
            }
        }
        sheets.concat()
    }

    /// The shell filling `width` x `height`, with a live status line. Call it inside the window's
    /// build function.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let (bar, side, surface, with_nav) = self.parts(&self.on_select(), &self.on_trigger());
        if self.with_shell(Shell::open_menu).is_some() {
            menu_overlay::watch_outside(&self.shell);
        }
        if self.with_shell(Shell::status_overlay).is_some() {
            status_panel::watch_outside(&self.shell);
        }
        let status = {
            let shell = Arc::clone(&self.shell);
            let on_status = status_panel::on_status(&self.shell, width);
            stateful::<NoState>()
                .deps([status_signal(self.id).id()])
                .on_state(move |_| {
                    let message = locked(&shell).status().cloned();
                    status_line(message.as_ref(), width, &on_status)
                })
        };
        let main = main_area(surface.as_ref(), with_nav, width, height);
        frame(bar, side, main, status, width, height)
    }

    /// Draw the shell as it is now to a PNG, without a window. The same view functions as in the
    /// window draw the same state, on the light theme, with no menu open; no click is simulated. Not while a window of
    /// this process is open: it would reset that window's theme.
    ///
    /// # Errors
    /// [`AppError::Picture`] if the picture cannot be drawn or written.
    pub fn picture(&self, width: u32, height: u32, path: &Path) -> Result<(), AppError> {
        let canvas = Canvas::new(width, height, self.styles());
        let on_select: OnSelect = Arc::new(|_| {});
        let on_trigger: OnTrigger = Arc::new(|_, _, _| {});
        let on_status: OnStatus = Arc::new(|_, _| {});
        let build = |w: f32, h: f32| {
            let (bar, side, surface, with_nav) = self.parts(&on_select, &on_trigger);
            let main = main_area(surface.as_ref(), with_nav, w, h);
            let message = self.with_shell(|shell| shell.status().cloned());
            let status = status_line(message.as_ref(), w, &on_status);
            frame(bar, side, main, status, w, h)
        };
        draw_to_png(&canvas, build, || {}, path).map_err(AppError::Picture)
    }

    /// What is read under the lock is only copied; nothing built here runs a surface or a closure.
    fn parts(
        &self,
        on_select: &OnSelect,
        on_trigger: &OnTrigger,
    ) -> (Div, Option<Div>, Option<Arc<dyn Surface>>, bool) {
        let (title, menus, side, surface, with_nav) = {
            let shell = locked(&self.shell);
            let with_nav = has_nav(&shell);
            (
                shell.title().to_owned(),
                shell.menus().to_vec(),
                with_nav.then(|| nav(&shell, on_select)),
                shell.current().map(|s| Arc::clone(&s.surface)),
                with_nav,
            )
        };
        let bar = title_bar(&title, menu_bar(&menus, on_trigger));
        (bar, side, surface, with_nav)
    }

    fn on_select(&self) -> OnSelect {
        let this = self.clone();
        Arc::new(move |id| {
            this.select(id);
        })
    }
}

pub(crate) fn locked(shell: &Mutex<Shell>) -> MutexGuard<'_, Shell> {
    shell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Counts up when the status line changed. A signal, not a `State`: it also reaches the window when
/// a picture set up the runtime first.
fn status_signal(id: u64) -> Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-app-{id}-status"), || 0u64)
}
