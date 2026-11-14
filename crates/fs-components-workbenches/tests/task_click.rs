//! Checkboxes in the reading view: a click ticks or unticks them and the source stores it as an ordinary change; without the
//! right to change the page they are drawn and locked (finding W4 of 2026-11-14).
//!
//! Tests: event tests on the built widget in all three layouts (a mouse click through the window's router and hit test); the
//! source records every call, so "no operation without the right" is a count. Pictures before and after are kept when
//! `FS_COMP_SHOTS` names a directory.

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use common::Fake;
use fs_components_workbenches::{
    Block, Document, DocumentSource, Entry, Hit, Inline, Labels, Layout, LinkTarget, ListItem,
    Picture, ReaderWidget, SourceError, WikiLabels, snapshot,
};

/// A page with three list tasks and two table cells with a box. `states` are the five boxes in document order.
struct Tasks {
    states: Mutex<[bool; 5]>,
    calls: Mutex<Vec<(usize, bool)>>,
    may: AtomicBool,
}

impl Tasks {
    fn new(may: bool) -> Self {
        Self {
            states: Mutex::new([false, true, false, false, true]),
            calls: Mutex::default(),
            may: AtomicBool::new(may),
        }
    }
}

impl DocumentSource for Tasks {
    fn entries(&self, _prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(vec![Entry::new("p.md", "P")])
    }
    fn read(&self, path: &str) -> Result<Document, SourceError> {
        let s = *self.states.lock().unwrap();
        let item =
            |t: &str, done| ListItem::new(vec![Block::Paragraph(Inline::plain(t))]).task(done);
        let cell = |done: bool| Inline::plain(if done { "[x]" } else { "[ ]" });
        Ok(Document::new(
            path,
            vec![
                Block::Heading {
                    level: 1,
                    text: Inline::plain("P"),
                },
                Block::List {
                    ordered: false,
                    start: 1,
                    items: vec![item("one", s[0]), item("two", s[1]), item("three", s[2])],
                },
                Block::Table {
                    align: vec![],
                    rows: vec![
                        vec![Inline::plain("a"), Inline::plain("b")],
                        vec![Inline::plain("x"), cell(s[3])],
                        vec![Inline::plain("y"), cell(s[4])],
                    ],
                },
            ],
        ))
    }
    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        LinkTarget::Missing(href.into())
    }
    fn find(&self, _term: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(Vec::new())
    }
    fn can_edit(&self, _path: &str) -> bool {
        self.may.load(Ordering::SeqCst)
    }
    fn set_task(
        &self,
        _path: &str,
        index: usize,
        done: bool,
        _revision: Option<&str>,
    ) -> Result<(), SourceError> {
        self.calls.lock().unwrap().push((index, done));
        self.states.lock().unwrap()[index] = done;
        Ok(())
    }
}

fn init() {
    let tmp = std::env::temp_dir().join(format!("fs-task-{}.png", std::process::id()));
    let _ = snapshot(Picture::new(Arc::new(Fake), 200, 100), &tmp);
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
}

fn opened(layout: Layout, may: bool) -> (Arc<Tasks>, ReaderWidget, RenderTree) {
    init();
    let source = Arc::new(Tasks::new(may));
    let widget = ReaderWidget::new(source.clone(), Labels::default(), None)
        .layout(layout, WikiLabels::default());
    widget.with_reader(|r| r.open("p.md"));
    let mut tree = RenderTree::from_element(&widget.build(1000.0, 700.0));
    tree.compute_layout(1000.0, 700.0);
    (source, widget, tree)
}

/// A click with the press and release a mouse sends, on the element with this id, then the redraw it asks for.
fn click(tree: &mut RenderTree, id: &str) {
    let node = tree
        .query_by_id(id)
        .unwrap_or_else(|| panic!("no element {id}"));
    let b = tree.get_absolute_bounds(node).expect("bounds");
    let (x, y) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    let mut router = EventRouter::new();
    for events in [
        router.on_mouse_move(tree, x, y),
        router.on_mouse_down(tree, x, y, MouseButton::Left),
        router.on_mouse_up(tree, x, y, MouseButton::Left),
    ] {
        for (n, e) in events {
            tree.dispatch_event(n, e, x, y);
        }
    }
    tree.process_pending_subtree_rebuilds();
    tree.compute_layout(1000.0, 700.0);
}

#[test]
fn a_click_ticks_and_unticks_a_list_task_and_a_table_cell_in_every_layout() {
    for layout in [Layout::Classic, Layout::Confluence, Layout::Notion] {
        let (source, widget, mut tree) = opened(layout, true);
        // Box 0 is open, box 1 is done, box 3 (table cell) is open, box 4 (table cell) is done.
        click(&mut tree, "fs-tv-task-0");
        click(&mut tree, "fs-tv-task-1");
        click(&mut tree, "fs-tv-task-3");
        click(&mut tree, "fs-tv-task-4");
        assert_eq!(
            *source.calls.lock().unwrap(),
            [(0, true), (1, false), (3, true), (4, false)],
            "{layout:?}"
        );
        assert_eq!(
            *source.states.lock().unwrap(),
            [true, false, false, true, false],
            "{layout:?}"
        );
        // The page was read again: a click on box 0 now unticks it.
        click(&mut tree, "fs-tv-task-0");
        assert_eq!(
            source.calls.lock().unwrap().last(),
            Some(&(0, false)),
            "{layout:?}"
        );
        assert!(
            widget.with_reader(|r| r.editing().is_none()),
            "a click on a box starts no edit ({layout:?})"
        );
    }
}

#[test]
fn without_the_right_the_boxes_are_drawn_locked_and_a_click_changes_nothing() {
    for layout in [Layout::Classic, Layout::Confluence, Layout::Notion] {
        let (source, widget, mut tree) = opened(layout, false);
        assert!(
            tree.query_by_id("fs-tv-task-1").is_some(),
            "the state is drawn ({layout:?})"
        );
        click(&mut tree, "fs-tv-task-1");
        click(&mut tree, "fs-tv-task-3");
        assert!(
            source.calls.lock().unwrap().is_empty(),
            "no operation without the right ({layout:?})"
        );
        assert_eq!(
            *source.states.lock().unwrap(),
            [false, true, false, false, true]
        );
        assert!(!widget.with_reader(|r| r.can_tick()));
    }
}

#[test]
fn the_pictures_before_and_after_a_tick() {
    use common::draw::{Scratch, fonts_work, pixels};
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let mut shots = Vec::new();
    for (name, ticked, may) in [
        ("aufgabe-klick-vorher.png", false, true),
        ("aufgabe-klick-nachher.png", true, true),
        ("aufgabe-gesperrt.png", false, false),
    ] {
        let source = Tasks::new(may);
        if ticked {
            source.set_task("p.md", 0, true, None).unwrap();
            source.set_task("p.md", 3, true, None).unwrap();
        }
        let mut picture = Picture::new(Arc::new(source), 900, 600);
        picture.layout = Layout::Confluence;
        picture.steps = vec![fs_components_workbenches::Step::Open("p.md".into())];
        let file = tmp.file(name);
        if snapshot(picture, &file).is_err() {
            eprintln!("skipped: no graphics");
            return;
        }
        if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
        }
        shots.push(pixels(&file));
    }
    assert_ne!(
        shots[0].as_raw(),
        shots[1].as_raw(),
        "a ticked box looks different"
    );
}
