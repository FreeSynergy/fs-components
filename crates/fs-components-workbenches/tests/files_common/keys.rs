//! The keys of the harness: the window's order for a key, and Tab to an element.
//!
//! Pattern: Facade over Blinc's tree and router, as `harness` is.

use blinc_core::events::event_types::KEY_DOWN;

use super::harness::Harness;

impl Harness {
    /// Press a key in the order a window uses.
    pub fn key(&mut self, code: u32) {
        if self
            .tree
            .handle_navigation_key(&mut self.router, code, false)
        {
            return;
        }
        if let Some((node, event)) = self.router.on_key_down(code) {
            self.tree.dispatch_event(node, event, 0.0, 0.0);
        }
        self.tree
            .broadcast_key_event(KEY_DOWN, code, false, false, false, false);
    }

    /// Press Tab until the element `<name>-<what>` has the keyboard; `false` if it never does.
    pub fn tab_to(&mut self, what: &str) -> bool {
        let (left, top, right, bottom) = self.edges(what);
        for _ in 0..60 {
            self.key(9);
            // The element itself, or the button it holds.
            let inside = self.router.focused().is_some_and(|node| {
                self.tree.get_absolute_bounds(node).is_some_and(|b| {
                    b.x >= left - 0.5
                        && b.y >= top - 0.5
                        && b.x + b.width <= right + 0.5
                        && b.y + b.height <= bottom + 0.5
                })
            });
            if inside {
                return true;
            }
        }
        false
    }

    /// Press Tab until the list itself has the keyboard; `false` if it never does.
    pub fn tab_to_list(&mut self) -> bool {
        let name = self.widget.element_name().to_string();
        let Some(list) = self.tree.query_by_id(&name) else {
            return false;
        };
        for _ in 0..60 {
            self.key(9);
            if self.router.focused() == Some(list) {
                return true;
            }
        }
        false
    }
}
