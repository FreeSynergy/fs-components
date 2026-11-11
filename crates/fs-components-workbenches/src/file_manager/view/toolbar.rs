//! Search, views, order, tools.
//!
//! Pattern: Adapter (state to elements) — what the source or the variant cannot do is not offered, and in the
//! trash other things are offered. At narrow widths the tools hide behind one button, so the rest stays
//! usable; their height is worked out from their words, since what wraps must still fit.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

use fs_components_widgets::icon::Glyph;

use super::actions::{Act, FileActions, ICON_BUTTON, button, icon_button};
use crate::file_manager::ask::Ask;
use crate::file_manager::context::{FileView, Variant};
use crate::file_manager::model::SortKey;
use crate::file_manager::state::FilesState;

/// Below this width the tools hide behind a button.
const TOOLS_BELOW: f32 = 560.0;
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

fn sort_word(state: &FilesState, key: SortKey) -> &str {
    let labels = state.labels();
    match key {
        SortKey::Name => &labels.sort_name,
        SortKey::Modified => &labels.sort_modified,
        SortKey::Size => &labels.sort_size,
        _ => &labels.sort_type,
    }
}

fn pressed(on: bool) -> ButtonVariant {
    if on {
        ButtonVariant::Primary
    } else {
        ButtonVariant::Secondary
    }
}

fn main_presses(state: &FilesState, actions: &FileActions, narrow: bool) -> Vec<Press> {
    let labels = state.labels();
    let mut out = Vec::new();
    if state.capabilities().search {
        let word = if state.filter().is_active() {
            labels.filter_on.clone()
        } else {
            labels.filter.clone()
        };
        out.push((
            "filter",
            word,
            pressed(state.filter().is_active()),
            actions.plain(FilesState::open_filter),
        ));
    }
    out.push((
        "view-list",
        labels.view_list.clone(),
        pressed(state.view() == FileView::List),
        actions.with(|s| s.set_view(FileView::List)),
    ));
    out.push((
        "view-grid",
        labels.view_grid.clone(),
        pressed(state.view() == FileView::Grid),
        actions.with(|s| s.set_view(FileView::Grid)),
    ));
    if state.capabilities().trash {
        if state.in_trash() {
            out.push((
                "leave-trash",
                labels.leave_trash.clone(),
                ButtonVariant::Secondary,
                actions.with(|s| s.open_folder(None)),
            ));
        } else {
            out.push((
                "trash",
                labels.trash.clone(),
                ButtonVariant::Secondary,
                actions.plain(FilesState::open_trash),
            ));
        }
    }
    if narrow {
        out.push((
            "tools",
            labels.tools.clone(),
            pressed(state.tools_open()),
            actions.plain(FilesState::toggle_tools),
        ));
    }
    out
}

fn tool_presses(state: &FilesState, actions: &FileActions) -> Vec<Press> {
    let labels = state.labels();
    let caps = state.capabilities();
    let mut out: Vec<Press> = Vec::new();
    if caps.sort {
        for (name, key) in [
            ("sort-name", SortKey::Name),
            ("sort-modified", SortKey::Modified),
            ("sort-size", SortKey::Size),
            ("sort-type", SortKey::Type),
        ] {
            out.push((
                name,
                sort_word(state, key).to_string(),
                if state.sort().key == key {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                },
                actions.with(move |s| s.set_sort(key)),
            ));
        }
        if state.can_toggle_direction() {
            let word = if state.sort().descending {
                &labels.descending
            } else {
                &labels.ascending
            };
            out.push((
                "direction",
                word.clone(),
                ButtonVariant::Ghost,
                actions.plain(FilesState::toggle_direction),
            ));
        }
    }
    if state.in_trash() {
        if caps.trash && !state.typing() {
            out.push((
                "empty-trash",
                labels.empty_trash.clone(),
                ButtonVariant::Ghost,
                actions.plain(FilesState::ask_empty_trash),
            ));
        }
        return out;
    }
    let mut add = |on: bool, name, word: &String, change: fn(&mut FilesState)| {
        if on {
            out.push((
                name,
                word.clone(),
                ButtonVariant::Secondary,
                actions.plain(change),
            ));
        }
    };
    add(
        caps.change,
        "new-folder",
        &labels.new_folder,
        FilesState::begin_new_folder,
    );
    add(
        caps.upload,
        "upload",
        &labels.upload,
        FilesState::begin_upload,
    );
    add(caps.rescan, "rescan", &labels.rescan, FilesState::rescan);
    add(
        caps.upload || caps.download,
        "transfers",
        &labels.transfers,
        FilesState::open_transfers,
    );
    out
}


fn shows_tools(state: &FilesState, width: f32) -> bool {
    // At narrow widths a question takes the room of the tools: it must be seen.
    state.variant() == Variant::Full
        && (width >= TOOLS_BELOW || (state.tools_open() && *state.ask() == Ask::Nothing))
}

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
