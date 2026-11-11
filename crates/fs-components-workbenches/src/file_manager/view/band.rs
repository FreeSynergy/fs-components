//! The band at the bottom: the question on the table, the line being typed, what is being carried.
//!
//! Pattern: none (pure functions of what is shown). The second act of something destructive is here, and only here, far from
//! where the first press was, at every width. The way out comes first, so a press that lands on the
//! first button keeps things as they are.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_cn::components::input::{InputSize, input};
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::{Act, FileActions, wrapping_button};
use super::band_size::{band_height, carrying_text, room};
use crate::file_manager::ask::{Ask, InputAsk};
use crate::file_manager::carry::CarryKind;
use crate::file_manager::labels::FileLabels;
use crate::file_manager::state::FilesState;
use crate::input_slots::InputSlots;

/// A button of the band, held to `room(width)` so that none leaves the window.
fn band_button(width: f32, id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    wrapping_button(id, label, variant, act, room(width))
}

/// The band, if there is something to ask.
pub(in crate::file_manager) fn band(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    slots: &InputSlots,
    width: f32,
) -> Option<Div> {
    if band_height(state, width) <= 0.0 {
        return None;
    }
    let mut area = div().class("fs-files-band-inner").flex_col().w(width);
    if let Some(text_of) = carrying_text(state) {
        area = area
            .child(
                div()
                    .id(format!("{name}-carrying"))
                    .w(width - 32.0)
                    .child(text(text_of)),
            )
            .child(carrying_buttons(state, name, actions, width));
    }
    match state.ask() {
        Ask::Input(input) => area = input_area(state, input, name, actions, slots, area, width),
        Ask::Confirm(_) => {
            if let Some((question, yes)) = state.confirm_words() {
                area = area.child(question_line(name, question, width)).child(
                    row()
                        .child(band_button(
                            width,
                            format!("{name}-keep-it"),
                            &state.labels().keep,
                            ButtonVariant::Secondary,
                            &actions.plain(FilesState::keep),
                        ))
                        .child(band_button(
                            width,
                            format!("{name}-really"),
                            &yes,
                            ButtonVariant::Destructive,
                            &actions.plain(FilesState::really),
                        )),
                );
            }
        }
        Ask::NameTaken(question) => {
            let labels = state.labels();
            let words = FileLabels::fill(&labels.name_taken_question, &question.message);
            area = area.child(question_line(name, words, width)).child(
                row()
                    .child(band_button(
                        width,
                        format!("{name}-back"),
                        &labels.back,
                        ButtonVariant::Secondary,
                        &actions.plain(FilesState::name_taken_back),
                    ))
                    .child(band_button(
                        width,
                        format!("{name}-keep-both"),
                        &labels.keep_both,
                        ButtonVariant::Primary,
                        &actions.plain(FilesState::keep_both),
                    )),
            );
        }
        _ => {}
    }
    // The box of the band: exactly as high as the estimate says and clipping what is taller, which is why the
    // estimate must not be low (`files_band_buttons.rs` compares the buttons with this box).
    Some(
        div()
            .id(format!("{name}-band"))
            .class("fs-files-band")
            .flex_shrink_0()
            .w(width)
            .h(band_height(state, width))
            .overflow_clip()
            .child(area),
    )
}

fn row() -> Div {
    div().class("fs-files-band-row").flex_row().flex_wrap()
}

fn question_line(name: &str, words: String, width: f32) -> Div {
    div()
        .id(format!("{name}-question"))
        .w(width - 32.0)
        .child(text(words))
}

fn carrying_buttons(state: &FilesState, name: &str, actions: &FileActions, width: f32) -> Div {
    let labels = state.labels();
    let mut bar = row().child(band_button(
        width,
        format!("{name}-stop-carrying"),
        &labels.cancel,
        ButtonVariant::Secondary,
        &actions.plain(FilesState::stop_carrying),
    ));
    if state.can_put_down() {
        let word = match state.carrying().map(|c| c.kind) {
            Some(CarryKind::Move) => &labels.move_here,
            Some(CarryKind::Copy) => &labels.copy_here,
            _ => &labels.restore_here,
        };
        bar = bar.child(band_button(
            width,
            format!("{name}-put-down"),
            word,
            ButtonVariant::Primary,
            &actions.plain(FilesState::put_down),
        ));
    }
    bar
}

fn input_area(
    state: &FilesState,
    ask: &InputAsk,
    name: &str,
    actions: &FileActions,
    slots: &InputSlots,
    mut area: Div,
    width: f32,
) -> Div {
    let labels = state.labels();
    let words = if ask.discard_asked {
        labels.discard_confirm.clone()
    } else {
        ask.label.clone()
    };
    area = area.child(question_line(name, words, width));
    let data = slots.line(ask.id, "line", &ask.value);
    let typed = Arc::clone(&actions.typed);
    area = area.child(
        div().id(format!("{name}-line")).w(width - 32.0).child(
            input(&data)
                .size(InputSize::Small)
                .w_full()
                .on_change(move |value| typed(value)),
        ),
    );
    if let Some(error) = &ask.error {
        area = area.child(
            div()
                .id(format!("{name}-input-error"))
                .class("fs-files-failed-line")
                .w(width - 32.0)
                .child(text(error.clone())),
        );
    }
    let mut bar = row();
    if ask.discard_asked {
        bar = bar
            .child(band_button(
                width,
                format!("{name}-keep-typing"),
                &labels.keep_typing,
                ButtonVariant::Secondary,
                &actions.plain(FilesState::keep_typing),
            ))
            .child(band_button(
                width,
                format!("{name}-discard"),
                &labels.discard,
                ButtonVariant::Destructive,
                &actions.plain(FilesState::discard_input),
            ));
    } else {
        bar = bar
            .child(band_button(
                width,
                format!("{name}-cancel"),
                &labels.cancel,
                ButtonVariant::Secondary,
                &actions.plain(FilesState::cancel_input),
            ))
            .child(band_button(
                width,
                format!("{name}-apply"),
                &labels.apply,
                ButtonVariant::Primary,
                &actions.plain(FilesState::submit_input),
            ));
    }
    area.child(bar)
}
