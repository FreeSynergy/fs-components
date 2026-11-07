//! `CalendarWidget`: no room, the reactive graph over rebuilds, a scrolled axis, and pictures.
//!
//! Tests: input tests + pixel tests (skipped with a message without a graphics adapter). No window is
//! opened. A hang here is a deadlock, not slowness.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::path::PathBuf;
use std::sync::Arc;

use calendar_common::Diary;
use calendar_common::harness::{Harness, serial};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::calendar::{CalendarLabels, CalendarWidget, ItemProblem, ViewRange};

const ADA: &str = "item-ada@2026-11-04";

fn widget() -> (Arc<Diary>, CalendarWidget) {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")]));
    (
        diary.clone(),
        CalendarWidget::new(diary, CalendarLabels::default()),
    )
}

fn harness() -> (Arc<Diary>, Harness) {
    let (diary, w) = widget();
    (diary, Harness::new(w, 1300.0, 800.0))
}

fn selected(h: &Harness) -> Option<String> {
    h.widget.with_state(|s| s.selected().map(str::to_string))
}

#[test]
fn no_room_at_all_does_not_crash() {
    let (_, w) = widget();
    let _h = Harness::new(w, 0.0, 0.0);
}

#[test]
fn rebuilding_as_a_window_does_keeps_the_reactive_graph_bounded() {
    let (_, mut h) = harness();
    let stats = || blinc_core::reactive::global_graph().lock().unwrap().stats();
    h.click(ADA);
    for state in ["read", "form"] {
        h.rebuild();
        let before = stats();
        for _ in 0..30 {
            h.rebuild();
        }
        let after = stats();
        assert!(
            after.derived_count <= before.derived_count + 5,
            "{state}: {before:?} {after:?}"
        );
        assert!(
            after.signal_count <= before.signal_count + 5,
            "{state}: {before:?} {after:?}"
        );
        if state == "read" {
            h.click("edit");
        }
    }
}

fn draw(widget: &CalendarWidget, name: &str) -> Option<image::RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-calendar-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let canvas = Canvas::new(1300, 800, widget.styles());
    match draw_to_png(&canvas, |w, h| widget.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
                let _ = std::fs::remove_dir(&dir);
            }
            Some(picture)
        }
    }
}

#[test]
fn pictures_of_the_views_and_of_reading_editing_and_a_failure_differ() {
    let _one = serial();
    let (diary, w) = widget();
    let Some(month) = draw(&w, "calendar-wb-month.png") else {
        return;
    };
    w.with_state(|s| s.set_view(ViewRange::Week));
    let week = draw(&w, "calendar-wb-week.png").unwrap();
    assert_ne!(month, week);
    w.with_state(|s| s.select("ada"));
    let read = draw(&w, "calendar-wb-read.png").unwrap();
    assert_ne!(week, read);
    w.with_state(|s| s.begin_edit());
    let form = draw(&w, "calendar-wb-form.png").unwrap();
    assert_ne!(read, form);
    w.with_state(|s| s.set_field("title", "changed"));
    *diary.refuse.lock().unwrap() = Some("the store refuses this".into());
    w.with_state(|s| s.save());
    let failed = draw(&w, "calendar-wb-failed.png").unwrap();
    assert_ne!(form, failed);
}

#[test]
fn an_item_on_the_scrolled_axis_is_clicked_where_it_is_drawn() {
    let (_, w) = widget();
    w.with_state(|s| s.set_view(ViewRange::Week));
    let mut h = Harness::new(w, 1300.0, 800.0);
    assert!(h.widget.with_state(|s| s.view()) == ViewRange::Week);
    // The axis starts at seven: nine o'clock is two hours down, not nine.
    let (_, y) = h.centre(ADA);
    assert!(y < 300.0, "the bounds follow the scroll: {y}");
    h.click(ADA);
    assert_eq!(selected(&h).as_deref(), Some("ada"));
}

#[test]
fn a_mark_stays_in_view_and_a_new_item_starts_at_the_top() {
    let diary = Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-05")]);
    diary.marked.lock().unwrap().push("bob".into());
    let diary = Arc::new(diary);
    let w = CalendarWidget::new(diary, CalendarLabels::default());
    w.with_state(|s| s.select("ada"));
    let mut h = Harness::new(w, 1300.0, 300.0);
    let at_rest = h.centre("page").1;
    h.click("edit");
    let form_at_rest = h.centre("page").1;
    let scrolled = h.wheel_at((h.centre("page").0, 200.0), 200.0);
    assert!(
        scrolled < form_at_rest - 50.0,
        "the form did scroll: {form_at_rest} {scrolled}"
    );
    h.widget.with_state(|s| s.cancel_edit());
    h.widget.with_state(|s| s.select("bob"));
    h.rebuild();
    assert!(h.has("pinned"));
    let page = h.centre("page").1;
    assert!(
        (page - at_rest).abs() < 60.0,
        "at the top again: {page} {at_rest}"
    );
    let pinned = h.centre("pinned").1;
    assert!(
        pinned < page,
        "the mark is above the page, out of the scroller"
    );
}

#[test]
fn a_problem_gets_a_line_that_opens_it() {
    let diary = Diary::with(&[("ada", "2026-11-04")]);
    diary.problems.lock().unwrap().extend([
        ItemProblem::of("broken-1", "One cannot be read"),
        ItemProblem::of("ada", "Too many occurrences"),
    ]);
    let w = CalendarWidget::new(Arc::new(diary), CalendarLabels::default());
    let mut h = Harness::new(w, 1300.0, 800.0);
    assert!(h.has(ADA), "the rest is there");
    h.click("problem-1");
    assert_eq!(selected(&h).as_deref(), Some("ada"));
}

#[test]
fn the_context_of_the_widget_is_a_plain_value() {
    let (_, w) = widget();
    w.with_state(|s| s.select("bob"));
    let context = w.context();
    assert_eq!(context.item.as_deref(), Some("bob"));
    assert_eq!(context.view, ViewRange::Month);
}

#[test]
fn a_very_long_title_leaves_the_page_in_reach_and_a_long_sentence_is_cut() {
    let long = "Long title ".repeat(400);
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04")]));
    diary.other_changes("ada", &long);
    let w = CalendarWidget::new(diary, CalendarLabels::default());
    w.with_state(|s| s.select("ada"));
    let h = Harness::new(w, 1300.0, 600.0);
    let name = h.widget.element_name();
    let pinned = h
        .tree
        .get_absolute_bounds(h.tree.query_by_id(&format!("{name}-pinned")).unwrap())
        .unwrap();
    assert!(pinned.height < 100.0, "one line: {pinned:?}");
    let page = h
        .tree
        .get_absolute_bounds(h.tree.query_by_id(&format!("{name}-page")).unwrap())
        .unwrap();
    assert!(page.y < 300.0, "the page starts in view: {page:?}");
    h.widget.with_state(|s| s.ask_delete());
    let question = h.widget.with_state(|s| s.notice().unwrap().to_string());
    assert!(question.chars().count() < 200, "{}", question.len());
}

#[test]
fn a_problem_about_something_marked_carries_its_mark() {
    let diary = Diary::with(&[("ada", "2026-11-04")]);
    diary.problems.lock().unwrap().push(
        ItemProblem::of("ada", "An event cannot be shown").marked(vec!["Sensitive".to_string()]),
    );
    let w = CalendarWidget::new(Arc::new(diary), CalendarLabels::default());
    let h = Harness::new(w, 1300.0, 800.0);
    assert!(h.has("problem-0"));
    assert_eq!(
        h.widget.with_state(|s| s.problems()[0].marks.clone()),
        ["Sensitive"]
    );
}
