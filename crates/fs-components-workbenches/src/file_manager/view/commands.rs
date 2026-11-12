//! The command bar: what can be done here, as icons with their words as tooltips.
//!
//! Pattern: Adapter (state to elements) — one list of items says what the bar holds, the same list gives its
//! height and its elements. What is not built yet is shown dimmed and says so in its tooltip (ADR 0066); what the
//! source cannot do is not shown.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use fs_components_widgets::icon::{Tone, icon};

use super::actions::{
    FileActions, ICON_BUTTON, button, icon_button, icon_button_marked, text_menu_button,
};
use super::command_items::{Item, items};
use crate::file_manager::state::FilesState;
use fs_components_widgets::{badge_count, badge_dot};

/// Padding of the bar.
pub(super) const PADDING: f32 = 24.0;

/// How high the command bar is in `width`: its items wrap.
pub(in crate::file_manager) fn commands_height(state: &FilesState, width: f32) -> f32 {
    let room = width - PADDING - 12.0;
    let (mut lines, mut used) = (1.0_f32, 0.0_f32);
    for item in items(state, &FileActions::silent(), width) {
        let wide = item.width();
        if used > 0.0 && used + wide > room {
            lines += 1.0;
            used = 0.0;
        }
        used += wide;
    }
    lines * ICON_BUTTON + 8.0
}

/// The command bar.
pub(in crate::file_manager) fn command_row(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    width: f32,
) -> Div {
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div()
        .class("fs-files-command-row")
        .flex_row()
        .flex_wrap()
        .items_start()
        .w(width - PADDING);
    for item in items(state, actions, width) {
        bar = bar.child(match item {
            Item::Icon(what, glyph, tip, flags, act) => {
                icon_button(id(what), glyph, &tip, flags, &act)
            }
            Item::Marked(what, glyph, tip, act, (count, said)) => {
                let mark = if count == 0 {
                    Some(badge_dot(&said))
                } else {
                    badge_count(count, &said)
                };
                icon_button_marked(id(what), glyph, &tip, (false, true), &act, mark)
            }
            Item::New(word, open, act) => text_menu_button(id("new"), &word, open, &act),
            Item::Word(what, glyph, word, act) => div()
                .id(id(what))
                .flex_shrink_0()
                .flex_row()
                .items_center()
                .gap(2.0)
                .child(icon(glyph, 18.0, Tone::Kind))
                .child(button(
                    id(&format!("{what}-button")),
                    &word,
                    ButtonVariant::Ghost,
                    &act,
                )),
            Item::Gap => div().flex_grow().h(1.0),
        });
    }
    bar
}
