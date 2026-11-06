//! The answer guard's rules: refusal re-arms, Refuse is never starved, long commands, digests,
//! tombstones.
//!
//! Pattern: the real element in a tree driven like a window drives it (events with their
//! rectangles, frames), a clock by hand.

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};

use chat_support::Manual;
use common::{Input, layout, serial};
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Applied, Approvals, Outcome, ToolCall, ToolCalls, ToolChoice, ToolEnd, ToolKind, ToolOption,
    ToolRisk, ToolState,
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
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let view = ToolCallView::new()
        .id(id)
        .approvals(Approvals::with_clock(clock.clone()))
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    Rig { clock, view, got }
}

impl Rig {
    fn tree(&self) -> blinc_layout::prelude::RenderTree {
        layout(self.view.build(), 520.0, 900.0)
    }

    /// Show the questions, let the delay pass, draw again.
    fn armed(&self) -> blinc_layout::prelude::RenderTree {
        drop(self.tree());
        self.clock.advance(701);
        self.tree()
    }

    fn answers(&self) -> usize {
        self.got.lock().unwrap().len()
    }
}

#[test]
fn an_engine_refusal_without_change_rearms_after_the_delay() {
    let _g = serial();
    let r = rig("r1");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = r.armed();
    Input::new().click(&mut tree, "r1-opt-q1-once");
    assert_eq!(r.answers(), 1);
    r.view.refused("q1", "the digest no longer fits");
    let mut tree = r.tree();
    Input::new().click(&mut tree, "r1-opt-q1-once");
    assert_eq!(r.answers(), 1, "armed at once after a refusal");
    r.clock.advance(701);
    let mut tree = r.tree();
    Input::new().click(&mut tree, "r1-opt-q1-once");
    assert_eq!(r.answers(), 2, "stuck after a refusal");
}

#[test]
fn refuse_is_never_starved_and_allow_needs_a_digest_and_a_steady_question() {
    let _g = serial();
    let r = rig("f1");
    r.view.apply(q("q1", 1, "rm ./cache"));
    let mut tree = r.tree();
    Input::new().click(&mut tree, "f1-opt-q1-no");
    assert_eq!(r.answers(), 1, "Refuse waited for the delay");
    // a question that changes all the time: Allow never arms
    for rev in 2..6 {
        r.clock.advance(300);
        r.view.apply(q("q1", rev, "rm ./cache"));
        let mut tree = r.tree();
        Input::new().click(&mut tree, "f1-opt-q1-once");
    }
    assert_eq!(r.answers(), 1);
    // the same revision and digest with another command is another question
    let r2 = rig("f2");
    r2.view.apply(q("q1", 1, "rm ./cache"));
    drop(r2.armed());
    let mut changed = q("q1", 1, "rm -rf ~");
    changed.digest = b"rm ./cache".to_vec();
    r2.view.apply(changed);
    let mut tree = r2.tree();
    Input::new().click(&mut tree, "f2-opt-q1-once");
    assert_eq!(
        r2.answers(),
        0,
        "a different command with the same digest was armed"
    );
    // no digest: Allow never answers, Refuse does
    let r3 = rig("f3");
    let mut nodigest = q("q1", 1, "rm x");
    nodigest.digest.clear();
    r3.view.apply(nodigest);
    let mut tree = r3.armed();
    Input::new().click(&mut tree, "f3-opt-q1-once");
    assert_eq!(r3.answers(), 0, "an empty digest was allowed");
    Input::new().click(&mut tree, "f3-opt-q1-no");
    assert_eq!(r3.answers(), 1);
}

#[test]
fn a_command_that_does_not_fit_says_so_and_allow_stays_off_but_refuse_works() {
    let _g = serial();
    let r = rig("l1");
    let command = (1..=41)
        .map(|i| format!("step {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    r.view.apply(q("q1", 1, &command));
    r.view.set_question_height(100.0);
    let mut tree = r.armed();
    assert!(
        tree.query_by_id("l1-more-q1").is_some(),
        "nothing says that more follows"
    );
    let node = tree.query_by_id("l1-opt-q1-once").unwrap();
    let b = tree.get_absolute_bounds(node).unwrap();
    assert!(b.y + b.height <= 900.0);
    Input::new().click(&mut tree, "l1-opt-q1-once");
    assert_eq!(
        r.answers(),
        0,
        "Allow answered without the command having been read"
    );
    Input::new().click(&mut tree, "l1-opt-q1-no");
    assert_eq!(r.answers(), 1, "Refuse was held back");
}

#[test]
fn a_press_needs_a_release_and_the_floor_of_the_delay_holds() {
    assert!(
        Approvals::new().delay_ms(0).delay() >= 300,
        "arming can be switched off"
    );
    let a = Approvals::with_clock(Arc::new(Manual::default()));
    assert!(
        !a.release("p", "m", "c", "b", true),
        "a release without a press answered"
    );
}

#[test]
fn settled_is_final_and_a_lower_revision_never_comes_back() {
    let mut c = ToolCalls::new();
    c.apply(q("a", 9, "x"));
    assert_eq!(c.apply(q("a", 1, "x")), Applied::Stale);
    c.apply(q("b", 1, "y").settled(ToolEnd::Done).revision(2));
    c.replace(vec![]); // the list leaves both out
    assert_eq!(
        c.apply(q("b", 1, "y")),
        Applied::AlreadySettled,
        "a settled call came back"
    );
    assert_eq!(
        c.apply(q("a", 3, "x")),
        Applied::Stale,
        "a stale revision came back after the call left"
    );
    assert_eq!(c.apply(q("a", 10, "x")).clone(), Applied::Added);
    assert!(matches!(
        c.get("a").map(|x| x.state),
        Some(ToolState::Question)
    ));
}
