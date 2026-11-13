//! An open menu in Blinc's overlay stack, kept in step with [`Shell`].
//!
//! Pattern: Facade over the overlay stack. Which menu is open is the shell's; the stack only draws it.
//! The lock of the shell is never held while the stack is touched, because the stack runs closures
//! that take that lock. Blinc does not close a dropdown on a press outside, so the shell asks for it.

use std::sync::{Arc, Mutex};

use blinc_layout::click_outside;
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};

use crate::menu_view::{BAR_ID, Callbacks, PANEL_ID, SUB_PANEL_ID, panel, panel_size};
use crate::shell::Shell;
use crate::widget::locked;

/// The key of the one registration for a press outside.
const OUTSIDE_KEY: &str = "fs-app-menu-outside";

/// Close the open menu, if any.
pub(crate) fn close(shell: &Mutex<Shell>) {
    let overlay = {
        let mut shell = locked(shell);
        shell.close_menu();
        shell.take_overlay()
    };
    click_outside::unregister_click_outside(OUTSIDE_KEY);
    if let Some(raw) = overlay {
        OverlayHandle::from_raw(raw).close();
    }
}

/// Open this menu; if it is the open one, close it. Another open menu closes first.
pub(crate) fn toggle(
    shell: &Arc<Mutex<Shell>>,
    index: usize,
    at: (f32, f32),
    calls: &Callbacks,
) {
    let was_open = locked(shell).open_menu() == Some(index);
    close(shell);
    if was_open {
        return;
    }
    let menu = {
        let mut shell = locked(shell);
        if !shell.open(index) {
            return;
        }
        shell.set_open_at(at);
        shell.menus()[index].clone()
    };
    let raw = show(shell, index, menu, (at, None), calls);
    locked(shell).set_overlay(raw);
    watch_outside(shell);
}

/// The next level of a line of the open menu opens, or closes if it was open: the panel is drawn again. Does nothing if
/// no menu is open or the line has no children.
pub(crate) fn toggle_sub(shell: &Arc<Mutex<Shell>>, position: usize, calls: &Callbacks) {
    let (index, menu, sub, at, old) = {
        let mut shell = locked(shell);
        if !shell.toggle_sub(position) {
            return;
        }
        let Some(index) = shell.open_menu() else { return };
        (index, shell.menus()[index].clone(), shell.open_sub(), shell.open_at(), shell.take_overlay())
    };
    // The old overlay goes first; its close callback sees that it is no longer the shell's and does nothing.
    if let Some(raw) = old {
        OverlayHandle::from_raw(raw).close();
    }
    let raw = show(shell, index, menu, (at, sub), calls);
    locked(shell).set_overlay(raw);
    watch_outside(shell);
}

/// Ask for a press outside the bar and the panel. Blinc forgets this at every full rebuild, so the
/// build asks again while a menu is open.
pub(crate) fn watch_outside(shell: &Arc<Mutex<Shell>>) {
    let shell = Arc::clone(shell);
    let inside = [BAR_ID.to_owned(), PANEL_ID.to_owned(), SUB_PANEL_ID.to_owned()];
    click_outside::register_click_outside_multi(OUTSIDE_KEY, &inside, move || close(&shell));
}

fn show(
    shell: &Arc<Mutex<Shell>>,
    index: usize,
    menu: crate::menu::Menu,
    ((x, y), sub): ((f32, f32), Option<usize>),
    calls: &Callbacks,
) -> u64 {
    let calls = calls.clone();
    let (width, height) = panel_size(&menu, sub);
    let raw = overlay_stack()
        .lock()
        .map_or(0, |stack| stack.peek_next_handle_id());
    let shell = Arc::clone(shell);
    OverlayBuilder::dropdown()
        .at(x, y)
        .size(width, height)
        .dismissable_by_escape(true)
        .on_close(move |_reason| {
            // Escape comes this way. A menu opened since then is not this one.
            let mut shell = locked(&shell);
            if shell.overlay() == Some(raw) {
                shell.close_menu();
                shell.take_overlay();
                click_outside::unregister_click_outside(OUTSIDE_KEY);
            }
        })
        .content(move || panel(index, &menu, sub, &calls))
        .show()
        .raw()
}
