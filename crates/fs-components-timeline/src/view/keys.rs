//! Keyboard for the grid root.
//!
//! Pattern: Adapter — Blinc key events in, [`Key`] out. Enter and Space reach a Blinc widget as a
//! click on the focused node (the window's navigation takes them first), so the click handler
//! reports them as `Key::Activate`, only when the keyboard, not the mouse, caused it. A copy of
//! the private helper of `fs-components-widgets` (see DESIGN.md: to move into a shared crate).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::Div;
use blinc_layout::event_router::keyboard_focused_node;
use fs_components_widgets::Key;

/// Make `div` a tab stop whose handler runs while `div` itself has the keyboard.
pub fn bind_keys(div: Div, handler: Arc<dyn Fn(Key) -> bool + Send + Sync>) -> Div {
    let focused = Arc::new(AtomicBool::new(false));
    let (on, off) = (Arc::clone(&focused), Arc::clone(&focused));
    let activate = Arc::clone(&handler);
    div.attr("tabindex", "0")
        .on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| off.store(false, Ordering::Relaxed))
        .on_click(move |ctx| {
            if keyboard_focused_node() == Some(ctx.node_id) {
                activate(Key::Activate);
            }
        })
        .on_key_down(move |ctx| {
            let mine = match keyboard_focused_node() {
                Some(node) => node == ctx.node_id,
                None => focused.load(Ordering::Relaxed),
            };
            if mine && !blinc_layout::widgets::text_input::has_focused_text_input() {
                handler(Key::from_code(ctx.key_code));
            }
        })
}
