//! The shell, ready to be drawn.
//!
//! Pattern: Facade + Observer — `AppWidget` holds the shared [`Shell`] and builds the element; one
//! counter (signal) per live area tells it to draw again after a change, so a status message does
//! not rebuild the workbench.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::BlincContextState;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use fs_components_render::{Canvas, draw_to_png};

use crate::Element;
use crate::error::AppError;
use crate::shell::{Shell, Status};
use crate::styles::STYLES;
use crate::surface::Screen;
use crate::view::{OnSelect, frame, has_nav, main_area, nav, status_line};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The shell of one program. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct AppWidget {
    id: u64,
    shell: Arc<Mutex<Shell>>,
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
        })
    }

    /// The state, for reading. Change it through [`AppWidget::select`] and
    /// [`AppWidget::set_status`], which also draw again.
    pub fn shell(&self) -> MutexGuard<'_, Shell> {
        locked(&self.shell)
    }

    /// Show the screen with this id. Returns `false` and changes nothing if there is none. Call it
    /// on the interface thread.
    pub fn select(&self, id: &str) -> bool {
        let changed = locked(&self.shell).select(id);
        if changed {
            refresh(self.id, "nav");
            refresh(self.id, "main");
        }
        changed
    }

    /// Replace what the status line says; `None` empties it. Call it on the interface thread.
    pub fn set_status(&self, status: Option<Status>) {
        locked(&self.shell).set_status(status);
        refresh(self.id, "status");
    }

    /// The style sheet of the shell and all its surfaces.
    pub fn styles(&self) -> String {
        [STYLES, &locked(&self.shell).surface_styles()].concat()
    }

    /// The shell filling `width` x `height`, reacting to changes. Call it inside the window's build
    /// function (it keeps its counters in the framework's keyed state).
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let (shell, id) = (&self.shell, self.id);
        // Read first and let go of the lock: `on_state` may run at once and takes the lock itself.
        let (title, with_nav) = {
            let shell = locked(shell);
            (shell.title().to_owned(), has_nav(&shell))
        };
        let side = with_nav.then(|| {
            let (shell, on_select) = (Arc::clone(shell), self.on_select());
            stateful::<NoState>()
                .deps([counter(id, "nav").signal_id()])
                .on_state(move |_| nav(&locked(&shell), &on_select))
        });
        let main = {
            let shell = Arc::clone(shell);
            stateful::<NoState>()
                .deps([counter(id, "main").signal_id()])
                .on_state(move |_| main_area(&locked(&shell), width, height))
        };
        let status = {
            let shell = Arc::clone(shell);
            stateful::<NoState>()
                .deps([counter(id, "status").signal_id()])
                .on_state(move |_| status_line(&locked(&shell), width))
        };
        frame(&title, side, main, status, width, height)
    }

    /// Draw the shell as it is now to a PNG, without a window. The same view functions as in the
    /// window draw the same state; no click is simulated.
    ///
    /// # Errors
    /// [`AppError::Picture`] if the picture cannot be drawn or written.
    pub fn picture(&self, width: u32, height: u32, path: &Path) -> Result<(), AppError> {
        let canvas = Canvas::new(width, height, self.styles());
        let on_select: OnSelect = Arc::new(|_| {});
        let build = |w: f32, h: f32| {
            let shell = locked(&self.shell);
            let side = has_nav(&shell).then(|| nav(&shell, &on_select));
            frame(
                shell.title(),
                side,
                main_area(&shell, w, h),
                status_line(&shell, w),
                w,
                h,
            )
        };
        draw_to_png(&canvas, build, || {}, path).map_err(AppError::Picture)
    }

    fn on_select(&self) -> OnSelect {
        let this = self.clone();
        Arc::new(move |id| {
            this.select(id);
        })
    }
}

fn locked(shell: &Mutex<Shell>) -> MutexGuard<'_, Shell> {
    shell.lock().unwrap_or_else(PoisonError::into_inner)
}

fn counter(id: u64, area: &str) -> blinc_core::State<u64> {
    BlincContextState::get().use_state_keyed(&format!("fs-app-{id}-{area}"), || 0u64)
}

/// Count up the signal of `area`, which makes it draw again. Before the runtime is up there is
/// nothing to draw again.
fn refresh(id: u64, area: &str) {
    if BlincContextState::is_initialized() {
        counter(id, area).update(|v| v + 1);
    }
}
