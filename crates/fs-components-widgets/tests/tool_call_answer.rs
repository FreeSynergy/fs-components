//! What `answer` accepts, and how the store takes a list.
//!
//! Pattern: Model tests.

mod common;

use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Decision, Outcome, ToolCall, ToolCalls, ToolChoice, ToolEnd, ToolOption, ToolState,
};

fn question(digest: &[u8]) -> ToolCall {
    ToolCall::new("q1", "shell")
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
        .revision(1)
        .digest(digest.to_vec())
}

#[test]
fn an_answer_must_say_what_the_option_it_names_means() {
    let view = ToolCallView::new().id("a1");
    view.apply(question(b""));
    // an empty digest: Allow cannot be had by calling the option "once" a refusal
    let mut sneaky = Decision::for_test("", "q1", "once", 1, b"");
    sneaky.choice = ToolChoice::Refuse;
    assert!(matches!(view.answer(sneaky), Outcome::NotAsked));
    // and the other way round: a refusal dressed as an allowance
    let view = ToolCallView::new().id("a2");
    view.apply(question(b"ls"));
    let mut wrong = Decision::for_test("", "q1", "no", 1, b"ls");
    wrong.choice = ToolChoice::AllowOnce;
    assert!(matches!(view.answer(wrong), Outcome::NotAsked));
    // the honest ones pass
    let mut refuse = Decision::for_test("", "q1", "no", 1, b"ls");
    refuse.choice = ToolChoice::Refuse;
    assert!(matches!(view.answer(refuse), Outcome::Chose(_)));
}

#[test]
fn a_list_naming_one_call_twice_holds_it_once() {
    let mut calls = ToolCalls::new();
    let a = question(b"x");
    calls.replace(vec![("m".into(), a.clone()), ("m".into(), a.clone())]);
    assert_eq!(calls.len(), 1);
    let done = a.clone().settled(ToolEnd::Done);
    calls.replace(vec![("m".into(), done.clone())]);
    calls.replace(vec![
        ("m".into(), done.clone()),
        ("m".into(), done),
        ("m".into(), a),
    ]);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls.get_in("m", "q1").unwrap().state, ToolState::Settled);
}
