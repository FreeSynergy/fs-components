//! Keys for the map: the one place that binds a handler to an element that has the keyboard.
//!
//! Pattern: Adapter + Guard — the same rule as `fs_components_widgets::keys` (whose binder is not public): a press
//! reaches the handler only when this element is the one with the keyboard (Blinc may broadcast a press to every
//! handler of the window), and Enter or Space, which the window turns into a click, arrive as [`Key::Activate`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::Div;
use blinc_layout::event_router::keyboard_focused_node;
use fs_components_widgets::{Handled, Key, KeyEvent};

pub(crate) fn bind_keys(div: Div, handler: Arc<dyn Fn(KeyEvent) -> Handled + Send + Sync>) -> Div {
    let focused = Arc::new(AtomicBool::new(false));
    let (on, off) = (Arc::clone(&focused), Arc::clone(&focused));
    let activate = Arc::clone(&handler);
    div.attr("tabindex", "0")
        .on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| off.store(false, Ordering::Relaxed))
        .on_click(move |ctx| {
            if keyboard_focused_node() == Some(ctx.node_id) {
                activate(KeyEvent::new(Key::Activate));
            }
        })
        .on_key_down(move |ctx| {
            let mine = match keyboard_focused_node() {
                Some(node) => node == ctx.node_id,
                None => focused.load(Ordering::Relaxed),
            };
            if mine && !blinc_layout::widgets::text_input::has_focused_text_input() {
                let mut event = KeyEvent::new(Key::from_code(ctx.key_code));
                event.shift = ctx.shift;
                event.ctrl = ctx.ctrl;
                event.alt = ctx.alt;
                handler(event);
            }
        })
}
