//! The menus as elements: the row in the title bar and the panel of an open menu.
//!
//! Pattern: Adapter (state to elements). Blinc's own keyboard navigation walks these elements by
//! their roles: Tab reaches the bar, arrows move between menus and lines, Enter chooses, Escape
//! closes. Colours and paddings are `blinc_cn`'s menu classes. Only the outermost element of a menu or
//! a line takes a press, so no click target sits inside another.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_theme::{RadiusToken, ThemeState};

use crate::menu::{Menu, MenuEntry};
use crate::styles::{MARK_WIDTH, MENU_GAP, MENU_LINE_HEIGHT, MENU_LINE_PITCH, MENU_TEXT, MENU_WIDTH};

/// What a press on a menu in the bar does: which menu, and where its panel would start.
pub(crate) type OnTrigger = Arc<dyn Fn(usize, f32, f32) + Send + Sync>;
/// What a press on a line does: which menu it is in, and the id of the entry.
pub(crate) type OnChoose = Arc<dyn Fn(usize, &str) + Send + Sync>;
/// What a press on a line with a next level does: which menu it is in, and the position of the line in it.
pub(crate) type OnSub = Arc<dyn Fn(usize, usize) + Send + Sync>;

/// What the lines of an open menu call.
#[derive(Clone)]
pub(crate) struct Callbacks {
    pub(crate) choose: OnChoose,
    pub(crate) sub: OnSub,
}

/// The element id of the row of menus.
pub(crate) const BAR_ID: &str = "fs-app-menus";
/// The element id of the open panel.
pub(crate) const PANEL_ID: &str = "fs-app-menu-panel";
/// The element id of the panel of the next level.
pub(crate) const SUB_PANEL_ID: &str = "fs-app-menu-sub-panel";

/// The size of what the overlay holds, for keeping it inside the window: the panel, and beside it the panel of the next level
/// when one is open (`open_sub` = the position of its line). Long labels wrap, so the height is a floor.
pub(crate) fn panel_size(menu: &Menu, open_sub: Option<usize>) -> (f32, f32) {
    #[allow(clippy::cast_precision_loss)] // a menu has far fewer lines than 2^23
    let lines = |n: usize| n as f32 * MENU_LINE_HEIGHT;
    let main = (MENU_WIDTH, lines(menu.entries.len()));
    match open_sub.and_then(|p| menu.entries.get(p).map(|e| (p, e))) {
        Some((position, entry)) => (
            main.0 + MENU_WIDTH,
            main.1.max(lines(position) + lines(entry.children.len())),
        ),
        None => main,
    }
}

/// The row of menus; `None` if there is nothing to show, then the title bar stays as it was.
pub(crate) fn menu_bar(menus: &[Menu], on_trigger: &OnTrigger) -> Option<Div> {
    let mut row = div()
        .id(BAR_ID)
        .class("fs-app-menus")
        .attr("role", "menubar")
        .attr("tabindex", "0");
    let mut any = false;
    for (index, menu) in menus.iter().enumerate() {
        if !menu.entries.is_empty() {
            any = true;
            row = row.child(trigger(index, menu, on_trigger));
        }
    }
    any.then_some(row)
}

fn trigger(index: usize, menu: &Menu, on_trigger: &OnTrigger) -> Div {
    let on_trigger = Arc::clone(on_trigger);
    div()
        .id(format!("fs-app-menu-{index}"))
        .class("cn-menubar-trigger")
        .attr("role", "menuitem")
        .attr("tabindex", "-1")
        .attr("aria-haspopup", "menu")
        .cursor_pointer()
        .child(label(&menu.label).no_wrap())
        .on_click(move |ctx| {
            on_trigger(
                index,
                ctx.bounds_x,
                ctx.bounds_y + ctx.bounds_height + MENU_GAP,
            );
        })
}

/// The panel of an open menu, and beside it the panel of the next level when `open_sub` names a line with children. Builds plain
/// elements only: it runs under the lock of Blinc's overlay stack.
pub(crate) fn panel(index: usize, menu: &Menu, open_sub: Option<usize>, calls: &Callbacks) -> Div {
    let main = one_panel(PANEL_ID, index, &menu.entries, open_sub, calls);
    let sub = open_sub
        .and_then(|p| menu.entries.get(p).map(|e| (p, e)))
        .filter(|(_, e)| !e.children.is_empty());
    let Some((position, entry)) = sub else {
        return main;
    };
    #[allow(clippy::cast_precision_loss)]
    let down = position as f32 * MENU_LINE_PITCH / 4.0; // `mt` counts in units of 4 px
    let sub = one_panel(SUB_PANEL_ID, index, &entry.children, None, calls).mt(down);
    div().flex_row().items_start().child(main).child(sub)
}

fn one_panel(
    id: &str,
    index: usize,
    entries: &[MenuEntry],
    open_sub: Option<usize>,
    calls: &Callbacks,
) -> Div {
    let radius = ThemeState::get().radius(RadiusToken::Md);
    let marks = entries.iter().any(|e| e.checked);
    let mut panel = div()
        .id(id)
        .class("cn-menu")
        .class("cn-menubar-content")
        .attr("role", "menu")
        .attr("tabindex", "0")
        .flex_col()
        .rounded(radius)
        .overflow_clip()
        .h_fit()
        .w(MENU_WIDTH);
    for (position, entry) in entries.iter().enumerate() {
        panel = panel.child(line(index, position, entry, (marks, open_sub), calls));
    }
    panel
}

/// A line takes a press even when disabled: a line without a handler could keep the handler of the
/// line it replaces (`DESIGN.md`), and the shell refuses a disabled entry anyway. A line with a next level opens it
/// (`aria-expanded` says whether it is open); a line in a group with a chosen one leaves room for the mark.
fn line(
    index: usize,
    position: usize,
    entry: &MenuEntry,
    (marks, open_sub): (bool, Option<usize>),
    calls: &Callbacks,
) -> Div {
    let (choose, sub, id) = (Arc::clone(&calls.choose), Arc::clone(&calls.sub), entry.id.clone());
    let opens = !entry.children.is_empty();
    let mut row = div().flex_row().items_center().w_full().h_fit();
    if marks {
        let mark = if entry.checked { "\u{2713}" } else { "" };
        row = row.child(div().w(MARK_WIDTH).flex_shrink_0().child(label(mark)));
    }
    row = row.child(div().flex_grow().child(label(&entry.label)));
    if opens {
        row = row.child(label("\u{25b8}"));
    }
    let mut line = div()
        .id(format!("fs-app-menu-entry-{}", entry.id))
        .class("cn-menu-item")
        .class("cn-menubar-item")
        .attr("role", if entry.checked { "menuitemradio" } else { "menuitem" })
        .attr("tabindex", "-1")
        .attr_when("disabled", "", entry.disabled)
        .attr_when("aria-checked", "true", entry.checked)
        .attr_when("aria-haspopup", "menu", opens)
        .attr_when("aria-expanded", "true", opens && open_sub == Some(position))
        .w_full()
        .h_fit()
        // The same as `.cn-menubar-item`: class rules can drop on a rebuilt overlay (a note in `blinc_cn`).
        .py(2.0)
        .px(3.0)
        .child(row)
        .on_click(move |_| {
            if opens {
                sub(index, position);
            } else {
                choose(index, &id);
            }
        });
    if !entry.disabled {
        line = line.cursor_pointer();
    }
    line
}

fn label(words: &str) -> blinc_layout::text::Text {
    text(words)
        .size(MENU_TEXT)
        .no_cursor()
        .pointer_events_none()
}
