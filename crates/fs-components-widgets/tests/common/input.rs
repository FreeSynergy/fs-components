//! In-process input for tests: the window's own router and hit test, fed by the test.
#![allow(dead_code)]

use blinc_layout::prelude::RenderTree;

/// An in-process mouse and keyboard: the same router and hit test a window uses, fed by the test.
/// No window opens and nothing reaches the real desktop.
pub struct Input {
    router: blinc_layout::event_router::EventRouter,
}

impl Input {
    /// A fresh input state.
    pub fn new() -> Self {
        Self {
            router: blinc_layout::event_router::EventRouter::new(),
        }
    }

    /// The centre of the element with this id.
    pub fn centre(tree: &RenderTree, id: &str) -> (f32, f32) {
        let node = tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        let b = tree.get_absolute_bounds(node).expect("bounds");
        (b.x + b.width / 2.0, b.y + b.height / 2.0)
    }

    /// What a window does: the event carries the pointer, the pointer relative to the element and
    /// the element's rectangle (the pressed element gets the release wherever the pointer is).
    fn send(
        &mut self,
        tree: &mut RenderTree,
        events: Vec<(blinc_layout::tree::LayoutNodeId, u32)>,
        x: f32,
        y: f32,
    ) {
        for (node, event) in events {
            match tree.get_absolute_bounds(node) {
                Some(b) => tree.dispatch_event_with_local(
                    node,
                    event,
                    x,
                    y,
                    x - b.x,
                    y - b.y,
                    b.x,
                    b.y,
                    b.width,
                    b.height,
                ),
                None => tree.dispatch_event(node, event, x, y),
            }
        }
    }

    /// Press and release the left button at a point.
    pub fn click_at(&mut self, tree: &mut RenderTree, x: f32, y: f32) {
        use blinc_layout::event_router::MouseButton::Left;
        let ev = self.router.on_mouse_move(tree, x, y);
        self.send(tree, ev, x, y);
        let ev = self.router.on_mouse_down(tree, x, y, Left);
        self.send(tree, ev, x, y);
        let ev = self.router.on_mouse_up(tree, x, y, Left);
        self.send(tree, ev, x, y);
    }

    /// Move the pointer to a point.
    pub fn move_to(&mut self, tree: &mut RenderTree, x: f32, y: f32) {
        let ev = self.router.on_mouse_move(tree, x, y);
        self.send(tree, ev, x, y);
    }

    /// Press the left button at a point (moving there first).
    pub fn press_at(&mut self, tree: &mut RenderTree, x: f32, y: f32) {
        use blinc_layout::event_router::MouseButton::Left;
        self.move_to(tree, x, y);
        let ev = self.router.on_mouse_down(tree, x, y, Left);
        self.send(tree, ev, x, y);
    }

    /// Release the left button at a point (the pointer moves there first).
    pub fn release_at(&mut self, tree: &mut RenderTree, x: f32, y: f32) {
        use blinc_layout::event_router::MouseButton::Left;
        self.move_to(tree, x, y);
        let ev = self.router.on_mouse_up(tree, x, y, Left);
        self.send(tree, ev, x, y);
    }

    /// Click the centre of the element with this id.
    pub fn click(&mut self, tree: &mut RenderTree, id: &str) {
        let (x, y) = Self::centre(tree, id);
        self.click_at(tree, x, y);
    }

    /// Press a key in the order a Blinc window uses (`blinc_app` `windowed/event_loop/input`):
    /// first the navigation (Tab, and Enter and Space as a click on the focused element); only if
    /// that does not take the key, the router's focused dispatch (without a key code) and then
    /// the broadcast of the key to every key handler.
    pub fn press(&mut self, tree: &mut RenderTree, code: u32, mods: (bool, bool, bool)) {
        use blinc_core::events::event_types::KEY_DOWN;
        let (shift, ctrl, alt) = mods;
        if tree.handle_navigation_key(&mut self.router, code, shift) {
            return;
        }
        if let Some((node, event)) = self.router.on_key_down(code) {
            tree.dispatch_event(node, event, 0.0, 0.0);
        }
        tree.broadcast_key_event(KEY_DOWN, code, shift, ctrl, alt, false);
    }

    /// A key without modifiers, as a window delivers it.
    pub fn key(&mut self, tree: &mut RenderTree, code: u32) {
        self.press(tree, code, (false, false, false));
    }

    /// Same as [`Input::key`]: a window always broadcasts, so there is no separate path.
    pub fn key_broadcast(&mut self, tree: &mut RenderTree, code: u32) {
        self.key(tree, code);
    }

    /// Tab to the next tab stop.
    pub fn tab(&mut self, tree: &mut RenderTree) {
        self.key(tree, 9);
    }

    /// Tab until the element with this id has the keyboard (at most 40 presses); false if it never does.
    pub fn tab_to(&mut self, tree: &mut RenderTree, id: &str) -> bool {
        let want = tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        for _ in 0..40 {
            if self.router.focused() == Some(want) {
                return true;
            }
            self.tab(tree);
        }
        self.router.focused() == Some(want)
    }

    /// Release a key the way a window does: to the focused element.
    pub fn key_release(&mut self, tree: &mut RenderTree, code: u32) {
        if let Some((node, ev)) = self.router.on_key_up(code) {
            tree.dispatch_key_event_bubbling(&[node], ev, code, false, false, false, false);
        }
    }

    /// Release a key to every handler (a window that cannot tell who has focus).
    pub fn key_release_broadcast(&mut self, tree: &mut RenderTree, code: u32) {
        tree.broadcast_key_event(
            blinc_core::events::event_types::KEY_UP,
            code,
            false,
            false,
            false,
            false,
        );
    }

    /// Send one key event of `event_type` (`KEY_DOWN` or `KEY_UP`) to the element with this id, as a
    /// window does to the focused element.
    pub fn key_event_to(&mut self, tree: &mut RenderTree, id: &str, event_type: u32, code: u32) {
        let node = tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        tree.dispatch_key_event_bubbling(&[node], event_type, code, false, false, false, false);
    }

    /// Give the keyboard to the element with this id.
    pub fn focus(&mut self, tree: &RenderTree, id: &str) {
        let node = tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        self.router.focus_via_keyboard(node);
    }
}
