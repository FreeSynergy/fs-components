//! The card of the board: a title in normal size, a row of chips, one line of facts (key, due chip, initials).
//!
//! Pattern: Adapter — the data of one card in, an element out; a click on the card is the host's `pick`.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::chip::{Chip, chip_element};
use super::events::{Events, tell};
use crate::avatar_group_initials;

/// One card.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlanCard {
    /// Names the card for the host.
    pub key: String,
    /// The title.
    pub title: String,
    /// A short name of the task beside its due chip (a number, a code); may be empty.
    pub code: String,
    /// The due date as a chip.
    pub due: Option<Chip>,
    /// Who holds it; the card shows initials, at most three.
    pub people: Vec<String>,
    /// Labels and marks, as chips under the title.
    pub chips: Vec<Chip>,
    /// A small sentence (why it waits), in weak text.
    pub note: Option<String>,
    /// Drawn pale: it cannot be started.
    pub muted: bool,
    /// Drawn as the chosen card.
    pub selected: bool,
}

impl PlanCard {
    /// A card with a title.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            ..Self::default()
        }
    }

    /// The element; `id` prefixes the element id of the card.
    pub fn build(&self, id: &str, events: &Events) -> Div {
        let mut card = div()
            .class("fs-plan-card")
            .flex_col()
            .w_full()
            .id(format!("{id}-{}", self.key));
        if self.selected {
            card = card.class("fs-plan-card-selected");
        }
        if self.muted {
            card = card.class("fs-plan-muted");
        }
        card = card.child(text(self.title.clone()).class("fs-plan-card-title"));
        if let Some(note) = &self.note {
            card = card.child(text(note.clone()).class("fs-plan-note"));
        }
        if !self.chips.is_empty() {
            let mut row = div().class("fs-plan-chips").flex_row().flex_wrap();
            for chip in &self.chips {
                row = row.child(chip_element(chip));
            }
            card = card.child(row);
        }
        let mut facts = div().class("fs-plan-facts").flex_row().w_full();
        if !self.code.is_empty() {
            facts = facts.child(text(self.code.clone()).class("fs-plan-code"));
        }
        if let Some(due) = &self.due {
            facts = facts.child(chip_element(due));
        }
        facts = facts.child(div().flex_1());
        if !self.people.is_empty() {
            for name in self.people.iter().take(3) {
                facts = facts.child(initials_circle(name));
            }
        }
        let (pick, key) = (events.pick.clone(), self.key.clone());
        card.child(facts).on_click(move |_| tell(&pick, &key))
    }
}

/// The circle with the initials of `name`, drawn small beside a due chip.
pub fn initials_circle(name: &str) -> Div {
    div()
        .class("fs-plan-initials")
        .attr("title", name)
        .child(text(avatar_group_initials(name)).class("fs-plan-initials-text"))
}
