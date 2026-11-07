//! The chat workbench: the full and the compact form, send, agents, refresh through a frame.
//!
//! Tests: the real element driven in process like a window drives it (frames, not rebuilds), and
//! offscreen pictures.

mod chat_common;

use std::sync::{Arc, Mutex};

use chat_common::{Chat, Driven, busy_chat, say, shot};
use fs_components_widgets::SenderKind;
use fs_components_widgets::tool_call::Approvals;
use fs_components_workbenches::llm_chat::{LlmChat, Outcome};

fn chat(c: &Arc<Chat>) -> (LlmChat, Arc<Mutex<Vec<Outcome>>>) {
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let clock = Arc::new(chat_common::Manual::default());
    let chat = LlmChat::new(c.clone())
        .approvals(Approvals::with_clock(clock))
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    (chat, got)
}

#[test]
fn the_full_chat_shows_messages_agents_dock_composer_and_stop() {
    let (chat, _) = chat(&busy_chat());
    let d = Driven::new(&chat, 1100.0, 720.0);
    for id in [
        "llm-stop",
        "llm-tree-row-h2",
        "llm-list-msg-4",
        "llm-dock-2-opt-q1-once",
        "llm-composer-send",
        "llm-dock-2-detail-q1",
    ] {
        assert!(d.has(id), "missing {id}");
    }
    drop(d);
    shot(&chat, 1100, 720, "llm-chat-full.png");
}

#[test]
fn every_sender_is_marked_in_the_full_and_in_the_compact_form() {
    let c = busy_chat();
    for compact in [false, true] {
        let (chat, _) = chat(&c);
        let chat = chat.compact(compact);
        let d = Driven::new(&chat, if compact { 520.0 } else { 1100.0 }, 700.0);
        // the marks of the messages in view; the list follows the end, so the last ones are built
        for key in ["3", "4"] {
            let (_, _, w, h) = d.bounds(&format!("llm-list-mark-{key}"));
            assert!(
                w > 20.0 && h > 8.0,
                "no mark on message {key} (compact={compact})"
            );
        }
        assert_eq!(
            d.has("llm-tree-row-h2"),
            !compact,
            "side panel (compact={compact})"
        );
        drop(d);
        shot(
            &chat,
            if compact { 520 } else { 1100 },
            700,
            if compact {
                "llm-chat-compact.png"
            } else {
                "llm-chat-full-2.png"
            },
        );
    }
}

#[test]
fn the_compact_header_says_what_needs_attention_and_what_works() {
    let (chat, _) = chat(&busy_chat());
    let chat = chat.compact(true);
    let a = shot(&chat, 520, 700, "llm-chat-compact-agents.png");
    let c2 = busy_chat();
    c2.agents.lock().unwrap().clear();
    let (quiet, _) = self::chat(&c2);
    let b = shot(&quiet.compact(true), 520, 700, "llm-chat-compact-quiet.png");
    assert_ne!(a, b, "the compact header does not show the agents");
}

#[test]
fn send_is_an_outcome_a_failure_keeps_the_text_and_success_clears_it() {
    let (chat, got) = chat(&busy_chat());
    chat.composer().set_text("Please go on.\n");
    let mut d = Driven::new(&chat, 900.0, 700.0);
    d.click("llm-composer-send");
    let Some(Outcome::Send { text, attempt, .. }) = got.lock().unwrap().last().cloned() else {
        panic!()
    };
    assert_eq!(text, "Please go on.\n");
    chat.failed("offline");
    assert_eq!(chat.composer().text(), "Please go on.\n");
    d.rebuild();
    d.click("llm-composer-send");
    let Some(Outcome::Send {
        attempt: second, ..
    }) = got.lock().unwrap().last().cloned()
    else {
        panic!()
    };
    // a late answer of the first attempt must not clear the text of the second
    chat.composer().sent_for(attempt);
    assert_eq!(
        chat.composer().text(),
        "Please go on.\n",
        "a late sent() of attempt 1 cleared attempt 2"
    );
    chat.composer().sent_for(second);
    assert_eq!(chat.composer().text(), "");
}

#[test]
fn an_agent_click_is_an_outcome() {
    let (chat, got) = chat(&busy_chat());
    let mut d = Driven::new(&chat, 1100.0, 700.0);
    d.click("llm-tree-main-h2");
    assert_eq!(
        *got.lock().unwrap(),
        [Outcome::AgentSelected { key: "h2".into() }]
    );
}

#[test]
fn new_messages_and_questions_reach_the_screen_through_a_frame() {
    let c = busy_chat();
    c.questions.lock().unwrap().clear();
    let (chat, _) = chat(&c);
    let mut d = Driven::new(&chat, 900.0, 700.0);
    assert!(!d.has("llm-dock-2-opt-q1-once"));
    c.messages
        .lock()
        .unwrap()
        .push(say("5", "Helper", SenderKind::Ai, "A new line."));
    chat.refresh();
    d.frame();
    assert!(
        d.has("llm-list-msg-5"),
        "the new message did not reach a frame"
    );
    // a question arriving changes the layout: it takes the next frame too
    let busy = busy_chat();
    *c.questions.lock().unwrap() = busy.questions.lock().unwrap().clone();
    chat.refresh();
    d.frame();
    assert!(
        d.has("llm-dock-2-opt-q1-once"),
        "the new question did not reach a frame"
    );
}
