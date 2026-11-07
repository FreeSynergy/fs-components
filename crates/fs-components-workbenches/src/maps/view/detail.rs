//! The place: read, or the form while editing.
//!
//! Pattern: Strategy by state — a place, the form, a failure or nothing. Actions the source cannot do
//! are not offered. Asking and confirming a delete are two actions in different places: what lies
//! where Delete was keeps the place, and the deed is across the bar.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::list_view::fit_cell;

use super::actions::MapActions;
use super::detail_bar::bar;
use super::form::form;
use super::page_scroll::PageScroll;
use crate::maps::model::{PlaceSheet, SheetSection};
use crate::maps::shown::Shown;
use crate::maps::state::MapState;
use crate::input_slots::InputSlots;

/// A title is larger than the text `fit_cell` measures, by about this.
const TITLE_FIT: f32 = 0.7;
/// What a mark takes of the line, about.
const MARK_ROOM: f32 = 100.0;
/// The most of a failure that is pinned, in characters; a failure quotes what was typed.
const LONGEST_FAILURE: usize = 600;

/// The detail area. `back` adds the way back where the calendar is out of sight.
pub(super) fn detail(
    state: &MapState,
    name: &str,
    actions: &MapActions,
    slots: &InputSlots,
    page_scroll: &PageScroll,
    back: bool,
    width: f32,
) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .h_full()
        .min_h(0.0)
        .child(bar(state, name, actions, back));
    if let Some(notice) = state.notice() {
        column = column.child(div().class("fs-map-notice").child(text(notice)));
    }
    // Marks and the reason of a failed save stay in view, however far the page is scrolled.
    if let Some(pinned) = pinned(state, name, width) {
        column = column.child(pinned);
    }
    let page = div()
        .id(format!("{name}-page"))
        .class("fs-map-page")
        .flex_col()
        .w_full();
    let page = if let Some(edit) = state.editing() {
        form(edit, state.labels(), name, actions, slots, page)
    } else {
        match state.shown() {
            Shown::Place { sheet, .. } => place(sheet, page, width),
            Shown::Failed { reason, .. } => page.child(
                div()
                    .class("fs-map-failed")
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

/// What must not scroll away: the title and marks of a place, the marks and failure of an edit.
#[allow(clippy::cast_precision_loss)]
fn pinned(state: &MapState, name: &str, width: f32) -> Option<Div> {
    let area = div()
        .id(format!("{name}-pinned"))
        .class("fs-map-pinned")
        .flex_col()
        .w_full();
    if let Some(edit) = state.editing() {
        if edit.draft.marks.is_empty() && edit.error.is_none() {
            return None;
        }
        let mut area = area;
        if !edit.draft.marks.is_empty() {
            area = area.child(with_marks(
                div().class("fs-map-title").flex_row(),
                &edit.draft.marks,
            ));
        }
        if let Some(error) = &edit.error {
            area = area.child(
                div()
                    .class("fs-map-failed")
                    .child(text(cut_to(error, LONGEST_FAILURE))),
            );
        }
        return Some(area);
    }
    let Shown::Place { sheet, .. } = state.shown() else {
        return None;
    };
    // One line, however long the title: the page below must stay reachable.
    let room = width * TITLE_FIT - MARK_ROOM * sheet.marks.len() as f32;
    let shown = fit_cell(&sheet.title, room);
    let head = div()
        .class("fs-map-title")
        .flex_row()
        .child(div().child(text(shown).no_wrap().class("fs-map-title-text")));
    Some(area.child(with_marks(head, &sheet.marks)))
}

/// `text` cut to `most` characters.
fn cut_to(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(most).collect();
    cut.push('\u{2026}');
    cut
}

#[allow(clippy::cast_precision_loss)]
fn place(sheet: &PlaceSheet, mut page: Div, width: f32) -> Div {
    // What the pinned line had to cut can be read here, and scrolls.
    let room = width * TITLE_FIT - MARK_ROOM * sheet.marks.len() as f32;
    if fit_cell(&sheet.title, room) != sheet.title {
        page = page.child(div().class("fs-map-value").child(text(&sheet.title)));
    }
    if let Some(note) = &sheet.note {
        page = page.child(div().class("fs-map-note").child(text(note)));
    }
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-map-sheet-mark").child(text(mark)));
    }
    row
}

fn section_element(section: &SheetSection) -> Div {
    let mut element = div().class("fs-map-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-map-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-map-field")
                .flex_row()
                .child(div().class("fs-map-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-map-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
