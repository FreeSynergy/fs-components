//! A double-click neither deletes nor discards, through the router and hit test a window uses.
//!
//! Tests: input tests, with and without a redraw between the presses. No window is opened.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::Diary;
use calendar_common::harness::Harness;
use fs_components_workbenches::calendar::{CalendarLabels, CalendarWidget};

const ADA: &str = "item-ada@2026-11-04";

fn harness() -> (Arc<Diary>, Harness) {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04"), ("bob", "2026-11-20")]));
    let w = CalendarWidget::new(diary.clone(), CalendarLabels::default());
    (diary, Harness::new(w, 1300.0, 800.0))
}

fn selected(h: &Harness) -> Option<String> {
    h.widget.with_state(|s| s.selected().map(str::to_string))
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
fn a_double_click_on_cancel_discards_nothing_with_or_without_a_redraw_between() {
    let (diary, mut h) = harness();
    h.click(ADA);
    h.click("edit");
    h.widget.with_state(|s| s.set_field("title", "Typed"));
    h.rebuild();
    let point = h.centre("cancel");
    h.press_at(point);
    h.press_at(point);
    let value = |h: &Harness| {
        h.widget.with_state(|s| {
            s.editing()
                .and_then(|e| e.draft.value("title").map(str::to_string))
        })
    };
    assert_eq!(value(&h).as_deref(), Some("Typed"));
    h.rebuild();
    assert!(h.has("keep-editing") && h.has("discard"), "only asking");
    assert!(
        (h.centre("discard").0 - point.0).abs() > 100.0,
        "the deed is far from the pointer"
    );
    h.press_at(point);
    h.rebuild();
    assert_eq!(
        value(&h).as_deref(),
        Some("Typed"),
        "the second press said keep"
    );
    assert!(h.has("cancel") && !h.has("discard"));
    h.click("cancel");
    h.click("discard");
    assert_eq!(value(&h), None);
    assert_eq!(diary.title("ada").as_deref(), Some("ada"));
}

#[test]
fn a_double_click_on_edit_leaves_the_form_open() {
    let (_, mut h) = harness();
    h.click(ADA);
    let point = h.centre("edit");
    h.click("edit");
    assert!(h.widget.with_state(|s| s.editing().is_some()));
    assert!(
        (h.centre("save").0 - point.0).abs() > 200.0
            && (h.centre("cancel").0 - point.0).abs() > 150.0,
        "neither lies where Edit was"
    );
    h.press_at(point);
    h.rebuild();
    assert!(
        h.widget.with_state(|s| s.editing().is_some()),
        "the second press did nothing"
    );
}
