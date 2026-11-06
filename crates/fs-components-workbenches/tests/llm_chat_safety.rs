//! The chat's questions and the stop: armed answers bound to what was drawn, a dock that cannot
//! scroll away, a stop that is always reachable at every size.
//!
//! Pattern: the real element in a tree driven like a window drives it (frames), a clock by hand.

mod chat_common;

use std::sync::{Arc, Mutex};

use chat_common::{Chat, Driven, Manual, busy_chat, say};
use fs_components_widgets::SenderKind;
use fs_components_widgets::tool_call::{
    Approvals, ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk,
};
use fs_components_workbenches::llm_chat::{LlmChat, Outcome, Pending};

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

fn rig(c: &Arc<Chat>) -> (LlmChat, Arc<Mutex<Vec<Outcome>>>, Arc<Manual>) {
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let clock = Arc::new(Manual::default());
    let chat = LlmChat::new(c.clone())
        .approvals(Approvals::with_clock(clock.clone()))
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    (chat, got, clock)
}

#[test]
fn a_docked_question_answers_only_when_armed_and_carries_what_was_drawn() {
    let c = busy_chat();
    {
        let mut m = c.messages.lock().unwrap();
        for i in 5..300 {
            m.push(say(
                &i.to_string(),
                "Helper",
                SenderKind::Ai,
                "More output.",
            ));
        }
    }
    let (chat, got, clock) = rig(&c);
    let mut d = Driven::new(&chat, 900.0, 700.0);
    assert!(
        !d.has("llm-list-msg-2"),
        "the message is out of the built range"
    );
    d.click("llm-dock-2-opt-q1-once");
    assert!(
        got.lock().unwrap().is_empty(),
        "a new question answered at once"
    );
    clock.advance(701);
    d.rebuild();
    d.click("llm-dock-2-opt-q1-once");
    let seen = got.lock().unwrap().clone();
    let Outcome::ToolChose(dec) = &seen[0] else {
        panic!("{seen:?}")
    };
    assert_eq!(
        (dec.message.as_str(), dec.call.as_str(), dec.revision),
        ("2", "q1", 1)
    );
    assert_eq!(dec.digest, b"rm -rf ./target");
}

#[test]
fn the_next_question_in_the_same_place_is_not_answered_by_the_same_double_click() {
    let c = busy_chat();
    let (chat, got, clock) = rig(&c);
    let mut d = Driven::new(&chat, 900.0, 700.0);
    clock.advance(701);
    d.rebuild();
    d.click("llm-dock-2-opt-q1-once");
    assert_eq!(got.lock().unwrap().len(), 1);
    *c.questions.lock().unwrap() = vec![Pending::new("2", q("q2", 1, "rm -rf ~"))];
    chat.refresh();
    d.frame();
    clock.advance(60);
    d.rebuild();
    d.click("llm-dock-2-opt-q2-once");
    assert_eq!(got.lock().unwrap().len(), 1, "the second click approved q2");
}

#[test]
fn a_question_gone_from_the_source_is_not_answered_from_a_stale_screen() {
    let c = busy_chat();
    let (chat, got, clock) = rig(&c);
    let mut d = Driven::new(&chat, 900.0, 700.0);
    clock.advance(701);
    d.rebuild();
    c.questions.lock().unwrap().clear();
    d.click("llm-dock-2-opt-q1-once"); // the host has not called refresh yet
    assert!(
        got.lock().unwrap().is_empty(),
        "a vanished question was answered"
    );
}

#[test]
fn a_refused_decision_is_shown_with_the_fresh_question() {
    let c = busy_chat();
    let (chat, _, _) = rig(&c);
    let mut d = Driven::new(&chat, 900.0, 700.0);
    *c.questions.lock().unwrap() = vec![Pending::new("2", q("q1", 2, "rm -rf ./other"))];
    chat.decision_refused("2", "q1", "the revision changed");
    d.frame();
    assert!(d.has("llm-dock-2-refused-q1"), "the refusal is not shown");
    assert!(
        d.has("llm-dock-2-opt-q1-once"),
        "the fresh question is not shown"
    );
}

#[test]
fn more_questions_make_a_taller_dock_and_the_first_buttons_stay_in_view() {
    let c = busy_chat();
    let (chat, _, _) = rig(&c);
    let mut d = Driven::new(&chat, 1000.0, 700.0);
    let (_, _, _, one) = d.bounds("llm-dockbox");
    let mut many = Vec::new();
    for i in 0..3 {
        let mut call = q(&format!("q{i}"), 1, "rm -rf ./target");
        call.summary = "A long explanation. ".repeat(60);
        many.push(Pending::new("2", call));
    }
    *c.questions.lock().unwrap() = many;
    chat.refresh();
    d.frame();
    let (_, _, _, three) = d.bounds("llm-dockbox");
    assert!(
        three > one,
        "the dock stays as it was for one question ({one} -> {three})"
    );
    let (_, y, _, h) = d.bounds("llm-dock-2-opt-q0-once");
    assert!(
        y + h <= 700.0,
        "the first question's buttons are out of the window"
    );
}

#[test]
fn the_stop_is_reachable_at_every_size_in_both_forms_whatever_the_title() {
    let c = busy_chat();
    *c.title.lock().unwrap() = "T".repeat(300);
    *c.usage.lock().unwrap() = "U".repeat(200);
    for compact in [false, true] {
        for (w, h) in [
            (1100.0, 720.0),
            (640.0, 480.0),
            (400.0, 400.0),
            (320.0, 360.0),
            (300.0, 360.0),
        ] {
            let (chat, got, _) = rig(&c);
            let chat = chat.compact(compact);
            let mut d = Driven::new(&chat, w, h);
            let (x, y, bw, bh) = d.bounds("llm-stop");
            assert!(
                x >= 0.0 && y >= 0.0 && x + bw <= w && y + bh <= h,
                "stop outside {w}x{h} (compact={compact}): {x},{y},{bw},{bh}"
            );
            d.click("llm-stop");
            assert_eq!(
                got.lock().unwrap().last(),
                Some(&Outcome::StopRequested),
                "the stop was not hit at {w}x{h} (compact={compact})"
            );
        }
    }
}
