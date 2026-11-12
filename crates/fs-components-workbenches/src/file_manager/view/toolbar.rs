//! What a bar of buttons of a side area needs: the shape of a button and how high a line of them is.
//!
//! Pattern: none — plain data and one pure function; the command bar of the window is in `commands.rs`.

use blinc_cn::components::button::ButtonVariant;

use super::actions::Act;

/// Height of a line of buttons.
const LINE: f32 = 40.0;
/// Padding of a bar.
const PADDING: f32 = 24.0;
/// About how wide a character of a button is, and what a button has besides its words.
const CHAR_WIDTH: f32 = 8.0;
const BUTTON_ROOM: f32 = 40.0;

/// One button of a bar: its name, its words, how it looks, what it does.
pub(super) type Press = (&'static str, String, ButtonVariant, Act);

/// How high a bar with buttons of these words is in `width`.
#[allow(clippy::cast_precision_loss)]
pub(in crate::file_manager) fn buttons_height(words: &[&str], first: f32, width: f32) -> f32 {
    let room = width - PADDING;
    let (mut lines, mut used) = (1.0_f32, first);
    for word in words {
        let wide = word.chars().count() as f32 * CHAR_WIDTH + BUTTON_ROOM;
        if used > 0.0 && used + wide > room {
            lines += 1.0;
            used = 0.0;
        }
        used += wide + 8.0;
    }
    lines * LINE + PADDING
}
