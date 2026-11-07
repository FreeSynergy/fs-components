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
    let _g = serial();
    let r = rig("h1");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    let mut tree = r.draw();
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "h1-keys-q1"));
    input.key(&mut tree, 39); // Right: to Allow once
    input.key_release(&mut tree, 39);
    input.key(&mut tree, 13); // disarmed: counts for nothing
    r.clock.advance(701); // armed while the key is still held
    for _ in 0..5 {
        input.key(&mut tree, 13); // the repeats
    }
    input.key_release(&mut tree, 13);
    assert_eq!(r.count.load(Ordering::Relaxed), 0, "a held key answered");
    // a new press, now that it is armed, answers on its release
    input.key(&mut tree, 13);
    input.key_release(&mut tree, 13);
    assert_eq!(r.count.load(Ordering::Relaxed), 1);
}

#[test]
fn a_redraw_between_the_press_and_its_repeat_keeps_the_key_held() {
    let _g = serial();
    let r = rig("h5");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    let mut tree = r.draw();
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "h5-keys-q1"));
    input.key(&mut tree, 39); // Right: to Allow once
    input.key_release(&mut tree, 39);
    input.key(&mut tree, 13); // disarmed: counts for nothing, but the key is down
    r.clock.advance(741);
    let mut tree = r.draw(); // the redraw the guard asked for: the question is armed now
    for _ in 0..5 {
        input.key(&mut tree, 13); // the repeats
    }
    input.key_release(&mut tree, 13);
    assert_eq!(
        r.count.load(Ordering::Relaxed),
        0,
        "a held key answered after a redraw"
    );
    input.key(&mut tree, 13);
    input.key_release(&mut tree, 13);
    assert_eq!(
        r.count.load(Ordering::Relaxed),
        1,
        "a new press did not answer"
    );
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

#[test]
fn scrolling_a_long_command_to_its_end_asks_for_exactly_one_redraw() {
    let _g = serial();
    let long = "echo x; ".repeat(400);
    let r = rig("h6");
    r.view.apply(q("q1", &long, long.as_bytes()));
    r.clock.advance(701);
    let mut tree = r.draw();
    // just inside the top left of the text, which is far taller than its window
    let b = tree
        .get_absolute_bounds(tree.query_by_id("h6-detail-q1").unwrap())
        .unwrap();
    let (x, y) = (b.x + 20.0, b.y + 5.0);
    let mut input = Input::new();
    // a little way: the end is not reached, nothing to draw
    input.wheel_at(&mut tree, x, y, 40.0);
    assert_eq!(
        r.approvals.redraws_asked(),
        0,
        "a scroll that did not reach the end redrew"
    );
    // the whole way, in many small turns of the wheel
    for _ in 0..200 {
        input.wheel_at(&mut tree, x, y, 60.0);
    }
    assert_eq!(
        r.approvals.redraws_asked(),
        1,
        "reaching the end must draw once"
    );
    for _ in 0..5 {
        input.wheel_at(&mut tree, x, y, 60.0);
    }
    assert_eq!(
        r.approvals.redraws_asked(),
        1,
        "at the end nothing more to draw"
    );
}
