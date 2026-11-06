//! What the workbench gives back to the program around it (ADR 0056).
//!
//! Pattern: State machine test, no window.

#[path = "calendar_common/mod.rs"]
mod calendar_common;

use std::sync::Arc;

use calendar_common::{Diary, d};
use fs_components_workbenches::calendar::{
    CalendarLabels, CalendarState, Picked, Variant, ViewRange,
};

#[test]
fn the_context_is_what_is_shown_as_a_plain_value() {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04")]));
    let mut s = CalendarState::new(diary, CalendarLabels::default(), Variant::Full);
    let context = s.context();
    assert_eq!(
        (context.item, context.view, context.date),
        (None, ViewRange::Month, d("2026-11-06"))
    );
    s.set_view(ViewRange::Week);
    s.pick(&Picked::Day(d("2026-11-05")));
    s.select("ada");
    let context = s.context();
    assert_eq!(context.item.as_deref(), Some("ada"));
    assert_eq!(
        (context.view, context.date),
        (ViewRange::Week, d("2026-11-05"))
    );
    s.next();
    assert_eq!(
        s.context().date,
        d("2026-11-13"),
        "the day chosen is out of view"
    );
}

#[test]
fn the_context_names_the_item_edited_and_none_while_a_new_one_is_being_made() {
    let diary = Arc::new(Diary::with(&[("ada", "2026-11-04")]));
    let mut s = CalendarState::new(diary, CalendarLabels::default(), Variant::Full);
    s.select("ada");
    s.begin_edit();
    assert_eq!(s.context().item.as_deref(), Some("ada"));
    s.cancel_edit();
    s.begin_create();
    assert_eq!(
        s.context().item,
        None,
        "the form is of a new one, not of what was open"
    );
    s.cancel_edit();
    assert_eq!(
        s.context().item.as_deref(),
        Some("ada"),
        "it is still open behind the form"
    );
}
