//! An answer button is no tab stop and takes no key; the watch on the end of a long command belongs
//! to the question, not to the build that made it.
//!
//! Tests: the real element in a tree driven like a window drives it, a clock by hand.

mod approvals_rig;
mod chat_support;
mod common;

use std::sync::atomic::Ordering;

use approvals_rig::{q, rig};
use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use common::{Input, serial};

#[test]
fn an_answer_button_is_no_tab_stop() {
    let _g = serial();
    let r = rig("h7");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    let mut tree = r.draw();
    let mut input = Input::new();
    assert!(
        !input.tab_to(&mut tree, "h7-opt-q1-once"),
        "a button took a tab stop that belongs to the row"
    );
    assert!(!input.tab_to(&mut tree, "h7-opt-q1-no"));
}

#[test]
fn a_key_held_on_a_focused_button_never_answers() {
    let _g = serial();
    for redraw in [false, true] {
        let r = rig("h8");
        r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
        let mut tree = r.draw();
        let mut input = Input::new();
        input.focus(&tree, "h8-opt-q1-once");
        input.key(&mut tree, 13); // disarmed: counts for nothing, but the key is down
        r.clock.advance(741);
        if redraw {
            tree = r.draw();
            input.focus(&tree, "h8-opt-q1-once");
        }
        for _ in 0..5 {
            input.key(&mut tree, 13); // the repeats
        }
        input.key_release(&mut tree, 13);
        assert_eq!(
            r.count.load(Ordering::Relaxed),
            0,
            "a held key answered on a button (redraw: {redraw})"
        );
    }
}

#[test]
fn a_click_on_a_button_while_the_row_holds_a_key_is_a_repeat_and_begins_no_press() {
    let _g = serial();
    let r = rig("h9");
    r.view.apply(q("q1", "rm ./cache", b"rm ./cache"));
    let mut tree = r.draw();
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "h9-keys-q1"));
    input.key(&mut tree, 13); // disarmed, the key is down
    r.clock.advance(741);
    let (x, y) = Input::centre(&tree, "h9-opt-q1-once");
    input.press_at(&mut tree, x, y);
    input.release_at(&mut tree, x, y);
    assert_eq!(r.count.load(Ordering::Relaxed), 0, "a repeat began a press");
}

#[test]
fn builds_given_back_by_a_window_do_not_take_the_end_of_a_long_command_with_them() {
    let _g = serial();
    let long = "echo x; ".repeat(400);
    let r = rig("h10");
    r.view.apply(q("q1", &long, long.as_bytes()));
    r.clock.advance(701);
    let mut scope = BuildScope::new();
    let mut tree = None;
    for _ in 0..2 {
        let pass = scope.begin();
        tree = Some(r.draw());
        scope.finish(pass); // a resize: what the builds before made is given back
    }
    let mut tree = tree.unwrap();
    let b = tree
        .get_absolute_bounds(tree.query_by_id("h10-detail-q1").unwrap())
        .unwrap();
    let (x, y) = (b.x + 20.0, b.y + 5.0);
    let mut input = Input::new();
    for _ in 0..200 {
        input.wheel_at(&mut tree, x, y, 60.0);
    }
    assert_eq!(
        r.approvals.redraws_asked(),
        1,
        "the end was reached and nothing asked to draw"
    );
    r.clock.advance(741);
    let mut tree = r.draw();
    input.click(&mut tree, "h10-opt-q1-once");
    assert_eq!(r.count.load(Ordering::Relaxed), 1, "Allow stayed disarmed");
}

#[test]
fn the_watch_of_a_question_goes_with_the_question() {
    let _g = serial();
    let long = "echo x; ".repeat(400);
    let r = rig("h11");
    let effects = || global_graph().lock().unwrap().stats().effect_count;
    let before = effects();
    r.view.apply(q("q1", &long, long.as_bytes()));
    drop(r.draw());
    drop(r.draw());
    assert_eq!(effects(), before + 1, "one watch, however often drawn");
    r.approvals.forget("", "q1");
    assert_eq!(effects(), before, "the watch outlived its question");
}
