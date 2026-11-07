//! A held key never answers, and a disarmed question asks for one redraw, not one per drawing.
//!
//! Tests: the real element in a tree driven like a window drives it, a clock by hand.

mod chat_support;
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chat_support::Manual;
use common::{Input, layout, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Approvals, ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk,
};

fn q(id: &str, command: &str, digest: &[u8]) -> ToolCall {
    ToolCall::new(id, "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Clean up")
        .detail(command)
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
        .revision(1)
        .digest(digest.to_vec())
}

struct Rig {
    clock: Arc<Manual>,
    approvals: Approvals,
    view: ToolCallView,
    count: Arc<AtomicUsize>,
}

fn rig(id: &str) -> Rig {
    let clock = Arc::new(Manual::default());
    let approvals = Approvals::with_clock(clock.clone());
    let count = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&count);
    let view = ToolCallView::new()
        .id(id)
        .approvals(approvals.clone())
        .on_outcome(Arc::new(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        }));
    Rig {
        clock,
        approvals,
        view,
        count,
    }
}

impl Rig {
    fn draw(&self) -> blinc_layout::prelude::RenderTree {
        layout(self.view.build(), 520.0, 900.0)
    }
}

#[test]
fn a_key_held_while_the_question_becomes_armed_does_not_answer_on_release() {
    use blinc_core::events::event_types::{KEY_DOWN, KEY_UP};
    let _g = serial();
    let r = rig("h1");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    let mut tree = r.draw();
    let mut input = Input::new();
    input.focus(&tree, "h1-keys-q1");
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_DOWN, 39); // Right: to Allow once
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_UP, 39);
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_DOWN, 13); // disarmed: counts for nothing
    r.clock.advance(701); // armed while the key is still held
    for _ in 0..5 {
        input.key_event_to(&mut tree, "h1-keys-q1", KEY_DOWN, 13); // the repeats
    }
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_UP, 13);
    assert_eq!(r.count.load(Ordering::Relaxed), 0, "a held key answered");
    // a new press, now that it is armed, answers on its release
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "h1-keys-q1", KEY_UP, 13);
    assert_eq!(r.count.load(Ordering::Relaxed), 1);
}

#[test]
fn a_disarmed_question_asks_for_one_redraw_however_often_it_is_drawn() {
    let _g = serial();
    let r = rig("h2");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    for _ in 0..6 {
        drop(r.draw());
    }
    assert_eq!(r.approvals.redraws_asked(), 1);
    r.clock.advance(701);
    for _ in 0..3 {
        drop(r.draw());
    }
    assert_eq!(
        r.approvals.redraws_asked(),
        1,
        "armed: nothing more to wait for"
    );
}

#[test]
fn a_question_that_time_cannot_arm_asks_for_no_redraw() {
    let _g = serial();
    // an empty digest never matches
    let r = rig("h3");
    r.view.apply(q("q1", "rm ./cache", b""));
    for _ in 0..4 {
        drop(r.draw());
    }
    assert_eq!(r.approvals.redraws_asked(), 0, "empty digest");
    // a long command nobody scrolled to its end
    let long = "echo x; ".repeat(400);
    let r = rig("h4");
    r.view.apply(q("q1", &long, long.as_bytes()));
    for _ in 0..4 {
        drop(r.draw());
    }
    assert_eq!(r.approvals.redraws_asked(), 0, "not read to the end");
}
