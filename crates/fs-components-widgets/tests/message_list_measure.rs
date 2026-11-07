//! The message list: learning measured heights, older messages at the front.
//!
//! Tests: layouts of the same view a window draws; the scroll is moved through the handle.

mod chat_support;
mod common;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chat_support::{Generated, Shared, person};
use common::layout;
use fs_components_widgets::message_list::MessageList;

fn shared(n: usize) -> Arc<Shared> {
    Arc::new(Shared(Mutex::new(
        (0..n).map(|i| Generated(n).message(i)).collect(),
    )))
}

#[test]
fn measured_heights_replace_estimates_and_a_message_above_does_not_shift_the_reader() {
    let _g = common::serial();
    let src = shared(200);
    let rows: Arc<Mutex<HashMap<String, f32>>> = Arc::default();
    let seen = Arc::clone(&rows);
    let list = MessageList::new(src.clone())
        .id("h")
        .probe(Arc::new(move |id| seen.lock().unwrap().get(id).copied()));
    let tree = layout(list.build(600.0, 400.0), 600.0, 400.0);
    list.scroll_by(-5000.0);
    let tree2 = layout(list.build(600.0, 400.0), 600.0, 400.0);
    drop(tree);
    // read the real heights out of the layout, as the widget does itself in a window
    let range = list.built();
    for i in range.clone() {
        let id = format!("h-msg-m{i}");
        let node = tree2.query_by_id(&id).expect("built");
        rows.lock()
            .unwrap()
            .insert(id, tree2.get_absolute_bounds(node).unwrap().height);
    }
    let before_total = list.total_height();
    let offset = list.offset();
    let anchor = list.index_at(offset);
    let top_before = list.top_of(anchor);
    let changed = list.measure();
    assert!(changed > 0, "nothing measured");
    assert!(
        (list.total_height() - before_total).abs() > 0.5,
        "the estimates were all exact?"
    );
    // what is above the first message in view changed by exactly as much as the offset moved
    let moved = list.offset() - offset;
    let grew = list.top_of(anchor) - top_before;
    assert!(
        (moved - grew).abs() < 0.01,
        "the reader's place shifted by {moved}, what is above changed by {grew}"
    );
    // a second pass over the same layout changes nothing
    assert_eq!(list.measure(), 0);
}

#[test]
fn older_messages_added_at_the_front_keep_the_row_being_read() {
    let _g = common::serial();
    let src = shared(300);
    let list = MessageList::new(src.clone()).id("p");
    layout(list.build(600.0, 400.0), 600.0, 400.0);
    list.scroll_by(-4000.0);
    let tree = layout(list.build(600.0, 400.0), 600.0, 400.0);
    let key = format!("m{}", list.built().start + 4);
    let y = |t: &blinc_layout::prelude::RenderTree, key: &str| {
        let n = t.query_by_id(&format!("p-msg-{key}")).expect("row built");
        t.get_absolute_bounds(n).unwrap().y
    };
    let before = y(&tree, &key);
    let unseen = list.unseen();
    {
        let mut m = src.0.lock().unwrap();
        let older: Vec<_> = (0..50).map(|i| person(&format!("old{i}"), "Ana")).collect();
        m.splice(0..0, older);
    }
    list.refresh_prepended(50);
    let after = layout(list.build(600.0, 400.0), 600.0, 400.0);
    assert!(
        (y(&after, &key) - before).abs() < 1.0,
        "the row moved from {before} to {}",
        y(&after, &key)
    );
    assert_eq!(list.unseen(), unseen, "older messages were counted as new");
}
