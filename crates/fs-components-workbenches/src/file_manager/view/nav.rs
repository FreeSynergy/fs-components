//! The bar of the way and the bar of commands: icons with their words as tooltips.
//!
//! Pattern: Adapter (state to elements) — what the source or the variant cannot do is not offered; the buttons of
//! `toolbar.rs` are the commands, this file lays them out (and wraps them) and draws the way (back, forward, up,
//! reload, the breadcrumb, the search).

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use fs_components_widgets::icon::Glyph;

use super::actions::{FileActions, ICON_BUTTON, button, icon_button};
use super::toolbar::{
    BUTTON_ROOM, CHAR_WIDTH, PADDING, Press, TOOLS_BELOW, main_presses, shows_tools, tool_presses,
};
use crate::file_manager::context::Variant;
use crate::file_manager::state::FilesState;

/// The glyph of a button that is only an icon; the others (the order of the list) stay small text buttons.
fn glyph_of_press(name: &str) -> Option<Glyph> {
    Some(match name {
        "filter" => Glyph::Filter,
        "view-list" => Glyph::ViewList,
        "view-grid" => Glyph::ViewGrid,
        "trash" => Glyph::Trash,
        "leave-trash" => Glyph::Back,
        "tools" => Glyph::Details,
        "new-folder" => Glyph::Plus,
        "upload" => Glyph::Upload,
        "rescan" => Glyph::Drive,
        "transfers" => Glyph::Transfers,
        "direction" => Glyph::Sort,
        _ => return None,
    })
}

/// How wide a button of the command row is.
fn press_width(press: &Press) -> f32 {
    if glyph_of_press(press.0).is_some() {
        ICON_BUTTON + 4.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        let chars = press.1.chars().count() as f32;
        chars * CHAR_WIDTH + BUTTON_ROOM + 8.0
    }
}

fn command_presses(state: &FilesState, actions: &FileActions, width: f32) -> Vec<Press> {
    let narrow = width < TOOLS_BELOW && state.variant() == Variant::Full;
    let main = main_presses(state, actions, narrow);
    let tools = if shows_tools(state, width) {
        tool_presses(state, actions)
    } else {
        Vec::new()
    };
    // What can be done first, then the order of the list, then how the list is shown.
    let is_order = |p: &Press| p.0.starts_with("sort-") || p.0 == "direction";
    let (order, acts): (Vec<Press>, Vec<Press>) = tools.into_iter().partition(is_order);
    let (tools_button, views): (Vec<Press>, Vec<Press>) =
        main.into_iter().partition(|p| p.0 == "tools");
    let mut presses = tools_button;
    presses.extend(acts);
    presses.extend(order);
    presses.extend(views);
    presses
}

/// How high the row of commands is in `width`: its buttons wrap.
#[allow(clippy::cast_precision_loss)]
pub(in crate::file_manager) fn commands_height(state: &FilesState, width: f32) -> f32 {
    let room = width - PADDING;
    let (mut lines, mut used) = (1.0_f32, 0.0_f32);
    for press in command_presses(state, &FileActions::silent(), width) {
        let wide = press_width(&press);
        if used > 0.0 && used + wide > room {
            lines += 1.0;
            used = 0.0;
        }
        used += wide;
    }
    lines * ICON_BUTTON + 8.0
}

/// How high the bar of the way (back, forward, up, reload, where you are, search) is.
pub(in crate::file_manager) const NAV_HEIGHT: f32 = 60.0;
/// The room of the search field at its right end.
const SEARCH_WIDTH: f32 = 240.0;

/// The bar of the way: back, forward, up, reload; the breadcrumb; the search at the right.
pub(in crate::file_manager) fn nav_row(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    field: impl ElementBuilder + 'static,
    width: f32,
) -> Div {
    let id = |what: &str| format!("{name}-{what}");
    let labels = state.labels();
    let mut bar = div()
        .class("fs-files-bar-row")
        .flex_row()
        .items_center()
        .w(width - PADDING)
        .h(NAV_HEIGHT - 8.0);
    let mut used = 0.0;
    if state.variant() == Variant::Full {
        for (what, glyph, tip, enabled, act) in [
            (
                "go-back",
                Glyph::Back,
                &labels.nav_back,
                state.can_go_back(),
                actions.plain(FilesState::go_back),
            ),
            (
                "go-forward",
                Glyph::Forward,
                &labels.nav_forward,
                state.can_go_forward(),
                actions.plain(FilesState::go_forward),
            ),
            (
                "go-up",
                Glyph::Up,
                &labels.nav_up,
                state.can_go_up(),
                actions.plain(FilesState::go_up),
            ),
            (
                "reload",
                Glyph::Reload,
                &labels.nav_reload,
                true,
                actions.plain(FilesState::retry),
            ),
        ] {
            bar = bar.child(icon_button(id(what), glyph, tip, (false, enabled), &act));
            used += ICON_BUTTON;
        }
    }
    let search_wide = if state.capabilities().search {
        SEARCH_WIDTH.min((width - PADDING - used) * 0.4)
    } else {
        0.0
    };
    let room = (width - PADDING - used - search_wide - 16.0).max(40.0);
    bar = bar.child(
        div()
            .flex_row()
            .items_center()
            .w(room)
            .overflow_clip()
            .child(super::places::crumbs(state, name, actions, room)),
    );
    if state.capabilities().search {
        bar = bar.child(
            div()
                .id(id("search"))
                .class("fs-files-search-field")
                .w(search_wide)
                .flex_shrink_0()
                .child(field),
        );
    }
    bar
}

/// The commands: what can be done here, as icons with their words as tooltips.
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
        .items_center()
        .w(width - PADDING);
    for (what, word, variant, act) in command_presses(state, actions, width) {
        bar = bar.child(match glyph_of_press(what) {
            Some(glyph) => icon_button(
                id(what),
                glyph,
                &word,
                (variant == ButtonVariant::Primary, true),
                &act,
            ),
            None => button(id(what), &word, variant, &act),
        });
    }
    bar
}
