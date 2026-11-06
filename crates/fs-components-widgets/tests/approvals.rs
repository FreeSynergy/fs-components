//! Answers to questions: armed only after a moment, bound to what was drawn, once, never by a held
//! key or a click aimed at something else. Nothing is answered by the component itself.
//!
//! Pattern: the real element in a tree driven like a window (frames, not rebuilds), a clock moved by
//! hand.

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};

use chat_support::Manual;
use common::{Input, frame, layout, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Approvals, Decision, Outcome, ToolCall, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk,
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
    let approvals = Approvals::with_clock(clock.clone());
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let view = ToolCallView::new()
        .id(id)
        .approvals(approvals)
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    Rig { clock, view, got }
}

fn click(tree: &mut blinc_layout::prelude::RenderTree, id: &str) {
    Input::new().click(tree, id);
}

#[test]
fn a_new_question_does_not_answer_until_armed_and_then_answers_once() {
    let _g = serial();
    let r = rig("a1");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a1-opt-q1-once");
    assert!(
        r.got.lock().unwrap().is_empty(),
        "a button answered at once"
    );
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a1-opt-q1-once");
    click(&mut tree, "a1-opt-q1-once");
    let seen = r.got.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "answered twice: {seen:?}");
    let Outcome::Chose(d) = &seen[0] else {
        panic!()
    };
    assert_eq!(
        (d.call.as_str(), d.option.as_str(), d.revision),
        ("q1", "once", 1)
    );
}

#[test]
fn a_double_click_does_not_answer_the_next_question_in_the_same_place() {
    let _g = serial();
    let r = rig("a2");
    r.view.apply(q("q1", 1, "rm ./cache"));
    drop(layout(r.view.build(), 520.0, 400.0)); // shown
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a2-opt-q1-once");
    assert_eq!(r.got.lock().unwrap().len(), 1);
    // the host puts the next question in the same place
    r.view
        .replace(vec![(String::new(), q("q2", 1, "rm -rf ~"))]);
    frame(&mut tree, 520.0, 400.0);
    r.clock.advance(80);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a2-opt-q2-once");
    assert_eq!(
        r.got.lock().unwrap().len(),
        1,
        "the second click approved q2"
    );
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a2-opt-q2-once");
    assert_eq!(r.got.lock().unwrap().len(), 2);
}

#[test]
fn an_answer_is_bound_to_the_revision_drawn_and_a_press_across_an_update_is_dropped() {
    let _g = serial();
    let r = rig("a3");
    r.view.apply(q("q1", 1, "rm ./cache"));
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    let (x, y) = Input::centre(&tree, "a3-opt-q1-once");
    let mut input = Input::new();
    input.press_at(&mut tree, x, y);
    // the call changes under the pointer before the release
    r.view.apply(q("q1", 2, "rm -rf ~"));
    frame(&mut tree, 520.0, 400.0);
    input.release_at(&mut tree, x, y);
    assert!(
        r.got.lock().unwrap().is_empty(),
        "a press across an update answered"
    );
    // after the new question is armed, the answer carries what was drawn
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    click(&mut tree, "a3-opt-q1-once");
    let seen = r.got.lock().unwrap().clone();
    let Outcome::Chose(d) = &seen[0] else {
        panic!("{seen:?}")
    };
    assert_eq!(
        (d.revision, d.digest.as_slice()),
        (2, b"rm -rf ~".as_slice())
    );
}

#[test]
fn an_answer_that_no_longer_fits_the_current_call_is_dropped() {
    let _g = serial();
    let r = rig("a4");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let drawn = Decision::for_test("", "q1", "once", 1, b"rm ./cache");
    assert!(matches!(r.view.answer(drawn.clone()), Outcome::Chose(_)));
    r.view.apply(q("q1", 2, "rm -rf ~"));
    assert_eq!(
        r.view.answer(drawn.clone()),
        Outcome::NotAsked,
        "a stale revision answered"
    );
    r.view.apply(q("q1", 3, "x").settled(ToolEnd::Refused));
    assert_eq!(
        r.view.answer(Decision::for_test("", "q1", "once", 3, b"x")),
        Outcome::NotAsked
    );
    // settled is final: a list from the host cannot revive it
    r.view
        .replace(vec![(String::new(), q("q1", 9, "rm -rf ~"))]);
    assert_eq!(r.view.calls()[0].end, ToolEnd::Refused);
    // nobody answered on its own
    assert_eq!(r.got.lock().unwrap().len(), 1);
}

#[test]
fn releasing_on_another_button_or_after_the_pointer_left_answers_nothing() {
    let _g = serial();
    let r = rig("a5");
    r.view.apply(q("q1", 1, "rm ./cache"));
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    let (ax, ay) = Input::centre(&tree, "a5-opt-q1-once");
    let (rx, ry) = Input::centre(&tree, "a5-opt-q1-no");
    let mut input = Input::new();
    input.press_at(&mut tree, ax, ay);
    input.release_at(&mut tree, rx, ry);
    assert!(
        r.got.lock().unwrap().is_empty(),
        "drag from Allow to Refuse answered: {:?}",
        r.got.lock().unwrap()
    );
}

#[test]
fn two_places_for_one_question_the_first_answer_wins() {
    let _g = serial();
    let clock = Arc::new(Manual::default());
    let shared = Approvals::with_clock(clock.clone());
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let mk = |id: &str| {
        let sink = Arc::clone(&got);
        let v = ToolCallView::new()
            .id(id)
            .approvals(shared.clone())
            .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
        v.apply_in("m1", q("q1", 1, "rm ./cache"));
        v
    };
    let (a, b) = (mk("pa"), mk("pb"));
    drop((
        layout(a.build(), 520.0, 400.0),
        layout(b.build(), 520.0, 400.0),
    )); // shown
    clock.advance(701);
    let mut ta = layout(a.build(), 520.0, 400.0);
    let mut tb = layout(b.build(), 520.0, 400.0);
    click(&mut ta, "pa-m1-opt-q1-once");
    click(&mut tb, "pb-m1-opt-q1-no");
    let seen = got.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "both places reported: {seen:?}");
}

#[test]
fn the_same_call_id_in_two_messages_reports_the_right_message() {
    let _g = serial();
    let r = rig("a6");
    r.view.replace(vec![
        ("m1".into(), q("q1", 1, "rm a")),
        ("m2".into(), q("q1", 1, "rm b")),
    ]);
    drop(layout(r.view.build(), 520.0, 700.0)); // shown
    r.clock.advance(701);
    let mut tree = layout(r.view.build(), 520.0, 700.0);
    click(&mut tree, "a6-m2-opt-q1-once");
    let seen = r.got.lock().unwrap().clone();
    let Outcome::Chose(d) = &seen[0] else {
        panic!("{seen:?}")
    };
    assert_eq!(
        (d.message.as_str(), d.digest.as_slice()),
        ("m2", b"rm b".as_slice())
    );
}

#[test]
fn the_engines_text_is_visible_without_opening_and_the_buttons_never_leave_the_view() {
    let _g = serial();
    let r = rig("a7");
    let mut long = q("q1", 1, "rm -rf ~");
    long.summary = "A very long explanation by the agent. ".repeat(80);
    r.view.apply(long);
    let tree = layout(r.view.build(), 520.0, 400.0);
    assert!(
        tree.query_by_id("a7-detail-q1").is_some(),
        "the engine's text is hidden behind a button"
    );
    let node = tree.query_by_id("a7-opt-q1-once").unwrap();
    let b = tree.get_absolute_bounds(node).unwrap();
    assert!(
        b.y + b.height <= 400.0,
        "the answer button is out of view at y={}",
        b.y
    );
}

#[test]
fn a_refusal_of_the_engine_is_shown_on_the_card() {
    let _g = serial();
    let r = rig("a8");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = layout(r.view.build(), 520.0, 400.0);
    assert!(tree.query_by_id("a8-refused-q1").is_none());
    r.view.refused("q1", "the revision changed");
    frame(&mut tree, 520.0, 400.0);
    assert!(tree.query_by_id("a8-refused-q1").is_some());
}
