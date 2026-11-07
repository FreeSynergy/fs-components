//! The real element, laid out and driven like a window drives it, without opening one.
//!
//! Tests: a helper over Blinc's tree and router, with a full rebuild (`Harness::rebuild`) and a frame
//! for live areas (`Harness::frame`) as the window's event loop does them. Keys take the window's
//! order: navigation first, then the router's focused dispatch, then the broadcast.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use blinc_layout::tree::LayoutNodeId;
use blinc_layout::widgets::text_input::{focused_text_area_node_id, focused_text_input_node_id};
use fs_components_workbenches::tasks::TasksWidget;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Hold this while a test touches Blinc's singletons: their first set-up panics when two threads race.
pub fn serial() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A widget in a tree of its own, a mouse and a keyboard.
pub struct Harness {
    pub widget: TasksWidget,
    pub tree: RenderTree,
    scope: BuildScope,
    router: EventRouter,
    size: (f32, f32),
    _serial: MutexGuard<'static, ()>,
}

fn tree_of(widget: &TasksWidget, scope: &mut BuildScope, (w, h): (f32, f32)) -> RenderTree {
    let bundle = fs_components_render::bundle(&widget.styles());
    blinc_theme::ThemeState::init(bundle.clone(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        // As a window does: a `State` reaches its live areas through this callback.
        BlincContextState::init_with_callback(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            Arc::new(|ids: &[blinc_core::reactive::SignalId]| {
                blinc_layout::check_stateful_deps(ids);
            }),
        );
    }
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    blinc_layout::init_text_measurer();
    let pass = scope.begin();
    blinc_layout::key::reset_call_counters();
    let mut tree = RenderTree::from_element(&widget.build(w, h));
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let css = bundle.css_sources.join("\n");
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).expect("stylesheet"));
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    scope.finish(pass);
    tree
}

impl Harness {
    /// `widget` laid out at `w` x `h`.
    pub fn new(widget: TasksWidget, w: f32, h: f32) -> Self {
        let serial = serial();
        let mut scope = BuildScope::new();
        let tree = tree_of(&widget, &mut scope, (w, h));
        Self {
            widget,
            tree,
            scope,
            router: EventRouter::new(),
            size: (w, h),
            _serial: serial,
        }
    }

    /// The window builds everything again, as after `request_rebuild`.
    pub fn rebuild(&mut self) {
        self.tree = tree_of(&self.widget, &mut self.scope, self.size);
    }

    /// The window runs a frame: live areas that changed draw again.
    pub fn frame(&mut self) {
        let updates = self.tree.take_pending_property_updates();
        self.tree.apply_partial_property_updates(updates);
        self.tree.process_pending_subtree_rebuilds_routed(None);
        self.tree.compute_layout(self.size.0, self.size.1);
    }

    /// Is there an element `<name>-<what>`?
    pub fn has(&self, what: &str) -> bool {
        self.tree.query_by_id(&self.id(what)).is_some()
    }

    fn id(&self, what: &str) -> String {
        format!("{}-{what}", self.widget.element_name())
    }

    /// Click the element `<name>-<what>`, with the press and release a mouse sends, then rebuild.
    pub fn click(&mut self, what: &str) {
        self.press(what);
        self.rebuild();
    }

    /// Click without the rebuild.
    pub fn press(&mut self, what: &str) {
        self.press_at(self.centre(what));
    }

    /// The middle of the element `<name>-<what>`.
    pub fn centre(&self, what: &str) -> (f32, f32) {
        let (left, top, right, bottom) = self.edges(what);
        (f32::midpoint(left, right), f32::midpoint(top, bottom))
    }

    /// The bounds of the element `<name>-<what>`: left, top, right, bottom.
    pub fn edges(&self, what: &str) -> (f32, f32, f32, f32) {
        let id = self.id(what);
        let node = self
            .tree
            .query_by_id(&id)
            .unwrap_or_else(|| panic!("no element {id}"));
        let b = self.tree.get_absolute_bounds(node).expect("bounds");
        (b.x, b.y, b.x + b.width, b.y + b.height)
    }

    /// Press and release at a point, with nothing in between.
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

    fn send(&mut self, events: Vec<(LayoutNodeId, u32)>, x: f32, y: f32) {
        for (node, event) in events {
            self.tree.dispatch_event(node, event, x, y);
        }
    }

    /// Type into the text field that has the focus. Returns `false` if none has.
    pub fn type_text(&mut self, text: &str) -> bool {
        let Some(node) = focused_text_input_node_id().or_else(focused_text_area_node_id) else {
            return false;
        };
        for c in text.chars() {
            self.tree
                .dispatch_text_input_event(node, c, false, false, false, false);
        }
        true
    }

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

    /// Turn the wheel over a point, as a window does.
    pub fn wheel_at(&mut self, (x, y): (f32, f32), down: f32) {
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
    }

    /// Press Tab until the list has the keyboard; `false` if it never does.
    pub fn tab_to_list(&mut self) -> bool {
        let wanted = self.tree.query_by_id(&self.widget.element_name());
        for _ in 0..60 {
            self.key(9);
            if wanted.is_some() && self.router.focused() == wanted {
                return true;
            }
        }
        false
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
}
