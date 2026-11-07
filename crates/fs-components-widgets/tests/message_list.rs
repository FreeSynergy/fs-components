//! The message list: sender marks, delivery, tool calls, references, pictures.
//!
//! Tests: pixel tests and an in-process mouse over the same view a window draws.

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};

use chat_support::{ai, msg, person, unknown};
use common::{Input, background, draw, extent_x, ink, layout, serial};
use fs_components_widgets::message_list::{
    ChatMessage, Delivery, MessageList, MessageSource, Outcome, Reference, SourceError, VecMessages,
};
use fs_components_widgets::tool_call::{
    Approvals, ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk,
};
use fs_components_widgets::{Block, Inline, Sender, SenderKind};

fn list_of(messages: Vec<ChatMessage>) -> MessageList {
    MessageList::new(Arc::new(VecMessages(messages)))
}

#[test]
fn the_kind_of_the_sender_changes_the_picture_and_cannot_be_left_out() {
    let _g = serial();
    let kinds = [
        person("p", "Sam"),
        ai("a", "Sam"),
        msg(
            "s",
            Sender::new("s", "Sam").of_kind(SenderKind::Software),
            "Hello there.",
        ),
        unknown("u", "Sam"),
    ];
    let mut shots = Vec::new();
    for (i, m) in kinds.iter().enumerate() {
        let l = list_of(vec![m.clone()]).id(format!("k{i}"));
        shots.push(draw(
            l.build(520.0, 110.0),
            520,
            110,
            &format!("message-kind-{i}.png"),
        ));
    }
    for i in 0..4 {
        for j in i + 1..4 {
            assert_ne!(shots[i], shots[j], "kinds {i} and {j} look alike");
        }
    }
    // the mark is in the header of every one of them, a person too: count ink right of the name
    for (i, s) in shots.iter().enumerate() {
        let bg = s.get_pixel(500, 100).0; // the row's own background (not the border at the edge)
        assert!(
            ink(s, (90, 8, 120, 26), bg) > 40,
            "no mark drawn for kind {i}"
        );
    }
}

#[test]
fn an_unset_kind_reads_as_unknown_not_as_a_person() {
    let s = Sender::new("x", "Someone");
    assert_eq!(s.kind, SenderKind::Unknown);
    let m = msg("1", s, "hi");
    assert_eq!(m.sender.kind, SenderKind::Unknown);
}

#[test]
fn delivery_and_failures_are_visible_and_retry_is_an_outcome() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let l = list_of(vec![
        person("1", "Ana").mine(true).delivery(Delivery::Sent),
        person("2", "Ana").mine(true).failed("server said no"),
    ])
    .id("d")
    .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(l.build(520.0, 360.0), 520.0, 360.0);
    assert!(
        tree.query_by_id("d-retry-2").is_some(),
        "no retry on the failed message"
    );
    assert!(tree.query_by_id("d-retry-1").is_none());
    Input::new().click(&mut tree, "d-retry-2");
    assert_eq!(
        *got.lock().unwrap(),
        [Outcome::Retry {
            message: "2".into()
        }]
    );
    let img = draw(l.build(520.0, 360.0), 520, 360, "message-failed.png");
    let red = img
        .pixels()
        .filter(|p| i32::from(p.0[0]) > i32::from(p.0[1]) + 5 && p.0[0] > 200)
        .count();
    assert!(red > 300, "a failure is not tinted: {red}");
}

#[test]
fn a_tool_question_in_a_message_is_an_outcome_only() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let call = ToolCall::new("c1", "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Delete the folder")
        .digest(b"rm -rf ./cache".to_vec())
        .asking(vec![ToolOption::new(
            "ok",
            "Allow once",
            ToolChoice::AllowOnce,
        )]);
    let clock = Arc::new(chat_support::Manual::default());
    let src = Arc::new(chat_support::Shared(Mutex::new(vec![
        ai("1", "Helper").tool_calls(vec![call.clone()]),
    ])));
    let l = MessageList::new(src.clone())
        .id("t")
        .approvals(Approvals::with_clock(clock.clone()))
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    drop(layout(l.build(560.0, 360.0), 560.0, 360.0)); // shown
    clock.advance(701);
    let mut tree = layout(l.build(560.0, 360.0), 560.0, 360.0);
    assert!(got.lock().unwrap().is_empty());
    // the call is settled in the source while the old element is still on screen: nothing answers
    src.0.lock().unwrap()[0] = ai("1", "Helper").tool_calls(vec![
        call.clone()
            .settled(fs_components_widgets::tool_call::ToolEnd::Refused)
            .revision(1),
    ]);
    Input::new().click(&mut tree, "t-1-opt-c1-ok");
    assert!(got.lock().unwrap().is_empty(), "a settled card answered");
    // an open question answers, bound to what was drawn
    src.0.lock().unwrap()[0] = ai("1", "Helper").tool_calls(vec![call.clone()]);
    Input::new().click(&mut tree, "t-1-opt-c1-ok");
    let seen = got.lock().unwrap().clone();
    let Outcome::ToolChose(d) = &seen[0] else {
        panic!("{seen:?}")
    };
    assert_eq!(
        (d.message.as_str(), d.call.as_str(), d.option.as_str()),
        ("1", "c1", "ok")
    );
    draw(l.build(560.0, 360.0), 560, 360, "message-toolcall.png");
}

#[test]
fn references_show_the_hosts_words_and_report_a_click() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let l =
        list_of(vec![person("1", "Ana").references(vec![Reference::new(
            "task-7",
            "Fix the printer (open)",
        )])])
        .id("r")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(l.build(520.0, 260.0), 520.0, 260.0);
    Input::new().click(&mut tree, "r-ref-1-task-7");
    assert_eq!(
        *got.lock().unwrap(),
        [Outcome::Reference {
            message: "1".into(),
            reference: "task-7".into()
        }]
    );
}

#[test]
fn reasoning_is_apart_and_opens_on_request() {
    let _g = serial();
    let m = ai("1", "Helper").reasoning(vec![Block::Paragraph(Inline::plain(
        "because of the cache",
    ))]);
    let l = list_of(vec![m]).id("w");
    let closed = draw(
        l.build(520.0, 260.0),
        520,
        260,
        "message-reasoning-closed.png",
    );
    l.toggle_open("r:1");
    let open = draw(
        l.build(520.0, 260.0),
        520,
        260,
        "message-reasoning-open.png",
    );
    assert_ne!(closed, open);
}

#[test]
fn rich_text_unknown_blocks_and_long_words_are_drawn_not_dropped() {
    let _g = serial();
    let long = "y".repeat(150);
    let blocks = vec![
        Block::Heading {
            level: 2,
            text: Inline::plain("Title"),
        },
        Block::Code {
            lang: "rs".into(),
            text: "let x = 1;".into(),
        },
        Block::Plain("fallback text".into()),
        Block::Paragraph(Inline::plain(long)),
    ];
    let l = list_of(vec![ChatMessage::new(
        "1",
        Sender::new("a", "A").of_kind(SenderKind::Ai),
        blocks,
    )])
    .id("x");
    let img = draw(l.build(420.0, 360.0), 420, 360, "message-blocks.png");
    let (_, right) = extent_x(&img, 0, 360).expect("ink");
    assert!(right < 420, "ran out of the box: {right}");
    assert!(ink(&img, (0, 150, 420, 200), background(&img)) > 500);
}

struct Broken;
impl MessageSource for Broken {
    fn count(&self) -> usize {
        3
    }
    fn messages(&self, _: usize, _: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Err(SourceError::new("store is locked"))
    }
}

#[test]
fn empty_and_unreadable_conversations_say_so() {
    let _g = serial();
    let empty = draw(
        list_of(vec![]).build(400.0, 120.0),
        400,
        120,
        "message-empty.png",
    );
    let broken = draw(
        MessageList::new(Arc::new(Broken)).build(400.0, 120.0),
        400,
        120,
        "message-broken.png",
    );
    let bg = background(&empty);
    assert!(ink(&empty, (0, 0, 400, 120), bg) > 100);
    assert!(ink(&broken, (0, 0, 400, 120), bg) > 300);
    assert_ne!(empty, broken);
}
