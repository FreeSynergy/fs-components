//! The chat workbench: the full and the compact form, docked questions, send, stop, refresh.
//!
//! Pattern: the real element driven in process like a window drives it, and offscreen pictures.

mod chat_common;

use std::sync::{Arc, Mutex};

use chat_common::{Chat, Driven, say, shot, stop};
use fs_components_widgets::SenderKind;
use fs_components_widgets::agent_tree::{AgentEnd, AgentNode, AgentState};
use fs_components_widgets::tool_call::{ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk};
use fs_components_workbenches::llm_chat::{LlmChat, Outcome, Pending, StopState};

fn question() -> ToolCall {
    ToolCall::new("q1", "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Delete the old build folder")
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
}

fn busy_chat() -> Arc<Chat> {
    let c = Arc::new(Chat::default());
    {
        let mut m = c.messages.lock().unwrap();
        m.push(
            say(
                "1",
                "Ana",
                SenderKind::Person,
                "Can you prepare the release notes?",
            )
            .at("10:02"),
        );
        m.push(
            say(
                "2",
                "Helper",
                SenderKind::Ai,
                "Yes. I started two helpers: one collects the changes, one writes the text.",
            )
            .at("10:02"),
        );
        m.push(
            say(
                "3",
                "indexer",
                SenderKind::Software,
                "Index updated: 212 pages.",
            )
            .at("10:03"),
        );
        m.push(say("4", "Stranger", SenderKind::Unknown, "Who allowed this?").at("10:04"));
    }
    *c.agents.lock().unwrap() = vec![
        AgentNode::new("h", "Helper")
            .of_kind(SenderKind::Ai)
            .state(AgentState::Working)
            .summary("Plan the release"),
        AgentNode::new("h1", "Collector")
            .of_kind(SenderKind::Ai)
            .under("h")
            .settled(AgentEnd::Failed)
            .summary("tests broke"),
        AgentNode::new("h2", "Writer")
            .of_kind(SenderKind::Ai)
            .under("h")
            .state(AgentState::Question)
            .summary("May I delete the build?"),
    ];
    *c.questions.lock().unwrap() = vec![Pending::new("2", question())];
    *c.stop.lock().unwrap() = stop(StopState::Ready);
    c
}

fn chat(c: &Arc<Chat>) -> (LlmChat, Arc<Mutex<Vec<Outcome>>>) {
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let chat =
        LlmChat::new(c.clone()).on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    (chat, got)
}

#[test]
fn the_full_chat_shows_messages_agents_dock_composer_and_stop() {
    let c = busy_chat();
    let (chat, _) = chat(&c);
    let mut d = Driven::new(&chat, 1100.0, 720.0);
    assert!(d.has("llm-stop"), "no stop control");
    assert!(d.has("llm-tree-row-h2"), "no agent tree");
    assert!(d.has("llm-list-msg-4"), "no messages");
    assert!(d.has("llm-dock-opt-q1-once"), "the question is not docked");
    assert!(d.has("llm-composer-send"), "no text field");
    d.rebuild();
    let img = shot(&chat, 1100, 720, "llm-chat-full.png");
    assert!(img.width() == 1100);
}

#[test]
fn the_compact_form_has_no_side_panel_but_keeps_marks_and_attention() {
    let c = busy_chat();
    let (full, _) = chat(&c);
    let (compact, _) = chat(&c);
    let compact = compact.compact(true);
    let d = Driven::new(&compact, 520.0, 640.0);
    assert!(
        !d.has("llm-tree-row-h2"),
        "the side panel is in the compact form"
    );
    assert!(d.has("llm-list-msg-2") && d.has("llm-composer-send") && d.has("llm-dock-opt-q1-once"));
    let a = shot(&compact, 520, 640, "llm-chat-compact.png");
    // the marks are drawn in the compact form: an AI message and a person's differ in the header
    let ink_in = |img: &image::RgbaImage| {
        img.pixels()
            .filter(|p| i32::from(p.0[2]) > i32::from(p.0[0]) + 25)
            .count()
    };
    assert!(
        ink_in(&a) > 200,
        "no blue mark of the AI in the compact form"
    );
    let b = shot(&full, 1100, 720, "llm-chat-full-2.png");
    assert_ne!(a.width(), b.width());
}

#[test]
fn a_question_cannot_scroll_away_in_a_long_conversation() {
    let c = busy_chat();
    {
        let mut m = c.messages.lock().unwrap();
        for i in 5..400 {
            m.push(say(
                &i.to_string(),
                "Helper",
                SenderKind::Ai,
                "More output from the agent, line after line of it.",
            ));
        }
    }
    let (chat, got) = chat(&c);
    let mut d = Driven::new(&chat, 900.0, 600.0);
    // the message the question belongs to is far above, the answer is still there
    assert!(!d.has("llm-list-msg-2"));
    assert!(d.has("llm-dock-opt-q1-once"));
    assert!(
        got.lock().unwrap().is_empty(),
        "something was answered by itself"
    );
    d.click("llm-dock-opt-q1-once");
    assert_eq!(
        *got.lock().unwrap(),
        [Outcome::ToolChose {
            message: "2".into(),
            call: "q1".into(),
            option: "once".into(),
            choice: ToolChoice::AllowOnce
        }]
    );
}

#[test]
fn send_is_an_outcome_a_failure_keeps_the_text_and_success_clears_it() {
    let c = busy_chat();
    let (chat, got) = chat(&c);
    chat.composer().set_text("Please go on.\n");
    let mut d = Driven::new(&chat, 900.0, 600.0);
    d.click("llm-composer-send");
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Send {
            text: "Please go on.\n".into(),
            attachments: vec![]
        })
    );
    chat.failed("offline");
    assert_eq!(chat.composer().text(), "Please go on.\n");
    d.rebuild();
    d.click("llm-composer-send");
    chat.sent();
    assert_eq!(chat.composer().text(), "");
}

#[test]
fn the_stop_is_only_shown_and_reported() {
    let c = busy_chat();
    let (chat, got) = chat(&c);
    let mut d = Driven::new(&chat, 900.0, 600.0);
    d.click("llm-stop");
    assert_eq!(*got.lock().unwrap(), [Outcome::StopRequested]);
    // the workbench did nothing itself: the source still says ready
    assert_eq!(
        c.stop.lock().unwrap().as_ref().unwrap().state,
        StopState::Ready
    );
    for state in [
        StopState::Stopping,
        StopState::Stopped,
        StopState::Unavailable,
    ] {
        *c.stop.lock().unwrap() = chat_common::stop(state);
        chat.refresh();
        d.rebuild();
        assert!(!d.has("llm-stop"), "a button while {state:?}");
    }
    *c.stop.lock().unwrap() = None;
    chat.refresh();
    d.rebuild();
    assert!(!d.has("llm-stop"), "a control the host did not give");
}

#[test]
fn an_agent_click_is_an_outcome() {
    let c = busy_chat();
    let (chat, got) = chat(&c);
    let mut d = Driven::new(&chat, 1100.0, 600.0);
    d.click("llm-tree-main-h2");
    assert_eq!(
        *got.lock().unwrap(),
        [Outcome::AgentSelected { key: "h2".into() }]
    );
}

#[test]
fn new_messages_and_a_new_question_appear_after_refresh() {
    let c = busy_chat();
    c.questions.lock().unwrap().clear();
    let (chat, _) = chat(&c);
    let mut d = Driven::new(&chat, 900.0, 600.0);
    assert!(!d.has("llm-dock-opt-q1-once"));
    c.messages
        .lock()
        .unwrap()
        .push(say("5", "Helper", SenderKind::Ai, "A new line."));
    *c.questions.lock().unwrap() = vec![Pending::new("5", question())];
    chat.refresh();
    d.frame();
    d.rebuild();
    assert!(d.has("llm-list-msg-5"), "the new message is not there");
    assert!(
        d.has("llm-dock-opt-q1-once"),
        "the new question is not docked"
    );
    let img = shot(&chat, 900, 600, "llm-chat-refreshed.png");
    assert!(img.width() == 900);
}
