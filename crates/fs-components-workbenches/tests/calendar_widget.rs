//! `CalendarWidget`: the real element, clicked in-process over the router and hit test a window uses.
//!
//! Pattern: input tests. No window is opened. A hang here is a deadlock, not slowness.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::harness::Harness;
use calendar_common::{Diary, d};
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
