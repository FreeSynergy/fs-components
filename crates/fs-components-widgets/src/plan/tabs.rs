//! The view tabs above the board: Board, List, Calendar, Timeline. A tab not built yet is drawn weak and does nothing.
//!
//! Pattern: Adapter — a row of tabs in, an element out; a locked tab has no handler, and its title says why.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::events::{Events, tell};
use super::model::Tab;

/// The tabs as one row; `active` is the key of the tab shown. `soon` is the hint of a locked tab.
pub fn tabs_element(tabs: &[Tab], active: &str, soon: &str, id: &str, events: &Events) -> Div {
    let mut row = div().class("fs-plan-tabs").flex_row().w_full();
    for tab in tabs {
        let mut one = div().class("fs-plan-tab").id(format!("{id}-{}", tab.key));
        if tab.key == active {
            one = one.class("fs-plan-tab-active");
        }
        if tab.locked {
            one = one
                .class("fs-plan-tab-locked")
                .attr("title", soon)
                .attr("data-locked", "true");
        } else {
            let (pick, key) = (events.tab.clone(), tab.key.clone());
            one = one.on_click(move |_| tell(&pick, &key));
        }
        row = row.child(one.child(text(tab.title.clone()).class("fs-plan-tab-text")));
    }
    row
}
