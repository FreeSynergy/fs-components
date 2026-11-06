//! The tasks as lanes of the `sortable-list` widget.
//!
//! Pattern: Adapter — rows and columns in, lanes out; the widget stays Thread 1's and knows nothing of
//! tasks. A column is a lane, a task is an item, and a card is what `task-card` draws for it. A row in
//! a column the source does not list still shows, in a lane of its own that takes nothing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::div;
use fs_components_widgets::Element;
use fs_components_widgets::sortable_list::{Bindings, Item, Labels, Lane, SortableList};
use fs_components_widgets::task_card::TaskCard;

use super::model::TaskRow;
use super::state::TasksState;
use super::view::ActOn;

/// What the cards draw from. Its own lock, held only to copy out, so drawing a card needs no other.
#[derive(Clone, Default)]
pub(super) struct Cards(Arc<Mutex<CardData>>);

#[derive(Default)]
struct CardData {
    rows: HashMap<String, TaskRow>,
    open: Option<String>,
}

impl Cards {
    fn card(&self, key: &str) -> TaskCard {
        let data = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(row) = data.rows.get(key) else {
            return TaskCard::new(key);
        };
        let mut card = TaskCard::new(row.title.clone());
        card.lines.clone_from(&row.lines);
        card.tags.clone_from(&row.tags);
        card.marks.clone_from(&row.marks);
        card.open = data.open.as_deref() == Some(key);
        card
    }

    fn set(&self, state: &TasksState) {
        let mut data = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        data.rows = state
            .rows()
            .iter()
            .map(|r| (r.key.clone(), r.clone()))
            .collect();
        data.open = state.selected().map(str::to_string);
    }
}

/// The lanes of `state`: one per column of the source, each saying how many it holds.
pub(super) fn lanes_of(state: &TasksState) -> Vec<Lane> {
    let movable = state.can_move();
    let item = |row: &TaskRow| {
        let item = Item::new(row.key.clone(), row.title.clone());
        if movable && row.move_refusal.is_none() {
            item
        } else {
            item.locked()
        }
    };
    let titled = |title: &str, n: usize| {
        if n == 0 {
            title.to_string()
        } else {
            format!("{title} ({n})")
        }
    };
    let mut lanes: Vec<Lane> = state
        .columns()
        .iter()
        .map(|column| {
            let items: Vec<Item> = state
                .rows()
                .iter()
                .filter(|r| r.column == column.key)
                .map(item)
                .collect();
            Lane::new(
                column.key.clone(),
                titled(&column.title, items.len()),
                items,
            )
        })
        .collect();
    let known = |key: &str| state.columns().iter().any(|c| c.key == key);
    for row in state.rows().iter().filter(|r| !known(&r.column)) {
        if let Some(lane) = lanes.iter_mut().find(|l| l.key == row.column) {
            lane.items.push(item(row).locked());
        } else {
            let items = vec![item(row).locked()];
            lanes.push(Lane::new(row.column.clone(), row.column.clone(), items).closed());
        }
    }
    lanes
}

/// The handle of the status view: the sortable list, and what its cards draw from.
#[derive(Clone)]
pub(super) struct Board {
    pub list: SortableList,
    cards: Cards,
}

impl Board {
    /// The status view over `state`. A click on a card reaches `open`; the keyboard's cursor opens
    /// nothing by itself, so walking over a marked card shows nothing of it (ADR 0029).
    pub fn new(id: String, state: &TasksState, open: ActOn) -> Self {
        let labels = state.labels();
        let mut words = Labels::default();
        words.empty.clone_from(&labels.column_empty);
        words.earlier.clone_from(&labels.earlier);
        words.later.clone_from(&labels.later);
        let cards = Cards::default();
        cards.set(state);
        let drawn = cards.clone();
        let content = Arc::new(move |item: &Item| -> Box<dyn Element> {
            let (open, key) = (Arc::clone(&open), item.key.clone());
            // The row's own click only moves the cursor on this same card, so the two cannot disagree.
            Box::new(
                div()
                    .w_full()
                    .child(drawn.card(&item.key).build())
                    .on_click(move |_| open(&key)),
            )
        });
        let list = SortableList::new(lanes_of(state))
            .id(id)
            .labels(words)
            .keys(Bindings::default())
            .content(content);
        Self { list, cards }
    }

    /// Take over what `state` holds now. While a card is held the lanes wait for it to land.
    pub fn sync(&self, state: &TasksState) {
        self.cards.set(state);
        if self.list.grabbed().is_some() {
            return;
        }
        self.list.set_lanes(lanes_of(state));
    }
}
