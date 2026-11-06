//! The tool call: update in place, the kind first, questions as outcomes, pictures.
//!
//! Pattern: Model tests, pixel tests, an in-process mouse.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, background, draw, extent_x, ink, layout, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Applied, Outcome, ToolCall, ToolCalls, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk,
    ToolState,
};

fn options() -> Vec<ToolOption> {
    vec![
        ToolOption::new("no", "Refuse", ToolChoice::Refuse),
        ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
    ]
}

fn question() -> ToolCall {
    ToolCall::new("c2", "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Remove the build folder")
        .detail("rm -rf ./target")
        .asking(options())
}

#[test]
fn a_call_is_updated_in_place_by_id() {
    let mut calls = ToolCalls::new();
    assert_eq!(
        calls.apply(ToolCall::new("a", "read").revision(1)),
        Applied::Added
    );
    assert_eq!(
        calls.apply(ToolCall::new("b", "write").revision(1)),
        Applied::Added
    );
    let next = ToolCall::new("a", "read")
        .state(ToolState::Working)
        .revision(2);
    assert_eq!(calls.apply(next), Applied::Updated);
    assert_eq!(calls.len(), 2, "an update must not append a second entry");
    assert_eq!(calls.all()[0].id, "a");
    assert_eq!(calls.get("a").unwrap().state, ToolState::Working);
    // a lower revision is ignored
    assert_eq!(
        calls.apply(ToolCall::new("a", "old").revision(1)),
        Applied::Stale
    );
    assert_eq!(calls.get("a").unwrap().name, "read");
    // settled is final
    let done = ToolCall::new("a", "read")
        .settled(ToolEnd::Done)
        .revision(3);
    assert_eq!(calls.apply(done), Applied::Updated);
    let later = ToolCall::new("a", "read")
        .state(ToolState::Working)
        .revision(9);
    assert_eq!(calls.apply(later), Applied::AlreadySettled);
    assert_eq!(calls.get("a").unwrap().state, ToolState::Settled);
}

#[test]
fn unset_things_read_as_unknown() {
    let c = ToolCall::new("x", "t");
    assert_eq!(c.kind, ToolKind::Unknown);
    assert_eq!(c.risk, ToolRisk::Unknown);
    assert_eq!(c.end, ToolEnd::Open);
    assert!(!c.needs_attention());
    assert!(question().needs_attention());
    assert!(
        ToolCall::new("f", "t")
            .settled(ToolEnd::Failed)
            .needs_attention()
    );
    assert!(
        !ToolCall::new("d", "t")
            .settled(ToolEnd::Done)
            .needs_attention()
    );
}

#[test]
fn a_click_on_an_option_is_an_outcome_and_nothing_is_decided_before() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let view = ToolCallView::new()
        .id("tc1")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    view.apply(question());
    let mut tree = layout(view.build(), 520.0, 300.0);
    assert!(got.lock().unwrap().is_empty(), "drawing must never answer");
    // the options are drawn although nothing is expanded
    assert!(tree.query_by_id("tc1-opt-c2-once").is_some());
    Input::new().click(&mut tree, "tc1-opt-c2-once");
    let seen = got.lock().unwrap().clone();
    assert_eq!(
        seen,
        [Outcome::Chose {
            call: "c2".into(),
            option: "once".into(),
            choice: ToolChoice::AllowOnce
        }]
    );
    // the view decided nothing: the call is still a question
    assert_eq!(view.calls()[0].state, ToolState::Question);
}

#[test]
fn only_an_offered_option_of_a_question_answers() {
    let _g = serial();
    let view = ToolCallView::new();
    view.apply(question());
    assert_eq!(view.choose("c2", "always"), Outcome::NotAsked);
    assert_eq!(view.choose("nope", "once"), Outcome::NotAsked);
    view.apply(question().settled(ToolEnd::Refused).revision(1));
    assert_eq!(view.choose("c2", "once"), Outcome::NotAsked);
    let tree = layout(view.build(), 520.0, 300.0);
    assert!(tree.query_by_id("fs-toolcall-opt-c2-once").is_none());
}

#[test]
fn details_open_on_request_and_stay_after_an_update() {
    let _g = serial();
    let view = ToolCallView::new().id("tc3");
    view.apply(question());
    let closed = draw(view.build(), 520, 300, "tool-call-closed.png");
    view.toggle("c2");
    assert!(view.is_expanded("c2"));
    let open = draw(view.build(), 520, 300, "tool-call-open.png");
    assert_ne!(closed, open);
    view.apply(question().summary("changed").revision(1));
    assert!(view.is_expanded("c2"), "an update keeps the call open");
}

#[test]
fn the_pictures_of_the_states_and_kinds_differ() {
    let _g = serial();
    let all = [
        ToolCall::new("1", "read_file")
            .kind(ToolKind::Reads)
            .risk(ToolRisk::ReadsOnly)
            .state(ToolState::Working)
            .summary("Reading notes.md"),
        ToolCall::new("2", "write_file")
            .kind(ToolKind::Writes)
            .risk(ToolRisk::Changes)
            .state(ToolState::Forming)
            .summary("Preparing an edit"),
        question(),
        ToolCall::new("4", "wiki.search")
            .kind(ToolKind::ConnectedTool)
            .risk(ToolRisk::ReadsOnly)
            .settled(ToolEnd::Done)
            .summary("3 pages found")
            .result("a\nb\nc"),
        ToolCall::new("5", "mystery")
            .summary("no kind given")
            .settled(ToolEnd::Failed)
            .detail("exit code 2"),
    ];
    let view = ToolCallView::new().id("tc4");
    for c in &all {
        view.apply(c.clone());
    }
    view.toggle("5");
    let img = draw(view.build(), 560, 640, "tool-call-all.png");
    let bg = background(&img);
    assert!(ink(&img, (0, 0, 560, 640), bg) > 6000);
    // each single card differs from the others
    let mut shots = Vec::new();
    for c in &all {
        let one = ToolCallView::new().id("tc5");
        one.apply(c.clone());
        shots.push(draw(
            one.build(),
            560,
            160,
            &format!("tool-call-{}.png", c.id),
        ));
    }
    for i in 0..shots.len() {
        for j in i + 1..shots.len() {
            assert_ne!(shots[i], shots[j], "cards {i} and {j} look alike");
        }
    }
}

#[test]
fn the_kind_changes_the_picture_even_with_the_same_text() {
    let _g = serial();
    let a = ToolCallView::new();
    a.apply(ToolCall::new("1", "same").kind(ToolKind::Reads));
    let b = ToolCallView::new();
    b.apply(ToolCall::new("1", "same").kind(ToolKind::RunsCommand));
    let ia = draw(a.build(), 520, 100, "tool-call-kind-reads.png");
    let ib = draw(b.build(), 520, 100, "tool-call-kind-runs.png");
    assert_ne!(ia, ib);
}

#[test]
fn a_long_word_wraps_inside_the_card() {
    let _g = serial();
    let view = ToolCallView::new();
    let word = "x".repeat(120);
    view.apply(ToolCall::new("1", "t").summary(word));
    let img = draw(view.build(), 360, 220, "tool-call-long.png");
    let (_, right) = extent_x(&img, 0, 220).expect("ink");
    assert!(right < 360, "text ran out of the box: {right}");
}
