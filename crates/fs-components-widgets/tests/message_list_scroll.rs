//! The message list over long conversations: virtualisation, following, streaming, learning heights.
//!
//! Pattern: model tests and layouts of the same view a window draws; the scroll is moved through
//! the handle (the wheel itself needs a real window).

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use chat_support::{Generated, msg, person};
use common::{draw, layout, serial};
use fs_components_widgets::message_list::{
    ChatMessage, Heights, MessageList, MessageSource, SourceError,
};
use fs_components_widgets::{Sender, SenderKind};

#[test]
fn heights_agree_with_a_plain_sum() {
    let mut h = Heights::new(1000, 50.0);
    let mut plain = vec![50.0f32; 1000];
    let mut seed = 12345u64;
    for _ in 0..3000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let i = (seed >> 33) as usize % 1000;
        let v = 20.0 + ((seed >> 20) % 300) as f32;
        h.set(i, v, true);
        plain[i] = v;
    }
    h.grow(1200);
    plain.resize(1200, 50.0);
    for i in [0usize, 1, 7, 500, 999, 1000, 1199, 1200] {
        let want: f32 = plain[..i].iter().sum();
        assert!(
            (h.top(i) - want).abs() < 0.01,
            "top({i}) = {} want {want}",
            h.top(i)
        );
    }
    for y in [0.0f32, 49.0, 50.0, 12345.6, 40000.0, 1e9] {
        let mut acc = 0.0;
        let mut want = plain.len() - 1;
        for (i, v) in plain.iter().enumerate() {
            if acc + v > y {
                want = i;
                break;
            }
            acc += v;
        }
        assert_eq!(h.at(y), want, "at({y})");
    }
}

#[test]
fn a_measured_height_is_not_replaced_by_an_estimate() {
    let mut h = Heights::new(3, 50.0);
    h.set(1, 80.0, true);
    assert_eq!(h.set(1, 60.0, false), 0.0);
    assert!((h.get(1) - 80.0).abs() < f32::EPSILON);
    h.reestimate(1, 60.0);
    assert!((h.get(1) - 60.0).abs() < f32::EPSILON);
}

#[test]
fn a_hundred_thousand_messages_build_only_what_is_near_the_view() {
    let _g = serial();
    let t = Instant::now();
    let list = MessageList::new(Arc::new(Generated(100_000))).id("big");
    let made = t.elapsed();
    let t = Instant::now();
    let tree = layout(list.build(640.0, 480.0), 640.0, 480.0);
    let first = t.elapsed();
    let at_end = list.built();
    assert!(
        at_end.end == 100_000 && at_end.len() < 40,
        "built {at_end:?}"
    );
    assert!(tree.query_by_id("big-msg-m99999").is_some());
    assert!(tree.query_by_id("big-msg-m50000").is_none());
    assert!(list.is_following());
    // the way up: to the middle
    let middle = list.total_height() / 2.0;
    let t = Instant::now();
    list.scroll_by(-(list.total_height() - middle));
    let tree = layout(list.build(640.0, 480.0), 640.0, 480.0);
    let jump = t.elapsed();
    let built = list.built();
    assert!(!list.is_following());
    assert!(
        built.len() < 40 && built.start > 20_000 && built.end < 80_000,
        "{built:?}"
    );
    let key = format!("big-msg-m{}", built.start + 5);
    assert!(tree.query_by_id(&key).is_some(), "{key} missing");
    // and a picture of it
    let t2 = Instant::now();
    draw(
        list.build(640.0, 480.0),
        640,
        480,
        "message-list-100000-middle.png",
    );
    let drawn = t2.elapsed();
    println!(
        "100000 messages (debug build): list made {made:?}, first build+layout {first:?}, jump to the middle+build+layout {jump:?}, build+layout+draw {drawn:?}; built {} messages, total height {:.0} px",
        built.len(),
        list.total_height()
    );
}

struct Shared(Mutex<Vec<ChatMessage>>);

impl MessageSource for Shared {
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    fn messages(&self, first: usize, limit: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .skip(first)
            .take(limit)
            .cloned()
            .collect())
    }
}

fn shared(n: usize) -> Arc<Shared> {
    Arc::new(Shared(Mutex::new(
        (0..n).map(|i| Generated(n).message(i)).collect(),
    )))
}

fn push(s: &Shared, text: &str) {
    let n = s.0.lock().unwrap().len();
    let a = Sender::new("a", "Helper").of_kind(SenderKind::Ai);
    s.0.lock().unwrap().push(msg(&format!("n{n}"), a, text));
}

#[test]
fn it_follows_at_the_bottom_and_keeps_the_place_when_reading_older_messages() {
    let _g = serial();
    let src = shared(300);
    let list = MessageList::new(src.clone()).id("f");
    layout(list.build(600.0, 400.0), 600.0, 400.0);
    let bottom = list.offset();
    assert!(list.is_following() && bottom > 1000.0);
    // a new message at the bottom: followed
    push(&src, "fresh");
    list.refresh();
    assert_eq!(list.unseen(), 0);
    layout(list.build(600.0, 400.0), 600.0, 400.0);
    assert!(
        list.offset() > bottom,
        "did not follow: {} <= {bottom}",
        list.offset()
    );
    // the reader goes up
    list.scroll_by(-3000.0);
    let reading = list.offset();
    assert!(!list.is_following());
    // messages arrive: the place does not move, they are counted
    push(&src, "one");
    push(&src, "two");
    list.refresh();
    assert_eq!(list.unseen(), 2);
    layout(list.build(600.0, 400.0), 600.0, 400.0);
    assert!(
        (list.offset() - reading).abs() < 0.5,
        "moved from {reading} to {}",
        list.offset()
    );
    assert!(!list.is_following());
    // the indicator is drawn, and jumping to the end follows again
    let tree = layout(list.build(600.0, 400.0), 600.0, 400.0);
    assert!(
        tree.query_by_id("f-new").is_some(),
        "no new-message indicator"
    );
    list.jump_to_end();
    let tree = layout(list.build(600.0, 400.0), 600.0, 400.0);
    assert!(list.is_following() && list.unseen() == 0);
    assert!(tree.query_by_id("f-new").is_none());
    assert!(tree.query_by_id("f-msg-n302").is_some() || tree.query_by_id("f-msg-n301").is_some());
}

#[test]
fn text_that_grows_stays_in_place_and_only_the_bottom_follows() {
    let _g = serial();
    let src = shared(100);
    let a = Sender::new("a", "Helper").of_kind(SenderKind::Ai);
    src.0
        .lock()
        .unwrap()
        .push(msg("live", a.clone(), "Starting").streaming(true));
    let list = MessageList::new(src.clone()).id("g");
    layout(list.build(500.0, 360.0), 500.0, 360.0);
    let total = list.total_height();
    let long = "growing words ".repeat(120);
    let grow = |rev: u64| {
        let mut m = src.0.lock().unwrap();
        *m.last_mut().unwrap() = msg("live", a.clone(), &long).streaming(true).revision(rev);
    };
    grow(1);
    list.refresh();
    layout(list.build(500.0, 360.0), 500.0, 360.0);
    assert!(
        list.total_height() > total + 200.0,
        "growth not seen: {} vs {total}",
        list.total_height()
    );
    assert!(list.is_following());
    assert!(
        list.offset() > list.total_height() - 360.0 - 5.0,
        "not at the bottom"
    );
    // reading older text: the growth below does not move the view
    list.scroll_by(-1500.0);
    let reading = list.offset();
    grow(2);
    list.refresh();
    layout(list.build(500.0, 360.0), 500.0, 360.0);
    assert!((list.offset() - reading).abs() < 0.5);
    assert!(!list.is_following());
}

#[test]
fn a_narrow_list_wraps_text_inside_the_box_and_keeps_the_marks() {
    let _g = serial();
    let list = MessageList::new(Arc::new(Generated(50))).id("c");
    let img = draw(
        list.build(300.0, 400.0),
        300,
        400,
        "message-list-narrow.png",
    );
    let (_, right) = common::extent_x(&img, 0, 400).expect("ink");
    assert!(right < 300, "text ran out of the box: {right}");
    let _ = person("x", "y");
}
