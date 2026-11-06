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
use crate::styles::{MENU_GAP, MENU_LINE_HEIGHT, MENU_TEXT, MENU_WIDTH};

/// What a press on a menu in the bar does: which menu, and where its panel would start.
pub(crate) type OnTrigger = Arc<dyn Fn(usize, f32, f32) + Send + Sync>;
/// What a press on a line does: which menu it is in, and the id of the entry.
pub(crate) type OnChoose = Arc<dyn Fn(usize, &str) + Send + Sync>;

/// The element id of the row of menus.
pub(crate) const BAR_ID: &str = "fs-app-menus";
/// The element id of the open panel.
pub(crate) const PANEL_ID: &str = "fs-app-menu-panel";

/// The size of a panel, for keeping it inside the window. Long labels wrap, so the height is a floor.
pub(crate) fn panel_size(menu: &Menu) -> (f32, f32) {
    #[allow(clippy::cast_precision_loss)] // a menu has far fewer lines than 2^23
    let lines = menu.entries.len() as f32;
    (MENU_WIDTH, lines * MENU_LINE_HEIGHT)
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

/// The panel of an open menu. Builds plain elements only: it runs under the lock of Blinc's overlay
/// stack.
pub(crate) fn panel(index: usize, menu: &Menu, on_choose: &OnChoose) -> Div {
    let radius = ThemeState::get().radius(RadiusToken::Md);
    let mut panel = div()
        .id(PANEL_ID)
        .class("cn-menu")
        .class("cn-menubar-content")
        .attr("role", "menu")
        .attr("tabindex", "0")
        .flex_col()
        .rounded(radius)
        .overflow_clip()
        .h_fit()
        .w(MENU_WIDTH);
    for entry in &menu.entries {
        panel = panel.child(line(index, entry, on_choose));
    }
    panel
}

/// A line takes a press even when disabled: a line without a handler could keep the handler of the
/// line it replaces (`DESIGN.md`), and the shell refuses a disabled entry anyway.
fn line(index: usize, entry: &MenuEntry, on_choose: &OnChoose) -> Div {
    let (on_choose, id) = (Arc::clone(on_choose), entry.id.clone());
    let mut line = div()
        .id(format!("fs-app-menu-entry-{}", entry.id))
        .class("cn-menu-item")
        .class("cn-menubar-item")
        .attr("role", "menuitem")
        .attr("tabindex", "-1")
        .attr_when("disabled", "", entry.disabled)
        .w_full()
        .h_fit()
        // The same as `.cn-menubar-item`: class rules can drop on a rebuilt overlay (a note in `blinc_cn`).
        .py(2.0)
        .px(3.0)
        .child(label(&entry.label))
        .on_click(move |_| on_choose(index, &id));
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
