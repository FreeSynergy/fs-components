//! The board screen (A): tabs on top, the list of projects at the left, tinted columns of cards, and a side panel that
//! exists only while a card is chosen.
//!
//! Pattern: Adapter + Template method — the host's data in, one element out; the sizes come from the pure function
//! [`board_layout`], so they are tested without a window. Every change is a request to the host (`Events`).

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::scroll;

use super::entry::Entry;
use super::events::{Events, tell};
use super::model::{Column, Panel, Tab};
use super::side::{SideItem, SideList};
use super::tabs::tabs_element;

/// Width of the list of projects.
const SIDE: f32 = 220.0;
/// Width of the panel of a chosen card.
const PANEL: f32 = 360.0;
/// The width of a column in pixels (the board shows 260 to 280 and scrolls sideways).
const COLUMN: f32 = 272.0;
/// Below this width the list of projects goes away.
const NO_SIDE_BELOW: f32 = 900.0;

/// Widths of the three parts: the project list, one column, the panel (0 when there is none).
#[must_use]
pub fn board_layout(width: f32, panel_open: bool) -> (f32, f32, f32) {
    let side = if width < NO_SIDE_BELOW { 0.0 } else { SIDE };
    let panel = if panel_open {
        PANEL.min(width * 0.45)
    } else {
        0.0
    };
    // The columns share what is left in whole columns; they stay readable and the board scrolls instead.
    (side, COLUMN, panel)
}

/// The data of the board screen and the words it needs.
pub struct BoardView<'a> {
    /// The view tabs, and the key of the one shown.
    pub tabs: &'a [Tab],
    /// Key of the tab shown.
    pub active: &'a str,
    /// The hint of a locked tab.
    pub soon: &'a str,
    /// The projects at the left (with a heading).
    pub projects: &'a [SideItem],
    /// The columns.
    pub columns: &'a [Column],
    /// The panel of the chosen card.
    pub panel: Option<&'a Panel>,
    /// The column that is being added to, if any; its end holds an entry line.
    pub creating: Option<&'a str>,
    /// The entry line used for it.
    pub entry: &'a Entry,
    /// The text of the create button, of the entry's hint is the entry's own.
    pub create: &'a str,
    /// The words of the panel's close button.
    pub close: &'a str,
    /// The callbacks.
    pub events: &'a Events,
}

fn column(view: &BoardView, col: &Column, id: &str, width: f32) -> Div {
    let (name, count) = col.head();
    let mut cards = div().class("fs-plan-cards").flex_col().w_full();
    for card in &col.cards {
        cards = cards.child(card.build(id, view.events));
    }
    let tail = if view.creating == Some(col.key.as_str()) {
        view.entry
            .build(&format!("{id}-{}-entry", col.key), &col.key, view.events)
    } else {
        let (create, key) = (view.events.create.clone(), col.key.clone());
        div()
            .class("fs-plan-create")
            .id(format!("{id}-{}-create", col.key))
            .child(text(format!("+ {}", view.create)).class("fs-plan-create-text"))
            .on_click(move |_| tell(create.as_ref(), &key))
    };
    div()
        .class("fs-plan-column")
        .flex_col()
        .w(width)
        .flex_shrink_0()
        .child(
            div()
                .class("fs-plan-column-head")
                .flex_row()
                .child(text(name).class("fs-plan-column-name"))
                .child(text(count).class("fs-plan-column-count")),
        )
        .child(cards)
        .child(tail)
}

fn panel(view: &BoardView, panel: &Panel, width: f32, id: &str) -> Div {
    let mut body = div()
        .class("fs-plan-panel")
        .flex_col()
        .w(width)
        .flex_shrink_0()
        .id(format!("{id}-panel"));
    let close = view.events.close.clone();
    body = body.child(
        div()
            .class("fs-plan-panel-close")
            .id(format!("{id}-close"))
            .child(text(view.close.to_string()).class("fs-plan-tab-text"))
            .on_click(move |_| {
                if let Some(close) = &close {
                    close();
                    super::events::redraw();
                }
            }),
    );
    body = body.child(text(panel.title.clone()).class("fs-plan-panel-title"));
    if !panel.moves.is_empty() {
        let mut row = div().class("fs-plan-moves").flex_row().flex_wrap();
        for m in &panel.moves {
            let mut one = div()
                .class("fs-plan-move")
                .id(format!("{id}-move-{}", m.key));
            if m.key == panel.here {
                one = one.class("fs-plan-move-here");
            }
            let (act, card, to) = (view.events.act.clone(), panel.key.clone(), m.key.clone());
            one = one.on_click(move |_| {
                if let Some(act) = &act {
                    act(&card, &to);
                    super::events::redraw();
                }
            });
            row = row.child(one.child(text(m.title.clone()).class("fs-plan-tab-text")));
        }
        body = body
            .child(text("Move to").class("fs-plan-detail-label"))
            .child(row);
    }
    if let Some(note) = &panel.note {
        body = body.child(div().class("fs-plan-panel-note").child(text(note.clone())));
    }
    for d in &panel.details {
        body = body.child(
            div()
                .class("fs-plan-detail")
                .flex_row()
                .child(text(d.label.clone()).class("fs-plan-detail-label"))
                .child(text(d.value.clone()).class("fs-plan-detail-value")),
        );
    }
    body
}

impl BoardView<'_> {
    /// The element, filling `width` x `height`; `id` prefixes every element id.
    pub fn build(&self, id: &str, width: f32, height: f32) -> Div {
        let (side, col_w, panel_w) = board_layout(width, self.panel.is_some());
        let main_w = (width - side - panel_w).max(0.0);
        let mut columns = div().class("fs-plan-columns").flex_row();
        for col in self.columns {
            columns = columns.child(column(self, col, id, col_w));
        }
        #[allow(clippy::cast_precision_loss)] // a few columns
        let inner = (col_w + 16.0) * self.columns.len().max(1) as f32 + 16.0;
        let board = div()
            .flex_col()
            .w(main_w)
            .h(height)
            .min_h(0.0)
            .class("fs-plan-main")
            .child(tabs_element(
                self.tabs,
                self.active,
                self.soon,
                &format!("{id}-tab"),
                self.events,
            ))
            .child(
                div().flex_1().min_h(0.0).w(main_w).child(
                    scroll().both_directions().w_full().h_full().content(
                        div()
                            .w(inner.max(main_w))
                            .min_w(inner.max(main_w))
                            .flex_shrink_0()
                            .child(columns),
                    ),
                ),
            );
        let mut root = div().class("fs-plan-root").flex_row().w(width).h(height);
        if side > 0.0 {
            root = root.child(
                SideList {
                    items: self.projects,
                }
                .build(&format!("{id}-project"), side, self.events),
            );
        }
        root = root.child(board);
        if let Some(p) = self.panel {
            root = root.child(panel(self, p, panel_w, id));
        }
        root
    }
}

/// A screen that says one thing (reading, or why nothing can be shown), filling `width` x `height`.
pub fn message(text_: &str, width: f32, height: f32) -> Div {
    div().class("fs-plan-root").w(width).h(height).child(
        div()
            .class("fs-plan-message")
            .child(text(text_.to_string())),
    )
}
