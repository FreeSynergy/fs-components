//! A mouse and a keyboard for the harness, sending what a window sends.
//!
//! Tests: a helper over Blinc's router and keyboard navigation.

use blinc_layout::event_router::MouseButton;
use blinc_layout::tree::LayoutNodeId;

use super::Harness;

/// Key codes as the window passes them.
pub const TAB: u32 = 9;
pub const ENTER: u32 = 13;
pub const ESCAPE: u32 = 27;
pub const LEFT: u32 = 37;
pub const UP: u32 = 38;
pub const RIGHT: u32 = 39;
pub const DOWN: u32 = 40;

impl Harness {
    /// Turn the wheel over a point, as a window does; returns how far it was asked to go.
    pub fn wheel_at(&mut self, (x, y): (f32, f32), down: f32) -> f32 {
        let moved = self.router.on_mouse_move(&self.tree, x, y);
        self.send(moved, x, y);
        let hit = self
            .router
            .hit_test(&self.tree, x, y)
            .expect("something is there");
        self.tree
            .dispatch_scroll_chain(hit.node, &hit.ancestors, x, y, 0.0, -down);
        self.tree.tick_scroll_physics(1_000);
        self.frame();
        down
    }

    /// Press and release the middle of an element, then run a frame.
    pub fn click(&mut self, id: &str) {
        self.press_at(self.centre(id));
        self.frame();
    }

    /// Press and release at a point, with no frame in between.
    pub fn press_at(&mut self, (x, y): (f32, f32)) {
        let moved = self.router.on_mouse_move(&self.tree, x, y);
        self.send(moved, x, y);
        let down = self
            .router
            .on_mouse_down(&self.tree, x, y, MouseButton::Left);
        self.send(down, x, y);
        let up = self.router.on_mouse_up(&self.tree, x, y, MouseButton::Left);
        self.send(up, x, y);
    }

    /// As the window does: each element gets its own bounds, which is where a panel opens.
    fn send(&mut self, events: Vec<(LayoutNodeId, u32)>, x: f32, y: f32) {
        for (node, event) in events {
            let (bx, by, bw, bh) = self.router.get_node_bounds(node).unwrap_or_default();
            self.tree.dispatch_event_full(
                node,
                event,
                x,
                y,
                x - bx,
                y - by,
                bx,
                by,
                bw,
                bh,
                0.0,
                0.0,
                1.0,
                false,
                false,
                false,
                false,
            );
        }
    }

    /// Press a key, then run a frame.
    pub fn key(&mut self, code: u32) {
        self.tree
            .handle_navigation_key(&mut self.router, code, false);
        self.frame();
    }

    /// Does this element have the keyboard?
    pub fn focused(&self, id: &str) -> bool {
        self.router.focused().is_some() && self.router.focused() == self.tree.query_by_id(id)
    }
}
