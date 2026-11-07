//! The answer guard against the real order of a window's input: the pointer routed to the pressed
//! button wherever it is, keys sent to everybody, questions that move under the pointer.
//!
//! Tests: the real element in a tree driven like a window drives it (events with their
//! rectangles, frames), a clock by hand.

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};

use chat_support::Manual;
use common::{Input, frame, layout, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Approvals, Outcome, ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk,
};

fn q(id: &str, rev: u64, command: &str) -> ToolCall {
    ToolCall::new(id, "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Clean up")
        .detail(command)
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
        .revision(rev)
        .digest(command.as_bytes().to_vec())
}

struct Rig {
    clock: Arc<Manual>,
    view: ToolCallView,
    got: Arc<Mutex<Vec<Outcome>>>,
}

fn rig(id: &str) -> Rig {
    let clock = Arc::new(Manual::default());
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let view = ToolCallView::new()
        .id(id)
        .approvals(Approvals::with_clock(clock.clone()))
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    Rig { clock, view, got }
}

impl Rig {
    fn tree(&self) -> blinc_layout::prelude::RenderTree {
        layout(self.view.build(), 520.0, 900.0)
    }

    /// Show the questions, let the delay pass, draw again.
    fn armed(&self) -> blinc_layout::prelude::RenderTree {
        drop(self.tree());
        self.clock.advance(701);
        self.tree()
    }

    fn answers(&self) -> usize {
        self.got.lock().unwrap().len()
    }
}

#[test]
fn typing_or_enter_anywhere_never_answers_a_question() {
    let _g = serial();
    let r = rig("k1");
    r.view.apply(q("q1", 1, "rm -rf ~"));
    let mut tree = r.armed();
    let mut input = Input::new();
    for code in [32, 13] {
        input.key_broadcast(&mut tree, code);
        input.key_release_broadcast(&mut tree, code);
    }
    assert_eq!(r.answers(), 0, "typed keys answered a question");
}

#[test]
fn the_keyboard_answers_only_from_the_focused_row_on_release_and_never_by_repeat() {
    use blinc_core::events::event_types::{KEY_DOWN, KEY_UP};
    let _g = serial();
    let r = rig("k2");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = r.armed();
    let mut input = Input::new();
    assert!(
        input.tab_to(&mut tree, "k2-keys-q1"),
        "the row is a tab stop"
    );
    input.key(&mut tree, 39); // Right: from Refuse to Allow once
    input.key_release(&mut tree, 39);
    for _ in 0..5 {
        input.key(&mut tree, 13); // a held key repeats the press
    }
    assert_eq!(r.answers(), 0, "key repeat answered");
    input.key_release(&mut tree, 13);
    let seen = r.got.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "the focused row did not answer on release");
    let Outcome::Chose(d) = &seen[0] else {
        panic!()
    };
    assert_eq!(d.option, "once");
    // without the focus the same keys do nothing
    let r2 = rig("k4");
    r2.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree2 = r2.armed();
    let mut other = Input::new();
    other.focus(&tree2, "k4-card-q1");
    other.key_event_to(&mut tree2, "k4-keys-q1", KEY_DOWN, 13);
    other.key_event_to(&mut tree2, "k4-keys-q1", KEY_UP, 13);
    assert_eq!(
        r2.answers(),
        0,
        "a key answered while another element had the focus"
    );
}

#[test]
fn focus_on_a_replaced_question_does_not_answer_the_next_one() {
    let _g = serial();
    let r = rig("k3");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = r.armed();
    let mut input = Input::new();
    input.focus(&tree, "k3-keys-q1");
    input.key(&mut tree, 13);
    r.view
        .replace(vec![(String::new(), q("q2", 1, "rm -rf ~"))]);
    let mut tree = r.tree();
    input.key_release(&mut tree, 13);
    assert_eq!(
        r.answers(),
        0,
        "the release answered the question that replaced the one pressed"
    );
}

#[test]
fn a_question_moving_into_the_place_of_another_is_disarmed() {
    let _g = serial();
    let r = rig("m1");
    r.view.replace(vec![
        (String::new(), q("a", 1, "rm a")),
        (String::new(), q("b", 1, "rm -rf ~")),
        (String::new(), q("c", 1, "rm c")),
    ]);
    let mut tree = r.armed();
    let (x, y) = Input::centre(&tree, "m1-opt-a-once");
    Input::new().click_at(&mut tree, x, y);
    assert_eq!(r.answers(), 1);
    // a leaves; b moves up into the pixels of a's button, 60 ms later
    r.view.replace(vec![
        (String::new(), q("b", 1, "rm -rf ~")),
        (String::new(), q("c", 1, "rm c")),
    ]);
    frame(&mut tree, 520.0, 900.0);
    r.clock.advance(60);
    let mut tree = r.tree();
    Input::new().click_at(&mut tree, x, y);
    assert_eq!(r.answers(), 1, "b was answered by the click aimed at a");
}

#[test]
fn swapping_removing_and_bringing_back_disarm_and_an_answered_call_does_not_rearm() {
    let _g = serial();
    let r = rig("s1");
    r.view.replace(vec![
        (String::new(), q("a", 1, "rm a")),
        (String::new(), q("b", 1, "rm b")),
    ]);
    let tree = r.armed();
    let (x, y) = Input::centre(&tree, "s1-opt-a-once");
    // swap: what is under the pointer is now b
    r.view.replace(vec![
        (String::new(), q("b", 1, "rm b")),
        (String::new(), q("a", 1, "rm a")),
    ]);
    let mut tree2 = r.tree();
    Input::new().click_at(&mut tree2, x, y);
    assert_eq!(r.answers(), 0, "a swap answered at once");
    // away and back, no time passing
    r.clock.advance(701);
    r.view.replace(vec![(String::new(), q("b", 1, "rm b"))]);
    drop(r.tree());
    r.view.replace(vec![
        (String::new(), q("b", 1, "rm b")),
        (String::new(), q("a", 1, "rm a")),
    ]);
    let mut tree3 = r.tree();
    Input::new().click(&mut tree3, "s1-opt-a-once");
    assert_eq!(
        r.answers(),
        0,
        "a question that came back was armed at once"
    );
    // answered once, it never arms again for the same revision
    r.clock.advance(701);
    let mut tree4 = r.tree();
    Input::new().click(&mut tree4, "s1-opt-a-once");
    assert_eq!(r.answers(), 1);
    r.view.replace(vec![(String::new(), q("a", 1, "rm a"))]);
    drop(r.tree());
    r.clock.advance(5000);
    let mut tree5 = r.tree();
    Input::new().click(&mut tree5, "s1-opt-a-once");
    assert_eq!(
        r.answers(),
        1,
        "an answered call armed again for the same revision"
    );
    drop(tree);
}

#[test]
fn a_release_far_from_the_button_answers_nothing() {
    let _g = serial();
    let r = rig("d1");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = r.armed();
    let (x, y) = Input::centre(&tree, "d1-opt-q1-once");
    let mut input = Input::new();
    input.press_at(&mut tree, x, y);
    // Blinc routes the release to the pressed element wherever the pointer is
    let node = tree.query_by_id("d1-opt-q1-once").unwrap();
    let b = tree.get_absolute_bounds(node).unwrap();
    tree.dispatch_event_with_local(
        node,
        blinc_core::events::event_types::POINTER_UP,
        900.0,
        800.0,
        900.0 - b.x,
        800.0 - b.y,
        b.x,
        b.y,
        b.width,
        b.height,
    );
    assert_eq!(r.answers(), 0, "a release far away approved");
}
