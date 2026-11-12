//! The row of headings of the list, and what a person can do with it.
//!
//! Pattern: Adapter (data to elements) — a heading with a `sort` key is a thing to press (Shift adds a second level),
//! an arrow and the rank show the order. With a layout (`ListView::columns_layout`) the person can: right-click a
//! heading for the menu that hides and shows the columns (a column that must stay is shown ticked and locked, ADR
//! 0066), drag a heading to move the column, drag its right edge to make it wider or narrower, or do the same by
//! keyboard on a focused heading: Alt+Left and Alt+Right move, Shift+Left and Shift+Right change the width, Enter and
//! Space sort. A change is reported as the new layout; the caller keeps it and builds the list again.

use std::sync::Arc;

use blinc_cn::components::context_menu::context_menu;
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::layout::{ColumnLayout, LEAST_WIDTH};
use super::model::{ListView, OnLayout};
use crate::keys::{Key, bind_keys};

/// Height of the row of headings in pixels.
pub(super) const HEAD_HEIGHT: f32 = 28.0;
/// The room of the edge that is dragged to change a width, and the step of the keys.
const EDGE: f32 = 6.0;
const STEP: f32 = 12.0;

/// What the head needs to let the person change the layout.
pub(super) struct Layout {
    pub current: ColumnLayout,
    pub on_change: OnLayout,
    /// Key and title of every column, hidden ones too, in the order of the caller.
    pub titles: Vec<(String, String)>,
}

fn menu(parts: &Arc<Layout>, x: f32, y: f32) {
    let mut menu = context_menu().at(x, y);
    for key in parts.current.order() {
        let title = parts
            .titles
            .iter()
            .find(|(k, _)| k == key)
            .map_or(key.as_str(), |(_, t)| t.as_str());
        let tick = if parts.current.is_shown(key) {
            "\u{2713}  "
        } else {
            "     "
        };
        let label = format!("{tick}{title}");
        // A column that must stay is shown and ticked, but locked (the name; the last one that is shown).
        if parts.current.is_shown(key) && !parts.current.can_hide(key) {
            menu = menu.item_disabled(label);
        } else {
            let (parts, key) = (Arc::clone(parts), key.clone());
            menu = menu.item(label, move || {
                let mut next = parts.current.clone();
                if next.toggle(&key) {
                    (parts.on_change)(next);
                }
            });
        }
    }
    let parts = Arc::clone(parts);
    let _ = menu
        .separator()
        .item("Reset columns", move || {
            let mut next = parts.current.clone();
            next.reset();
            (parts.on_change)(next);
        })
        .show();
}

fn changed(parts: &Layout, change: impl FnOnce(&mut ColumnLayout)) {
    let mut next = parts.current.clone();
    change(&mut next);
    if next != parts.current {
        (parts.on_change)(next);
    }
}

pub(super) fn head(list: &ListView, widths: &[f32], parts: Option<&Arc<Layout>>) -> Div {
    let mut element = div().class("fs-list-head").flex_row().h(HEAD_HEIGHT);
    for (column, width) in list.columns.iter().zip(widths) {
        let arrow = match column.order {
            Some(true) => " \u{25BE}",
            Some(false) => " \u{25B4}",
            None => "",
        };
        let rank = column.rank.map_or(String::new(), |r| r.to_string());
        let mut cell = div().class("fs-list-cell").w(*width).relative().child(
            text(format!("{}{arrow}{rank}", column.title))
                .no_wrap()
                .no_cursor(),
        );
        let key = column.key();
        if let Some(name) = &list.name {
            cell = cell.id(format!("{name}-head-{key}"));
        }
        if let Some(sort) = column.sort.clone() {
            let (on_sort, on_head) = (list.on_head_sort.clone(), list.on_head.clone());
            if on_sort.is_some() || on_head.is_some() {
                let press = Arc::new(move |shift: bool| match (&on_sort, &on_head) {
                    (Some(f), _) => f(&sort, shift),
                    (None, Some(f)) => f(&sort),
                    (None, None) => {}
                });
                let click = Arc::clone(&press);
                cell = cell
                    .class("fs-list-cell-sortable")
                    .on_click(move |ctx| click(ctx.shift));
                if let Some(parts) = parts {
                    cell = keyed(cell, parts, &key, *width, Some(press));
                }
            }
        } else if let Some(parts) = parts {
            cell = keyed(cell, parts, &key, *width, None);
        }
        if let Some(parts) = parts {
            cell = with_layout_handles(cell, parts, &key, *width);
        }
        element = element.child(cell);
    }
    element
}

/// The keys of a heading: move, widen, sort.
fn keyed(
    cell: Div,
    parts: &Arc<Layout>,
    key: &str,
    width: f32,
    press: Option<Arc<dyn Fn(bool) + Send + Sync>>,
) -> Div {
    let (parts, key) = (Arc::clone(parts), key.to_string());
    let handler = Arc::new(move |event: crate::keys::KeyEvent| {
        match (event.key, event.alt, event.shift) {
            (Key::Left, true, _) => changed(&parts, |l| l.move_by(&key, -1)),
            (Key::Right, true, _) => changed(&parts, |l| l.move_by(&key, 1)),
            (Key::Left, false, true) => changed(&parts, |l| {
                l.set_width(&key, (width - STEP).max(LEAST_WIDTH));
            }),
            (Key::Right, false, true) => changed(&parts, |l| l.set_width(&key, width + STEP)),
            (Key::Enter | Key::Space | Key::Activate, false, _) => match &press {
                Some(press) => press(event.shift),
                None => return false,
            },
            _ => return false,
        }
        true
    });
    bind_keys(cell.attr("tabindex", "0"), handler)
}

/// The menu on a right-click, the drag of the heading (move) and the drag of its edge (width).
fn with_layout_handles(cell: Div, parts: &Arc<Layout>, key: &str, width: f32) -> Div {
    let menu_parts = Arc::clone(parts);
    let cell = cell.on_right_click(move |ctx| menu(&menu_parts, ctx.mouse_x, ctx.mouse_y));
    // Drag the heading: the column goes as many places as the heading was dragged, by its own width.
    let (move_parts, move_key) = (Arc::clone(parts), key.to_string());
    let cell = cell.on_drag_end(move |ctx| {
        let places = (ctx.drag_delta_x / width.max(LEAST_WIDTH)).round();
        // A whole number of places, a few at most: the cast cannot lose more than the fraction.
        #[allow(clippy::cast_possible_truncation)]
        let places = places.clamp(-64.0, 64.0) as isize;
        if places != 0 {
            shift_among_shown(&move_parts, &move_key, places);
        }
    });
    let (edge_parts, edge_key) = (Arc::clone(parts), key.to_string());
    cell.child(
        div()
            .class("fs-list-edge")
            .absolute()
            .right(0.0)
            .top(0.0)
            .w(EDGE)
            .h(HEAD_HEIGHT)
            .on_drag_end(move |ctx| {
                changed(&edge_parts, |l| {
                    l.set_width(&edge_key, (width + ctx.drag_delta_x).max(LEAST_WIDTH));
                });
            }),
    )
}

/// Move a column by `places` among the columns that are shown (hidden ones are stepped over).
fn shift_among_shown(parts: &Layout, key: &str, places: isize) {
    changed(parts, |l| {
        let shown: Vec<String> = l.shown().iter().map(|k| (*k).to_string()).collect();
        let Some(at) = shown.iter().position(|k| k == key) else {
            return;
        };
        let to = at.saturating_add_signed(places).min(shown.len() - 1);
        if let Some(target) = shown.get(to) {
            l.move_to(key, target);
        }
    });
}
