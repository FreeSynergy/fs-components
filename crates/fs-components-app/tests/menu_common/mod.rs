//! The real shell, laid out and driven like a window drives it, without opening one.
//!
//! Pattern: Facade over Blinc's tree, router and overlay stack, in the order of its event loop
//! (`blinc_app/src/windowed/event_loop`): the overlay layer beside the program's element, a rebuild
//! through the incremental update, dismissals drained each frame.

#![allow(dead_code)]

pub mod fixture;
mod input;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::div::{Div, div};
use blinc_layout::event_router::EventRouter;
use blinc_layout::overlay_state::{overlay_stack, rebuild_overlay_subtree_if_dirty};
use blinc_layout::prelude::RenderTree;
use blinc_layout::widgets::overlay_stack::{OVERLAY_STACK_LAYER_ID, drain_close_callbacks};
use blinc_layout::{UpdateResult, click_outside};
use fs_components_app::AppWidget;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

#[allow(unused_imports)]
pub use input::{DOWN, ENTER, ESCAPE, LEFT, RIGHT, TAB, UP};

/// Hold this while a test touches Blinc's singletons.
pub fn serial() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the window composes: the program's element and the overlay stack above it.
pub fn composed(app: &AppWidget, w: f32, h: f32) -> Div {
    let layer = {
        let stack = overlay_stack();
        let mut stack = stack.lock().unwrap();
        stack.set_viewport_with_scale(w, h, 1.0);
        stack.build_overlay_layer()
    };
    div()
        .w(w)
        .h(h)
        .relative()
        .child(app.build(w, h))
        .child(layer)
}

/// An app in a tree of its own, a mouse and a keyboard.
pub struct Harness {
    pub app: AppWidget,
    pub tree: RenderTree,
    pub router: EventRouter,
    scope: BuildScope,
    size: (f32, f32),
    clock: u64,
    _serial: MutexGuard<'static, ()>,
}

impl Harness {
    /// `app` laid out at `w` x `h`, with no overlay left from an earlier test.
    pub fn new(app: AppWidget, w: f32, h: f32) -> Self {
        let serial = serial();
        let bundle = fs_components_render::bundle(&app.styles());
        blinc_theme::ThemeState::init(bundle.clone(), blinc_theme::ColorScheme::Light);
        if !BlincContextState::is_initialized() {
            BlincContextState::init(
                global_graph(),
                Arc::new(Mutex::new(HookState::new())),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            );
        }
        blinc_core::reactive::set_stateful_deps_notifier(|ids| {
            blinc_layout::check_stateful_deps(ids);
        });
        blinc_layout::init_text_measurer();
        click_outside::clear_click_outside_handlers();
        overlay_stack().lock().unwrap().close_all();
        drain_close_callbacks();
        overlay_stack().lock().unwrap().update(u64::MAX / 2);
        let mut scope = BuildScope::new();
        let pass = scope.begin();
        blinc_layout::key::reset_call_counters();
        let mut tree = RenderTree::from_element(&composed(&app, w, h));
        let vars = blinc_theme::ThemeState::get().to_css_variable_map();
        let css = bundle.css_sources.join("\n");
        tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).expect("stylesheet"));
        tree.apply_stylesheet_layout_overrides();
        tree.apply_stylesheet_base_styles();
        tree.compute_layout(w, h);
        scope.finish(pass);
        let mut harness = Self {
            app,
            tree,
            router: EventRouter::new(),
            scope,
            size: (w, h),
            clock: 1,
            _serial: serial,
        };
        harness.frame();
        harness
    }

    /// The window builds everything again, as after `request_rebuild` or a resize of a live area.
    pub fn rebuild(&mut self) {
        let (w, h) = self.size;
        let pass = self.scope.begin();
        blinc_layout::key::reset_call_counters();
        click_outside::clear_click_outside_handlers();
        let ui = composed(&self.app, w, h);
        match self.tree.incremental_update(&ui) {
            UpdateResult::NoChanges | UpdateResult::VisualOnly => {}
            UpdateResult::LayoutChanged => {
                self.tree.apply_stylesheet_layout_overrides();
                self.tree.compute_layout(w, h);
            }
            UpdateResult::ChildrenChanged => {
                self.tree.apply_stylesheet_base_styles();
                self.tree.apply_stylesheet_layout_overrides();
                self.tree.compute_layout(w, h);
            }
        }
        self.scope.keep(pass);
        self.frame();
    }

    /// The window runs a frame: dismissals report, the overlay layer and live areas draw again.
    pub fn frame(&mut self) {
        self.clock += 16;
        drain_close_callbacks();
        overlay_stack().lock().unwrap().update(self.clock);
        let registry = Arc::clone(self.tree.element_registry());
        let dirty = overlay_stack().lock().unwrap().take_dirty();
        rebuild_overlay_subtree_if_dirty(&registry, OVERLAY_STACK_LAYER_ID, dirty, || {
            overlay_stack().lock().unwrap().build_overlay_layer()
        });
        let updates = self.tree.take_pending_property_updates();
        self.tree.apply_partial_property_updates(updates);
        self.tree
            .process_pending_subtree_rebuilds_routed(Some(&self.router));
        self.tree.apply_stylesheet_base_styles();
        self.tree.apply_stylesheet_layout_overrides();
        self.tree.compute_layout(self.size.0, self.size.1);
        self.tree.sync_keyboard_focus(&mut self.router);
    }

    /// Frames with the clock far ahead, so that what is closing has closed.
    pub fn settle(&mut self) {
        self.clock += 10_000;
        for _ in 0..3 {
            self.frame();
        }
    }

    /// Is there an element with this id?
    pub fn has(&self, id: &str) -> bool {
        self.tree.query_by_id(id).is_some()
    }

    /// The bounds of an element: left, top, width, height.
    pub fn bounds(&self, id: &str) -> (f32, f32, f32, f32) {
        let node = self
            .tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no {id}"));
        let b = self.tree.get_absolute_bounds(node).expect("bounds");
        (b.x, b.y, b.width, b.height)
    }

    /// The middle of an element.
    pub fn centre(&self, id: &str) -> (f32, f32) {
        let (x, y, w, h) = self.bounds(id);
        (x + w / 2.0, y + h / 2.0)
    }
}
