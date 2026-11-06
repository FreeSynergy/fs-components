//! The menus of an [`AppWidget`]: what a program calls, and what the menu elements call back.
//!
//! Pattern: Facade over [`Shell`] and the overlay of the open menu. The handler runs with no lock
//! held, so it may use its `AppWidget`.

use std::sync::Arc;

use blinc_core::BlincContextState;

use crate::error::AppError;
use crate::menu::Menu;
use crate::menu_overlay;
use crate::menu_view::{OnChoose, OnTrigger};
use crate::shell::Shell;
use crate::widget::{AppWidget, locked};

impl AppWidget {
    /// Put these menus into the title bar; the one handler receives the id of a chosen entry, with no
    /// lock held, so it may use this `AppWidget`. Call again to change them, for example to disable an
    /// entry; an open menu closes. A handler holding a clone of this `AppWidget` keeps the shell alive
    /// as long as the shell holds the handler, which is what a program wants. Call it on the interface
    /// thread.
    ///
    /// # Errors
    /// [`AppError::DuplicateMenuEntry`] if two entries have the same id; then nothing changes.
    pub fn set_menus(
        &self,
        menus: Vec<Menu>,
        handler: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<(), AppError> {
        locked(&self.shell).set_menus(menus, handler)?;
        menu_overlay::close(&self.shell);
        if BlincContextState::is_initialized() {
            BlincContextState::get().request_rebuild();
        }
        Ok(())
    }

    /// Choose this menu entry, as a press on its line does: an open menu closes and the handler runs,
    /// with no lock held. Returns `false` and does nothing if the entry is unknown or disabled.
    pub fn choose(&self, id: &str) -> bool {
        let Some(handler) = locked(&self.shell).handler_for(id) else {
            return false;
        };
        // First close: what the handler opens must not go down with the menu.
        menu_overlay::close(&self.shell);
        handler(id);
        true
    }

    /// Reads the menu at the press, so a handler kept by an old element still opens what is current.
    pub(crate) fn on_trigger(&self) -> OnTrigger {
        let this = self.clone();
        Arc::new(move |index, x, y| {
            menu_overlay::toggle(&this.shell, index, (x, y), &this.on_choose());
        })
    }

    /// A line counts only while its menu is open: a closing panel is still drawn for a moment, and a
    /// second press of a double-click lands on it.
    fn on_choose(&self) -> OnChoose {
        let this = self.clone();
        Arc::new(move |index, id| {
            if this.with_shell(Shell::open_menu) == Some(index) {
                this.choose(id);
            }
        })
    }
}
