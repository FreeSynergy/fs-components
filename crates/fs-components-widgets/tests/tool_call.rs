//! The tool call: update in place, the kind first, questions as outcomes, pictures.
//!
//! Tests: Model tests, pixel tests, an in-process mouse.

mod common;

use common::{background, draw, extent_x, ink, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Applied, ToolCall, ToolCalls, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk, ToolState,
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
fn details_open_on_request_and_stay_after_an_update() {
    let _g = serial();
    let view = ToolCallView::new().id("tc3");
    let working = || {
        ToolCall::new("c2", "shell")
            .kind(ToolKind::RunsCommand)
            .state(ToolState::Working)
            .summary("Remove the build folder")
            .detail("rm -rf ./target")
    };
    view.apply(working());
    let closed = draw(view.build(), 520, 300, "tool-call-closed.png");
    view.toggle("c2");
    assert!(view.is_expanded("c2"));
    let open = draw(view.build(), 520, 300, "tool-call-open.png");
    assert_ne!(closed, open);
    view.apply(working().summary("changed").revision(1));
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
    view.apply(ToolCall::new("1", "t").summary("x".repeat(400)));
    // the card is 360 wide in a picture 600 wide: ink beyond 360 means it ran out of its box
    let boxed = blinc_layout::div::div().w(360.0).child(view.build());
    let img = draw(boxed, 600, 400, "tool-call-long.png");
    let (_, right) = extent_x(&img, 0, 400).expect("ink");
    assert!(right < 362, "text ran out of the box: {right}");
    assert!(
        ink(&img, (0, 60, 360, 200), background(&img)) > 300,
        "the text is not drawn"
    );
}
