//! An open menu in Blinc's overlay stack, kept in step with [`Shell`].
//!
//! Pattern: Facade over the overlay stack. Which menu is open is the shell's; the stack only draws it.
//! The lock of the shell is never held while the stack is touched, because the stack runs closures
//! that take that lock. Blinc does not close a dropdown on a press outside, so the shell asks for it.

use std::sync::{Arc, Mutex};

use blinc_layout::click_outside;
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};

use crate::menu_view::{BAR_ID, OnChoose, PANEL_ID, panel, panel_size};
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
    on_choose: &OnChoose,
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
        shell.menus()[index].clone()
    };
    let raw = show(shell, index, menu, at, on_choose);
    locked(shell).set_overlay(raw);
    watch_outside(shell);
}

/// Ask for a press outside the bar and the panel. Blinc forgets this at every full rebuild, so the
/// build asks again while a menu is open.
pub(crate) fn watch_outside(shell: &Arc<Mutex<Shell>>) {
    let shell = Arc::clone(shell);
    let inside = [BAR_ID.to_owned(), PANEL_ID.to_owned()];
    click_outside::register_click_outside_multi(OUTSIDE_KEY, &inside, move || close(&shell));
}

fn show(
    shell: &Arc<Mutex<Shell>>,
    index: usize,
    menu: crate::menu::Menu,
    (x, y): (f32, f32),
    on_choose: &OnChoose,
) -> u64 {
    let on_choose = Arc::clone(on_choose);
    let (width, height) = panel_size(&menu);
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
        .content(move || panel(index, &menu, &on_choose))
        .show()
        .raw()
}
