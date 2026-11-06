//! The real element, laid out and driven like a window drives it, without opening one.
//!
//! Pattern: Facade over Blinc's tree and router, with a full rebuild as the window's event loop does
//! it. Hold no other picture in the same binary without [`serial`]: Blinc's first set-up panics when two
//! threads race, and `draw_to_png` holds the process-wide picture lock.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use blinc_layout::tree::LayoutNodeId;
use fs_components_workbenches::calendar::CalendarWidget;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Hold this while a test touches Blinc's singletons.
pub fn serial() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A widget in a tree of its own, and a mouse.
pub struct Harness {
    pub widget: CalendarWidget,
    pub tree: RenderTree,
    scope: BuildScope,
    router: EventRouter,
    size: (f32, f32),
    _serial: MutexGuard<'static, ()>,
}

fn tree_of(widget: &CalendarWidget, scope: &mut BuildScope, (w, h): (f32, f32)) -> RenderTree {
    let bundle = fs_components_render::bundle(&widget.styles());
    blinc_theme::ThemeState::init(bundle.clone(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    blinc_layout::init_text_measurer();
    // What a window does around a rebuild: fresh keys, and a scope that owns what the build makes.
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
    pub fn new(widget: CalendarWidget, w: f32, h: f32) -> Self {
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

    /// Is there an element with this id?
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
        let id = self.id(what);
        let node = self
            .tree
            .query_by_id(&id)
            .unwrap_or_else(|| panic!("no element {id}"));
        let bounds = self.tree.get_absolute_bounds(node).expect("bounds");
        (
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        )
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

    /// Turn the wheel over a point, as a window does; returns where the element under it now is.
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
        let updates = self.tree.take_pending_property_updates();
        self.tree.apply_partial_property_updates(updates);
        self.tree.compute_layout(self.size.0, self.size.1);
        self.centre("page").1
    }
}
