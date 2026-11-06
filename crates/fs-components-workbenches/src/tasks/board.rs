//! The tasks as lanes of the `sortable-list` widget.
//!
//! Pattern: Adapter — rows and columns in, lanes out; the widget stays Thread 1's and knows nothing of
//! tasks. A column is a lane, a task is an item, and a card is what `task-card` draws for it. A row in
//! a column the source does not list still shows, in a lane of its own that takes nothing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use fs_components_widgets::Element;
use fs_components_widgets::sortable_list::{
    Bindings, Content, Item, Labels, Lane, Outcome, SortableList,
};
use fs_components_widgets::task_card::TaskCard;

use super::context::TaskView;
use super::model::TaskRow;
use super::state::TasksState;

/// What the cards draw from. Its own lock, held only to copy out, so drawing a card needs no other.
#[derive(Clone, Default)]
pub(super) struct Cards(Arc<Mutex<CardData>>);

#[derive(Default)]
struct CardData {
    rows: HashMap<String, TaskRow>,
    open: Option<String>,
}

impl Cards {
    fn card(&self, item: &Item) -> TaskCard {
        let data = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(row) = data.rows.get(&item.key) else {
            return TaskCard::new(item.label.clone());
        };
        let mut card = TaskCard::new(row.title.clone());
        card.lines.clone_from(&row.lines);
        card.tags.clone_from(&row.tags);
        card.marks.clone_from(&row.marks);
        card.open = data.open.as_deref() == Some(item.key.as_str());
        card
    }

    fn set_rows(&self, state: &TasksState) {
        let mut data = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        data.rows = state
            .rows()
            .iter()
            .map(|r| (r.key.clone(), r.clone()))
            .collect();
    }

    fn set_open(&self, state: &TasksState) {
        let mut data = self.0.lock().unwrap_or_else(PoisonError::into_inner);
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
    asked: Arc<Mutex<Option<String>>>,
}

impl Board {
    /// The status view over `state`. A click on a card arrives as `Outcome::Chosen`; the keyboard's
    /// cursor opens nothing by itself, so walking over a marked card shows nothing of it (ADR 0029).
    pub fn new(id: String, state: &TasksState) -> Self {
        let labels = state.labels();
        let mut words = Labels::default();
        words.empty.clone_from(&labels.column_empty);
        words.earlier.clone_from(&labels.earlier);
        words.later.clone_from(&labels.later);
        let cards = Cards::default();
        cards.set_rows(state);
        cards.set_open(state);
        let drawn = cards.clone();
        let content: Content =
            Arc::new(move |item: &Item| -> Box<dyn Element> { Box::new(drawn.card(item).build()) });
        let list = SortableList::new(lanes_of(state))
            .id(id)
            .labels(words)
            .keys(Bindings::default())
            .content(content);
        Self {
            list,
            cards,
            asked: Arc::default(),
        }
    }

    /// Pick up this card, for a frontend's gesture: a refusal then talks about this card.
    pub fn grab(&self, key: &str) -> Outcome {
        *self.asked.lock().unwrap_or_else(PoisonError::into_inner) = Some(key.to_string());
        let outcome = self.list.grab(key);
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        outcome
    }

    /// The card a refused pick-up was about: the one asked for, else the one under the cursor.
    pub fn refused_card(&self) -> Option<String> {
        let asked = self.asked.lock().unwrap_or_else(PoisonError::into_inner);
        asked.clone().or_else(|| self.list.cursor_key())
    }

    /// Take over what `state` holds now. While a card is held the lanes and what is on the cards wait
    /// for it to land, and leaving the view gives the hold up.
    pub fn sync(&self, state: &TasksState) {
        self.cards.set_open(state);
        if self.list.grabbed().is_some() {
            if state.view() == TaskView::Status {
                return;
            }
            // Comes back here through the list's outcome, with nothing held.
            self.list.cancel();
        }
        self.cards.set_rows(state);
        self.list.set_lanes(lanes_of(state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceError;
    use crate::tasks::labels::TaskLabels;
    use crate::tasks::model::{RowQuery, TaskCapabilities, TaskColumn, TaskSheet};
    use crate::tasks::shown::Variant;
    use crate::tasks::source::TaskSource;

    struct Two;

    impl TaskSource for Two {
        fn capabilities(&self) -> TaskCapabilities {
            TaskCapabilities::all()
        }
        fn columns(&self) -> Vec<TaskColumn> {
            vec![TaskColumn::new("todo", "To do")]
        }
        fn rows(
            &self,
            query: &RowQuery,
            _offset: usize,
            _limit: usize,
        ) -> Result<Vec<TaskRow>, SourceError> {
            Ok(["apple", "pear"]
                .iter()
                .filter(|key| key.contains(&query.term))
                .map(|key| TaskRow::new(*key, key.to_uppercase(), "todo").guarded("Marked"))
                .collect())
        }
        fn read(&self, _key: &str) -> Result<TaskSheet, SourceError> {
            Ok(TaskSheet::default())
        }
    }

    #[test]
    fn while_a_card_is_held_a_search_leaves_the_other_cards_as_they_are() {
        let mut state = TasksState::new(Arc::new(Two), TaskLabels::default(), Variant::Full);
        state.set_view(TaskView::Status);
        let board = Board::new("t".into(), &state);
        let pear = Item::new("pear", "PEAR");
        assert!(matches!(board.grab("apple"), Outcome::Grabbed(_)));
        state.search("apple");
        board.sync(&state);
        let card = board.cards.card(&pear);
        assert_eq!(card.title, "PEAR", "not its key");
        assert_eq!(card.marks, ["Marked"], "and still marked");
    }

    #[test]
    fn opening_a_card_shows_on_its_card() {
        let mut state = TasksState::new(Arc::new(Two), TaskLabels::default(), Variant::Full);
        let board = Board::new("t".into(), &state);
        state.select("pear");
        board.sync(&state);
        assert!(board.cards.card(&Item::new("pear", "PEAR")).open);
        assert!(!board.cards.card(&Item::new("apple", "APPLE")).open);
    }
}
