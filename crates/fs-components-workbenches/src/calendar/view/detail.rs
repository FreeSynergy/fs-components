//! The item: read, or the form while editing.
//!
//! Pattern: Strategy by state — an item, the form, a failure or nothing. Actions the source cannot do
//! are not offered. Asking and confirming a delete are two actions in different places: what lies
//! where Delete was keeps the item, and the deed is across the bar.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::CalendarActions;
use super::form::{InputSlots, form};
use super::page_scroll::PageScroll;
use super::toolbar::button;
use crate::calendar::model::{ItemSheet, SheetSection};
use crate::calendar::shown::Shown;
use crate::calendar::state::CalendarState;

/// The detail area. `back` adds the way back where the calendar is out of sight.
pub(super) fn detail(
    state: &CalendarState,
    name: &str,
    actions: &CalendarActions,
    slots: &InputSlots,
    page_scroll: &PageScroll,
    back: bool,
) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .child(bar(state, name, actions, back));
    if let Some(notice) = state.notice() {
        column = column.child(div().class("fs-calendar-notice").child(text(notice)));
    }
    // Marks and the reason of a failed save stay in view, however far the page is scrolled.
    if let Some(pinned) = pinned(state, name) {
        column = column.child(pinned);
    }
    let page = div()
        .id(format!("{name}-page"))
        .class("fs-calendar-page")
        .flex_col()
        .w_full();
    let page = if let Some(edit) = state.editing() {
        form(edit, state.labels(), name, actions, slots, page)
    } else {
        match state.shown() {
            Shown::Item { sheet, .. } => item(sheet, page),
            Shown::Failed { reason, .. } => page.child(
                div()
                    .class("fs-calendar-failed")
                    .child(text(reason.clone())),
            ),
            _ => page.child(text(&state.labels().nothing_selected)),
        }
    };
    let what = format!(
        "{:?}/{:?}",
        state.selected(),
        state.editing().map(|edit| edit.id)
    );
    column.child(
        div()
            .flex_1()
            .min_h(0.0)
            .w_full()
            .child(page_scroll.scroller(&what).w_full().h_full().content(page)),
    )
}

/// What must not scroll away: the title and marks of an item, the marks and failure of an edit.
fn pinned(state: &CalendarState, name: &str) -> Option<Div> {
    let area = div()
        .id(format!("{name}-pinned"))
        .class("fs-calendar-pinned")
        .flex_col()
        .w_full();
    if let Some(edit) = state.editing() {
        if edit.draft.marks.is_empty() && edit.error.is_none() {
            return None;
        }
        let mut area = area;
        if !edit.draft.marks.is_empty() {
            area = area.child(with_marks(
                div().class("fs-calendar-title").flex_row(),
                &edit.draft.marks,
            ));
        }
        if let Some(error) = &edit.error {
            area = area.child(div().class("fs-calendar-failed").child(text(error)));
        }
        return Some(area);
    }
    let Shown::Item { sheet, .. } = state.shown() else {
        return None;
    };
    let head = div()
        .class("fs-calendar-title")
        .flex_row()
        .child(div().child(text(&sheet.title).class("fs-calendar-title-text")));
    Some(area.child(with_marks(head, &sheet.marks)))
}

/// The buttons that fit the state.
fn bar(state: &CalendarState, name: &str, actions: &CalendarActions, back: bool) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-calendar-bar").flex_row().items_center();
    if let Some(edit) = state.editing() {
        bar = bar
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(cancel_or_keep(state, name, actions));
        if edit.discard_asked {
            // The deed is across the bar, so a second press at the same point can never be it.
            bar = bar.child(div().flex_1()).child(button(
                id("discard"),
                &labels.discard_really,
                ButtonVariant::Destructive,
                &actions.confirm_discard,
            ));
        }
        return bar;
    }
    if back {
        bar = bar.child(button(
            id("back"),
            &labels.back,
            ButtonVariant::Ghost,
            &actions.back,
        ));
    }
    if state.can_edit() {
        bar = bar.child(button(
            id("edit"),
            &labels.edit,
            ButtonVariant::Secondary,
            &actions.edit,
        ));
    }
    if state.can_delete() {
        if state.delete_asked() {
            bar = bar
                .child(button(
                    id("keep"),
                    &labels.delete_keep,
                    ButtonVariant::Secondary,
                    &actions.keep,
                ))
                .child(div().flex_1())
                .child(button(
                    id("really"),
                    &labels.delete_really,
                    ButtonVariant::Destructive,
                    &actions.confirm_delete,
                ));
        } else {
            bar = bar.child(button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.ask_delete,
            ));
        }
    }
    bar
}

/// What lies where Cancel is: after the question it keeps the edit.
fn cancel_or_keep(state: &CalendarState, name: &str, actions: &CalendarActions) -> Div {
    let labels = state.labels();
    if state.editing().is_some_and(|edit| edit.discard_asked) {
        button(
            format!("{name}-keep-editing"),
            &labels.discard_keep,
            ButtonVariant::Secondary,
            &actions.keep_editing,
        )
    } else {
        button(
            format!("{name}-cancel"),
            &labels.cancel,
            ButtonVariant::Secondary,
            &actions.cancel,
        )
    }
}

fn item(sheet: &ItemSheet, mut page: Div) -> Div {
    if let Some(note) = &sheet.note {
        page = page.child(div().class("fs-calendar-note").child(text(note)));
    }
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-calendar-sheet-mark").child(text(mark)));
    }
    row
}

fn section_element(section: &SheetSection) -> Div {
    let mut element = div().class("fs-calendar-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-calendar-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-calendar-field")
                .flex_row()
                .child(div().class("fs-calendar-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-calendar-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
