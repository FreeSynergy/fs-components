//! The rows of the page tree as drawn: where the title of a child stands, and what a click on each part does.
//!
//! Pattern: none (tests of `tree_scroll` laid out in process, the pointer fed like a window feeds it).

use std::sync::{Arc, Mutex};

use blinc_core::BlincContextState;
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::RenderTree;

use super::rows::{Look, tree_scroll};
use super::tree::PageTree;
use crate::source::Entry;
use crate::view::Actions;

type Log = Arc<Mutex<Vec<String>>>;

fn entries() -> Vec<Entry> {
    vec![
        Entry::new("handbook.md", "Handbook"),
        Entry::new("handbook/chapter-one.md", "Chapter one"),
        Entry::new("notes.md", "Notes"),
    ]
}

fn actions(log: &Log) -> Actions {
    let (open, toggle) = (Arc::clone(log), Arc::clone(log));
    Actions {
        open: Arc::new(move |k| open.lock().unwrap().push(format!("open {k}"))),
        toggle: Arc::new(move |k| toggle.lock().unwrap().push(format!("toggle {k}"))),
        ..Actions::inert()
    }
}

/// The tree laid out in 320 x 400 with the style sheet of the reader.
fn laid_out(tree: &PageTree, look: Look, log: &Log) -> RenderTree {
    let css = crate::view::styles();
    let bundle = fs_components_render::bundle(&css);
    let sources = bundle.css_sources.clone();
    blinc_theme::ThemeState::init(bundle, blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            blinc_core::reactive::global_graph(),
            Arc::new(Mutex::new(blinc_core::HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    blinc_layout::init_text_measurer();
    let ui = blinc_layout::div::div()
        .w(320.0)
        .h(400.0)
        .child(tree_scroll(tree, None, &actions(log), look));
    let mut built = RenderTree::from_element(&ui);
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let mut sheet = String::new();
    for part in &sources {
        sheet.push_str(part);
        sheet.push('\n');
    }
    built.set_stylesheet(Stylesheet::parse_with_variables(&sheet, &vars).expect("sheet"));
    built.apply_stylesheet_layout_overrides();
    built.apply_stylesheet_base_styles();
    built.compute_layout(320.0, 400.0);
    built
}

fn x_of(tree: &RenderTree, id: &str) -> f32 {
    let node = tree.query_by_id(id).unwrap_or_else(|| panic!("no {id}"));
    tree.get_absolute_bounds(node).expect("bounds").x
}

/// A click at a point, the way the window's router delivers it.
fn click(tree: &mut RenderTree, x: f32, y: f32) {
    use blinc_layout::event_router::{EventRouter, MouseButton::Left};
    let mut router = EventRouter::new();
    let _ = router.on_mouse_move(tree, x, y);
    let down = router.on_mouse_down(tree, x, y, Left);
    for (node, event) in down {
        tree.dispatch_event(node, event, x, y);
    }
    let up = router.on_mouse_up(tree, x, y, Left);
    for (node, event) in up {
        tree.dispatch_event(node, event, x, y);
    }
}

fn row_y(tree: &RenderTree, key: &str) -> f32 {
    let node = tree.query_by_id(&format!("fs-wk-row-{key}")).expect("row");
    let b = tree.get_absolute_bounds(node).expect("bounds");
    b.y + b.height / 2.0
}

fn open_handbook() -> PageTree {
    let mut tree = PageTree::new(&entries());
    tree.toggle("handbook.md");
    tree
}

#[test]
fn a_child_stands_one_step_right_of_its_parent() {
    let log = Log::default();
    for look in [Look::Icons, Look::Quiet] {
        let laid = laid_out(&open_handbook(), look, &log);
        let parent = x_of(&laid, "fs-wk-arrow-handbook.md");
        let child = x_of(&laid, "fs-wk-arrow-handbook/chapter-one.md");
        // The step is the width of the child's indent box: exactly one `--space-3`, no extra gap on top.
        let node = laid
            .query_by_id("fs-wk-steps-handbook/chapter-one.md")
            .expect("indent box");
        let step = laid.get_absolute_bounds(node).expect("bounds").width;
        assert!(step > 8.0, "the step is missing ({step})");
        assert!(
            ((child - parent) - step).abs() < 0.5,
            "the child is not exactly one step right ({child} against {parent}, step {step})"
        );
    }
}

#[test]
fn opening_a_parent_does_not_move_its_title() {
    let log = Log::default();
    let closed = laid_out(&PageTree::new(&entries()), Look::Icons, &log);
    let open = laid_out(&open_handbook(), Look::Icons, &log);
    let (a, b) = (
        x_of(&closed, "fs-wk-title-handbook.md"),
        x_of(&open, "fs-wk-title-handbook.md"),
    );
    assert!((a - b).abs() < 0.5, "the title moved from {a} to {b}");
    // A page without sub-pages keeps the same place for its title.
    let leaf = x_of(&open, "fs-wk-row-notes.md");
    assert!(leaf <= a + 0.5);
}

#[test]
fn a_click_on_the_free_rest_of_the_row_opens_the_page_the_arrow_expands() {
    let log = Log::default();
    let mut laid = laid_out(&PageTree::new(&entries()), Look::Icons, &log);
    let y = row_y(&laid, "handbook.md");
    click(&mut laid, 300.0, y); // far right of the title: the free rest
    assert_eq!(*log.lock().unwrap(), ["open handbook.md"]);
    log.lock().unwrap().clear();
    let arrow = x_of(&laid, "fs-wk-arrow-handbook.md") + 7.0;
    click(&mut laid, arrow, y);
    assert_eq!(*log.lock().unwrap(), ["toggle handbook.md"]);
    log.lock().unwrap().clear();
    let title = x_of(&laid, "fs-wk-title-handbook.md") + 40.0;
    click(&mut laid, title, y);
    assert_eq!(*log.lock().unwrap(), ["open handbook.md"]);
    log.lock().unwrap().clear();
    let leaf = row_y(&laid, "notes.md");
    click(&mut laid, 300.0, leaf);
    assert_eq!(*log.lock().unwrap(), ["open notes.md"]);
}
