//! Tests of the pure model: refusals while putting down, keys with nowhere to go, drop_at.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_kanban::*;
use fs_components_widgets::Key;

fn cols() -> Vec<Column> {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![Card::new("a", "A"), Card::new("b", "B")];
    let done = Column::new("done", "Done");
    vec![todo, done]
}

#[test]
fn putting_down_asks_again_whether_moves_are_still_supported() {
    let c = cols();
    let mut st = BoardState::default();
    st.grab(&c, true, "a");
    st.hover(&c, "done");
    assert_eq!(st.drop_held(false), Step::Refused("locked"));
    assert!(st.held.is_none(), "the card is let go");
    st.grab(&c, true, "a");
    assert_eq!(
        st.key(&c, false, Key::Activate),
        Step::Refused("locked"),
        "also by keyboard"
    );
}

#[test]
fn a_key_with_nowhere_to_go_changes_nothing() {
    let c = cols();
    let mut st = BoardState::default();
    st.key(&c, true, Key::Down);
    assert_eq!(
        st.key(&c, true, Key::Up),
        Step::Ignored,
        "already on the first card"
    );
    st.key(&c, true, Key::Down);
    assert_eq!(
        st.key(&c, true, Key::Down),
        Step::Ignored,
        "already on the last card"
    );
    assert_eq!(
        st.key(&c, true, Key::Left),
        Step::Ignored,
        "no column to the left"
    );
}

#[test]
fn drop_at_holds_over_the_column_and_puts_down_in_one_step() {
    let c = cols();
    let mut st = BoardState::default();
    st.grab(&c, true, "a");
    assert_eq!(
        st.drop_at(&c, true, "done"),
        Step::Move(MoveRequest {
            key: "a".into(),
            from: "todo".into(),
            to: "done".into()
        })
    );
    assert_eq!(st.drop_at(&c, true, "done"), Step::Ignored, "nothing held");
}
