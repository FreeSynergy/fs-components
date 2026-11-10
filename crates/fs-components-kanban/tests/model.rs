//! Tests of the pure model: sizes, initials, moves by gesture, keys and buttons. No window.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_kanban::layout::{
    COLUMN_MIN_W, card_height, column_at, column_width, count_mark, initials,
};
use fs_components_kanban::state::locate;
use fs_components_kanban::*;
use fs_components_styles::{StyleId, style};
use fs_components_widgets::Key;

fn cols() -> Vec<Column> {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![Card::new("a", "A"), Card::new("b", "B")];
    let done = Column::new("done", "Done");
    let mut doing = Column::new("doing", "Doing");
    doing.cards = vec![Card::new("c", "C")];
    doing.limit = Some(0);
    vec![todo, doing, done]
}

#[test]
fn initials_count_marks_and_widths() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("ada"), "A");
    assert_eq!(initials("Jan van der Berg"), "JB");
    assert_eq!(initials("  "), "");
    let c = cols();
    assert_eq!(count_mark(&c[0]), "2");
    assert_eq!(count_mark(&c[1]), "1/0");
    assert!(c[1].over_limit() && !c[0].over_limit());
    assert_eq!(
        column_width(300.0, 3, 8.0),
        COLUMN_MIN_W,
        "never narrower than the minimum"
    );
    assert!((column_width(1000.0, 4, 8.0) - 244.0).abs() < 0.01);
    assert_eq!(column_at(10.0, 100.0, 10.0, 3), Some(0));
    assert_eq!(column_at(105.0, 100.0, 10.0, 3), None, "in the gap");
    assert_eq!(column_at(115.0, 100.0, 10.0, 3), Some(1));
    assert_eq!(column_at(400.0, 100.0, 10.0, 3), None);
}

#[test]
fn a_card_with_more_lines_is_higher_in_every_style() {
    for id in StyleId::ALL {
        let s = style(id);
        let plain = Card::new("a", "A");
        let mut full = plain.clone();
        full.subtitle = Some("sub".into());
        full.assignee = Some("Ada L".into());
        full.labels = vec![Label {
            text: "x".into(),
            tone: Tone::Accent,
        }];
        assert!(
            card_height(&full, s) > card_height(&plain, s) + 20.0,
            "{id:?}"
        );
    }
}

#[test]
fn grab_hover_drop_makes_a_move_request() {
    let c = cols();
    let mut st = BoardState::default();
    assert_eq!(st.grab(&c, true, "a"), Step::Changed);
    assert_eq!(st.hover(&c, "done"), Step::Changed);
    assert_eq!(st.hover(&c, "nowhere"), Step::Ignored);
    assert_eq!(
        st.drop_held(true),
        Step::Move(MoveRequest {
            key: "a".into(),
            from: "todo".into(),
            to: "done".into()
        })
    );
    assert!(st.held.is_none());
}

#[test]
fn dropping_where_it_was_picked_up_is_no_move_and_cancel_gives_up() {
    let c = cols();
    let mut st = BoardState::default();
    st.grab(&c, true, "a");
    assert_eq!(st.drop_held(true), Step::Changed);
    st.grab(&c, true, "a");
    st.hover(&c, "done");
    assert_eq!(st.cancel(), Step::Changed);
    assert_eq!(st.drop_held(true), Step::Ignored);
}

#[test]
fn a_locked_move_is_refused_with_a_code() {
    let c = cols();
    let mut st = BoardState::default();
    assert_eq!(st.grab(&c, false, "a"), Step::Refused("locked"));
    assert_eq!(st.notice.as_deref(), Some("locked"));
    assert_eq!(st.step_over(&c, false, "a", 1), Step::Refused("locked"));
    assert!(st.held.is_none());
}

#[test]
fn move_buttons_go_one_column_and_stop_at_the_ends() {
    let c = cols();
    let mut st = BoardState::default();
    assert_eq!(
        st.step_over(&c, true, "a", 1),
        Step::Move(MoveRequest {
            key: "a".into(),
            from: "todo".into(),
            to: "doing".into()
        })
    );
    assert_eq!(st.step_over(&c, true, "a", -1), Step::Ignored);
    assert_eq!(st.step_over(&c, true, "missing", 1), Step::Ignored);
}

#[test]
fn keys_walk_skip_empty_columns_grab_move_and_drop() {
    let c = cols();
    let mut st = BoardState::default();
    assert_eq!(st.key(&c, true, Key::Down), Step::Changed);
    assert_eq!(
        st.cursor.as_deref(),
        Some("a"),
        "first card of the first column"
    );
    st.key(&c, true, Key::Down);
    assert_eq!(st.cursor.as_deref(), Some("b"));
    st.key(&c, true, Key::Down);
    assert_eq!(st.cursor.as_deref(), Some("b"), "stays at the end");
    st.key(&c, true, Key::Right);
    assert_eq!(st.cursor.as_deref(), Some("c"), "row clamped");
    assert_eq!(
        st.key(&c, true, Key::Right),
        Step::Ignored,
        "the empty column after it is skipped, nothing beyond"
    );
    st.key(&c, true, Key::Activate);
    assert_eq!(st.held.as_ref().unwrap().key, "c");
    st.key(&c, true, Key::Right);
    assert_eq!(st.held.as_ref().unwrap().over, "done");
    assert_eq!(st.key(&c, true, Key::Escape), Step::Changed);
    assert!(st.held.is_none());
    st.key(&c, true, Key::Activate);
    st.key(&c, true, Key::Left);
    match st.key(&c, true, Key::Space) {
        Step::Move(m) => assert_eq!((m.key.as_str(), m.to.as_str()), ("c", "todo")),
        other => panic!("{other:?}"),
    }
    assert_eq!(st.key(&c, true, Key::Enter), Step::Select("c".into()));
}

#[test]
fn locate_finds_column_and_row() {
    assert_eq!(locate(&cols(), "b"), Some((0, 1)));
    assert_eq!(locate(&cols(), "zz"), None);
}
