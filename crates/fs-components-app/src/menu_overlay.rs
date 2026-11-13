//! An open menu in Blinc's overlay stack, kept in step with [`Shell`].
//!
//! Pattern: Facade over the overlay stack. Which menu is open is the shell's; the stack only draws it. The panel and the panel of
//! its next level are two overlays: Blinc moves an overlay that would stand over the right edge of the window to the left, and
//! that must move only the next level (which folds to the left of the panel instead), never the panel under its menu.
//! The lock of the shell is never held while the stack is touched, because the stack runs closures
//! that take that lock. Blinc does not close a dropdown on a press outside, so the shell asks for it.

use std::sync::{Arc, Mutex};

use blinc_layout::click_outside;
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};

use crate::menu::MenuEntry;
use crate::menu_view::{
    Anchor, BAR_ID, Callbacks, Look, PANEL_ID, SUB_PANEL_ID, content_width, folds_left, panel,
    panel_size, sub_panel,
};
use crate::shell::Shell;
use crate::styles::{MENU_WIDTH, SIDE_MARGIN};
use crate::widget::locked;

/// The key of the one registration for a press outside.
const OUTSIDE_KEY: &str = "fs-app-menu-outside";

/// Close the open menu, if any.
pub(crate) fn close(shell: &Mutex<Shell>) {
    let (overlay, sub) = {
        let mut shell = locked(shell);
        shell.close_menu();
        (shell.take_overlay(), shell.take_sub_overlay())
    };
    click_outside::unregister_click_outside(OUTSIDE_KEY);
    for raw in [sub, overlay].into_iter().flatten() {
        OverlayHandle::from_raw(raw).close();
    }
}

/// Open this menu; if it is the open one, close it. Another open menu closes first.
pub(crate) fn toggle(shell: &Arc<Mutex<Shell>>, index: usize, at: (f32, f32), calls: &Callbacks) {
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
    let raw = show(shell, index, menu, at, calls);
    locked(shell).set_overlay(raw);
    watch_outside(shell);
}

/// The next level of a line of the open menu opens beside that line (`anchor`: its left, top and width), or closes if it was
/// open. It stands to the right of the panel, and folds to the left of it where the window has no room. Does nothing if no menu
/// is open or the line has no children.
pub(crate) fn toggle_sub(
    shell: &Arc<Mutex<Shell>>,
    position: usize,
    anchor: Anchor,
    calls: &Callbacks,
) {
    let (children, old) = {
        let mut shell = locked(shell);
        if !shell.toggle_sub(position) {
            return;
        }
        let children = shell
            .open_menu()
            .and_then(|i| shell.menus().get(i))
            .zip(shell.open_sub())
            .and_then(|(menu, p)| menu.entries.get(p))
            .map(|e| e.children.clone());
        (children, shell.take_sub_overlay())
    };
    // The old one goes first; its close callback sees that it is no longer the shell's and does nothing.
    if let Some(raw) = old {
        OverlayHandle::from_raw(raw).close();
    }
    let Some(children) = children else { return };
    let index = locked(shell).open_menu().unwrap_or(0);
    let window = window_width();
    let width = content_width(&children, window);
    let at = sub_position(anchor, width, window);
    let raw = show_sub(shell, index, children, (at, width), calls);
    locked(shell).set_sub_overlay(raw);
    watch_outside(shell);
}

fn window_width() -> f32 {
    overlay_stack().lock().map_or(0.0, |s| s.viewport().0)
}

/// Where the next level starts: at the right edge of the line, level with it; at the left edge of the line minus its own width
/// if the window ends before the right-hand place does (`folds_left`), but never off the window.
fn sub_position(anchor: Anchor, sub: f32, window: f32) -> (f32, f32) {
    let (left, top, line) = anchor;
    let x = if folds_left(left, line, sub, window) {
        (left - sub).max(SIDE_MARGIN)
    } else {
        left + line
    };
    (x, top)
}

/// Ask for a press outside the bar and the panel. Blinc forgets this at every full rebuild, so the
/// build asks again while a menu is open.
pub(crate) fn watch_outside(shell: &Arc<Mutex<Shell>>) {
    let shell = Arc::clone(shell);
    let inside = [
        BAR_ID.to_owned(),
        PANEL_ID.to_owned(),
        SUB_PANEL_ID.to_owned(),
    ];
    click_outside::register_click_outside_multi(OUTSIDE_KEY, &inside, move || close(&shell));
}

fn show(
    shell: &Arc<Mutex<Shell>>,
    index: usize,
    menu: crate::menu::Menu,
    (x, y): (f32, f32),
    calls: &Callbacks,
) -> u64 {
    let calls = calls.clone();
    let window = window_width();
    // Where Blinc will put the panel: it moves one that would cross the right edge.
    let left = if window > 0.0 { x.min(window - MENU_WIDTH - SIDE_MARGIN).max(SIDE_MARGIN) } else { x };
    let widest = menu
        .entries
        .iter()
        .filter(|e| !e.children.is_empty())
        .map(|e| content_width(&e.children, window))
        .fold(0.0, f32::max);
    let arrow_left = widest > 0.0 && folds_left(left, MENU_WIDTH, widest, window);
    let (width, height) = panel_size(&menu.entries, MENU_WIDTH);
    let raw = overlay_stack()
        .lock()
        .map_or(0, |stack| stack.peek_next_handle_id());
    let shell = Arc::clone(shell);
    let reader = Arc::clone(&shell);
    OverlayBuilder::dropdown()
        .at(x, y)
        .size(width, height)
        .dismissable_by_escape(true)
        .on_close(move |_reason| {
            // Escape comes this way. A menu opened since then is not this one.
            let sub = {
                let mut shell = locked(&shell);
                if shell.overlay() != Some(raw) {
                    return;
                }
                shell.close_menu();
                shell.take_overlay();
                shell.take_sub_overlay()
            };
            click_outside::unregister_click_outside(OUTSIDE_KEY);
            if let Some(sub) = sub {
                OverlayHandle::from_raw(sub).close();
            }
        })
        .content(move || {
            // Drawn again when the next level opens or closes (the stack rebuilds all its entries), so the line that opens it
            // says `aria-expanded` as long as it is open. The shell's lock is taken inside the stack's, as `on_close` does.
            let open_sub = locked(&reader).open_sub();
            let look = Look { open_sub, arrow_left, width: MENU_WIDTH };
            panel(index, &menu, look, &calls)
        })
        .show()
        .raw()
}

fn show_sub(
    shell: &Arc<Mutex<Shell>>,
    index: usize,
    entries: Vec<MenuEntry>,
    ((x, y), width): ((f32, f32), f32),
    calls: &Callbacks,
) -> u64 {
    let calls = calls.clone();
    let (width, height) = panel_size(&entries, width);
    let raw = overlay_stack()
        .lock()
        .map_or(0, |stack| stack.peek_next_handle_id());
    let shell = Arc::clone(shell);
    OverlayBuilder::dropdown()
        .at(x, y)
        .size(width, height)
        .dismissable_by_escape(true)
        .on_close(move |_reason| {
            // Escape closes this level only; the menu stays.
            let mut shell = locked(&shell);
            if shell.sub_overlay() == Some(raw) {
                shell.take_sub_overlay();
                shell.clear_open_sub();
            }
        })
        .content(move || sub_panel(index, &entries, width, &calls))
        .show()
        .raw()
}
