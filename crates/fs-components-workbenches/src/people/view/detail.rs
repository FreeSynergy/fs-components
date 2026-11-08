//! The person: read, or the form while editing.
//!
//! Pattern: none (pure functions of what is shown) — a person, the form, a failure or nothing. Actions the source cannot
//! do are not offered.

use blinc_cn::cn;
use blinc_cn::components::avatar::{AvatarSize, avatar};
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_widgets::avatar_group_initials;

use super::actions::{Act, PeopleActions};
use super::form::form;
use crate::input_slots::InputSlots;
use crate::page_scroll::PageScroll;
use crate::people::model::{PersonSection, PersonSheet};
use crate::people::shown::Shown;
use crate::people::state::PeopleState;

/// A button with an element id on a wrapper, so a program or a test can find it.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = std::sync::Arc::clone(act);
    div().id(id).w_fit().child(
        cn::button(label.to_string())
            .variant(variant)
            .on_click(move |_| act()),
    )
}

/// A button named `<name>-<what>`.
pub(in crate::people) fn button_for(
    name: &str,
    what: &str,
    label: &str,
    variant: ButtonVariant,
    act: &Act,
) -> Div {
    button(format!("{name}-{what}"), label, variant, act)
}

/// The area right of the list. All of it scrolls, the buttons too: in a small window there is no room
/// to keep them apart from the page.
pub(in crate::people) fn detail(
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    slots: &InputSlots,
    scroller: &PageScroll,
) -> Div {
    let editing = state.editing().is_some();
    // Editing: the buttons stand on top as always, but come last in the tree, so Tab walks the fields
    // in reading order, then Save, then Cancel (a form's usual order). The column is drawn in reverse
    // for that; so is the bar.
    let mut column = div().class("fs-people-detail").w_full();
    column = if editing {
        column.flex_col_reverse()
    } else {
        column.flex_col().child(bar(state, name, actions))
    };
    let page = div().class("fs-people-page").flex_col().w_full();
    let page = if let Some(edit) = state.editing() {
        form(
            edit,
            state.labels(),
            name,
            actions,
            slots,
            &|key| state.problem(key),
            page,
        )
    } else {
        match state.shown() {
            Shown::Person { sheet, .. } => person(sheet, page),
            Shown::Failed { reason, .. } => {
                page.child(div().class("fs-people-failed").child(text(reason.clone())))
            }
            _ => page.child(text(&state.labels().nothing_selected)),
        }
    };
    let notice = state
        .notice()
        .map(|notice| div().class("fs-people-notice").child(text(notice)));
    let deed = deed(state, name, actions);
    if editing {
        // Tree order; drawn from the bottom up.
        column = column.child(page);
        column = column.children(deed);
        column = column.children(notice);
        column = column.child(bar(state, name, actions));
    } else {
        column = column.children(notice).children(deed).child(page);
    }
    let what = match state.editing() {
        Some(edit) => format!("edit-{}", edit.id),
        None => state.selected().unwrap_or_default().to_string(),
    };
    div()
        .flex_1()
        .min_h(0.0)
        .w_full()
        .child(scroller.scroller(&what).w_full().h_full().content(column))
}

/// The second action of a question, on a line of its own below the bar: a second press at the place of
/// the first can never be it, at any width.
fn deed(state: &PeopleState, name: &str, actions: &PeopleActions) -> Option<Div> {
    let labels = state.labels();
    let button = if state.editing().is_some_and(|edit| edit.discard_asked) {
        button(
            format!("{name}-discard"),
            &labels.discard_really,
            ButtonVariant::Destructive,
            &actions.confirm_discard,
        )
    } else if state.can_delete() && state.delete_asked() {
        button(
            format!("{name}-really"),
            &labels.delete_really,
            ButtonVariant::Destructive,
            &actions.confirm_delete,
        )
    } else {
        return None;
    };
    Some(div().class("fs-people-really").flex_row().child(button))
}

/// The buttons that fit the state. Save and Cancel stand at the far side, away from Edit and Delete, so
/// the second press of a double-click on Edit lands on nothing; Keep editing stands where Cancel was.
fn bar(state: &PeopleState, name: &str, actions: &PeopleActions) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-people-bar").flex_row().flex_wrap();
    if let Some(edit) = state.editing() {
        let cancel_or_keep = if edit.discard_asked {
            button(
                id("keep-editing"),
                &labels.discard_keep,
                ButtonVariant::Secondary,
                &actions.keep_editing,
            )
        } else {
            button(
                id("cancel"),
                &labels.cancel,
                ButtonVariant::Secondary,
                &actions.cancel,
            )
        };
        // Tree order Save, Cancel, spacer; drawn in reverse: spacer, Cancel, Save.
        return bar
            .flex_row_reverse()
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(cancel_or_keep)
            .child(div().class("fs-people-spacer"));
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
        bar = bar.child(if state.delete_asked() {
            button(
                id("keep"),
                &labels.delete_keep,
                ButtonVariant::Secondary,
                &actions.keep,
            )
        } else {
            button(
                id("delete"),
                &labels.delete,
                ButtonVariant::Ghost,
                &actions.ask_delete,
            )
        });
    }
    bar
}

fn person(sheet: &PersonSheet, page: Div) -> Div {
    let theme = ThemeState::get();
    let mut face = avatar()
        .size(AvatarSize::ExtraLarge)
        .fallback_bg(theme.color(ColorToken::Primary))
        .fallback_color(theme.color(ColorToken::TextInverse))
        .fallback(avatar_group_initials(&sheet.title))
        .alt(sheet.title.clone());
    if let Some(image) = &sheet.image {
        face = face.src(image.clone());
    }
    let head = div()
        .class("fs-people-head")
        .flex_row()
        .child(div().class("fs-people-face").child(face))
        .child(
            div()
                .class("fs-people-title")
                .child(text(&sheet.title).class("fs-people-title-text")),
        );
    let mut page = page.child(with_marks(head, &sheet.marks));
    for section in &sheet.sections {
        page = page.child(section_element(section));
    }
    page
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-people-mark").child(text(mark)));
    }
    row
}

fn section_element(section: &PersonSection) -> Div {
    let mut element = div().class("fs-people-section").flex_col();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-people-section-title")
                .child(text(&section.title)),
        );
    }
    for field in &section.fields {
        element = element.child(
            div()
                .class("fs-people-field")
                .flex_row()
                .child(div().class("fs-people-label").child(text(&field.label)))
                .child(
                    div()
                        .class("fs-people-value")
                        .flex_1()
                        .child(text(&field.value)),
                ),
        );
    }
    element
}
