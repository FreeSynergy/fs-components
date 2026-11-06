//! `CalendarWidget`: no room, the reactive graph over rebuilds, a scrolled axis, and pictures.
//!
//! Pattern: input tests + pixel tests (skipped with a message without a graphics adapter). No window is
//! opened. A hang here is a deadlock, not slowness.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::path::PathBuf;
use std::sync::Arc;

use calendar_common::Diary;
use calendar_common::harness::{Harness, serial};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::calendar::{CalendarLabels, CalendarWidget, ViewRange};

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
