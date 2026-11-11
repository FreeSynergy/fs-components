//! How tall the band is: the question, the buttons (their words wrap) and the line being typed.
//!
//! Pattern: none (pure functions of what is shown). Split from `band.rs` for the file limit.

use crate::file_manager::ask::Ask;
use crate::file_manager::carry::CarryKind;
use crate::file_manager::labels::FileLabels;
use crate::file_manager::state::FilesState;

use super::frame::text_height;

/// Height of a line of buttons, and of the text field.
const BUTTONS: f32 = 44.0;
const FIELD: f32 = 44.0;
const PADDING: f32 = 24.0;

pub(super) fn carrying_text(state: &FilesState) -> Option<String> {
    let carrying = state.carrying()?;
    let labels = state.labels();
    let text = match carrying.kind {
        CarryKind::Move => &labels.carrying_move,
        CarryKind::Copy => &labels.carrying_copy,
        _ => &labels.carrying_restore,
    };
    Some(FileLabels::fill(text, &state.named(&carrying.entry)))
}

pub(super) fn question_text(state: &FilesState) -> Option<String> {
    match state.ask() {
        Ask::Input(input) if input.discard_asked => Some(state.labels().discard_confirm.clone()),
        Ask::Input(input) => Some(input.label.clone()),
        Ask::Confirm(_) => state.confirm_words().map(|w| w.0),
        Ask::NameTaken(question) => Some(FileLabels::fill(
            &state.labels().name_taken_question,
            &question.message,
        )),
        _ => None,
    }
}

/// How high the band is; 0 when there is nothing to ask.
pub(in crate::file_manager) fn band_height(state: &FilesState, width: f32) -> f32 {
    let mut height = 0.0;
    if let Some(text) = carrying_text(state) {
        height += text_height(text.chars().count(), width) + BUTTONS * button_lines(width, 3);
    }
    if let Some(text) = question_text(state) {
        height += text_height(text.chars().count(), width) + BUTTONS * button_lines(width, 2);
    }
    if let Ask::Input(input) = state.ask() {
        height += FIELD;
        if let Some(error) = &input.error {
            height += text_height(error.chars().count(), width);
        }
    }
    if height > 0.0 {
        height + PADDING + wrapped_extra(state, width)
    } else {
        0.0
    }
}

/// The height the buttons take beyond one line, where their words wrap (the buttons stand one under the
/// other or side by side; the worst is every one under the other).
fn wrapped_extra(state: &FilesState, width: f32) -> f32 {
    let labels = state.labels();
    let mut words: Vec<String> = Vec::new();
    if let Some(carrying) = state.carrying() {
        words.push(labels.cancel.clone());
        words.push(
            match carrying.kind {
                CarryKind::Move => &labels.move_here,
                CarryKind::Copy => &labels.copy_here,
                _ => &labels.restore_here,
            }
            .clone(),
        );
    }
    match state.ask() {
        Ask::Input(input) if input.discard_asked => {
            words.extend([labels.keep_typing.clone(), labels.discard.clone()]);
        }
        Ask::Input(_) => words.extend([labels.cancel.clone(), labels.apply.clone()]),
        Ask::Confirm(_) => {
            words.push(labels.keep.clone());
            words.extend(state.confirm_words().map(|(_, yes)| yes));
        }
        Ask::NameTaken(_) => words.extend([labels.back.clone(), labels.keep_both.clone()]),
        _ => {}
    }
    words
        .iter()
        .map(|w| (button_height(w, width) - BUTTONS).max(0.0))
        .sum()
}

/// The most a button of the band may take at `width`: the band's inset on the left (18 px) and a small
/// margin on the right are left free. The words of a band button wrap inside this (see `wrapping_button`),
/// so a long answer ("Yes, write this marked file") gets a second line instead of leaving the window; the
/// button itself, not only its frame, is held to it, and `band_height` counts the extra lines.
pub(super) fn room(width: f32) -> f32 {
    width - 20.0
}

/// What a button of `width` px room takes in height. An estimate by characters (8 px each, the widest letters), not
/// by words: the real text breaks between words and may need a line more than this says. No reserve is added on
/// purpose: the position of the answer buttons relative to the first press is part of what the double-press tests
/// check (`cancel_with_something_typed_is_two_acts...`), and a reserve moved them. The band is a column that grows
/// with its content, so a line more than estimated is not cut; `files_band_buttons.rs` measures the real bottom edge
/// of every button at 160, 200 and 300 px against the window.
#[allow(clippy::cast_precision_loss)]
pub(super) fn button_height(label: &str, width: f32) -> f32 {
    let per_line = ((room(width) - 24.0) / 8.0).floor().max(1.0);
    (label.chars().count() as f32 / per_line).ceil().max(1.0) * 18.0 + 8.0 + 4.0
}

/// Lines `count` buttons take at `width`, at the worst.
pub(super) fn button_lines(width: f32, count: u8) -> f32 {
    if width < 300.0 {
        f32::from(count)
    } else if width < 560.0 {
        2.0
    } else {
        1.0
    }
}
