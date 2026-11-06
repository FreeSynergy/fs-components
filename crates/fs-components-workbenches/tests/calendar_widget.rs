//! `CalendarWidget`: the real element, clicked in-process over the router and hit test a window uses,
//! and drawn offscreen.
//!
//! Pattern: input tests + pixel tests (skipped with a message without a graphics adapter). No window is
//! opened. A hang here is a deadlock, not slowness.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::path::PathBuf;
use std::sync::Arc;

use calendar_common::harness::{Harness, serial};
use calendar_common::{Diary, d};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::calendar::{CalendarLabels, CalendarWidget, Shown, ViewRange};

const ADA: &str = "item-ada@2026-11-04";
const BOB: &str = "item-bob@2026-11-20";

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
fn clicking_moves_the_calendar_and_the_view_buttons_change_it() {
    let (_, mut h) = harness();
    assert!(h.has(ADA));
    h.click("next");
    assert_eq!(h.widget.with_state(|s| s.anchor()), d("2026-12-06"));
    assert!(!h.has("day-2026-11-20") && h.has("day-2026-12-20"));
    h.click("today");
    assert_eq!(h.widget.with_state(|s| s.anchor()), d("2026-11-06"));
    h.click("view-week");
    assert_eq!(h.widget.with_state(|s| s.view()), ViewRange::Week);
    assert!(h.has(ADA) && !h.has(BOB));
    h.click("view-day");
    assert_eq!(h.widget.with_state(|s| s.view()), ViewRange::Day);
    h.click("view-agenda");
    assert!(h.has(BOB));
    h.click("previous");
    assert_eq!(h.widget.with_state(|s| s.anchor()), d("2026-10-07"));
}

#[test]
fn a_click_on_an_item_opens_it_and_a_click_on_a_day_chooses_the_day_only() {
    let (_, mut h) = harness();
    h.click("day-2026-11-11");
    assert_eq!(selected(&h), None);
    assert_eq!(
        h.widget.with_state(|s| s.selected_day()),
        Some(d("2026-11-11"))
    );
    h.click(ADA);
    assert_eq!(selected(&h).as_deref(), Some("ada"));
    assert_eq!(
        h.widget.with_state(|s| s.selected_day()),
        Some(d("2026-11-11")),
        "not the day under it"
    );
    assert!(h.has("edit") && h.has("delete"));
}

#[test]
fn edit_press_a_flag_save_by_mouse() {
    let (diary, mut h) = harness();
    h.click(ADA);
    h.click("edit");
    assert!(h.has("save") && !h.has("edit") && !h.has("delete"));
    h.click("field-all_day");
    assert_eq!(
        h.widget
            .with_state(|s| s
                .editing()
                .unwrap()
                .draft
                .value("all_day")
                .map(str::to_string))
            .as_deref(),
        Some("true")
    );
    h.widget.with_state(|s| s.set_field("title", "Ada L."));
    h.rebuild();
    assert_eq!(
        h.widget
            .with_state(|s| s
                .editing()
                .unwrap()
                .draft
                .value("title")
                .map(str::to_string))
            .as_deref(),
        Some("Ada L."),
        "a redraw keeps it"
    );
    h.click("save");
    assert_eq!(diary.title("ada").as_deref(), Some("Ada L."));
    assert!(h.has("edit"));
}

#[test]
fn a_new_item_goes_by_mouse() {
    let (diary, mut h) = harness();
    h.click("day-2026-11-11");
    h.click("create");
    assert!(h.has("save") && !h.has("create"));
    h.widget.with_state(|s| s.set_field("title", "Made"));
    h.click("save");
    assert_eq!(diary.title("Made").as_deref(), Some("Made"));
    assert!(h.has("item-Made@2026-11-11"));
    assert_eq!(selected(&h).as_deref(), Some("Made"));
}

#[test]
fn delete_takes_two_different_clicks_in_different_places() {
    let (diary, mut h) = harness();
    h.click(ADA);
    h.click("delete");
    assert!(h.has("keep") && diary.title("ada").is_some());
    h.click("really");
    assert!(diary.title("ada").is_none());
    assert!(!h.has(ADA), "it left the calendar");
    assert_eq!(selected(&h), None);
    assert!(!h.has("keep"));
}

#[test]
fn a_double_click_on_delete_deletes_nothing_with_or_without_a_redraw_between() {
    let (diary, mut h) = harness();
    h.click(ADA);
    let point = h.centre("delete");
    // A window that has not drawn again: the same button is pressed twice.
    h.press_at(point);
    h.press_at(point);
    assert!(diary.title("ada").is_some());
    h.rebuild();
    assert!(h.has("keep"), "still only asking");
    assert!(
        h.centre("really").0 > point.0 + 100.0,
        "the deed is far from the pointer"
    );
    // A window that has: what is under the pointer now keeps the item.
    h.press_at(point);
    h.rebuild();
    assert!(diary.title("ada").is_some());
    assert!(
        h.has("delete") && !h.has("keep"),
        "the second press said no"
    );
    assert_eq!(selected(&h).as_deref(), Some("ada"));
}

#[test]
fn the_compact_variant_offers_nothing_that_changes_and_narrow_the_item_has_a_way_back() {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-14"), ("bob", "2026-11-20")]));
    let compact = CalendarWidget::compact(diary, CalendarLabels::default());
    let mut h = Harness::new(compact, 500.0, 500.0);
    assert!(h.has("item-ada@2026-11-14"));
    assert!(!h.has("create") && !h.has("view-month") && !h.has("edit"));
    h.click("item-ada@2026-11-14");
    assert_eq!(selected(&h).as_deref(), Some("ada"));
    assert!(!h.has("edit") && !h.has("delete") && h.has("back"));
    assert!(
        !h.has("day-2026-11-14"),
        "the item takes the calendar's place"
    );
    h.click("back");
    assert!(h.has("item-bob@2026-11-20"));
    assert!(h.widget.with_state(|s| matches!(s.shown(), Shown::Nothing)));
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
