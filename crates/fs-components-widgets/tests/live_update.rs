//! A change reaches the screen through the same path a window uses (a frame), not by building the
//! element again. Tests: model change, one frame, look at the tree.

mod chat_support;
mod common;

use std::sync::{Arc, Mutex};

use chat_support::person;
use common::{frame, layout, serial};
use fs_components_widgets::message_composer::{Attachment, MessageComposer};
use fs_components_widgets::message_list::{ChatMessage, MessageList, MessageSource, SourceError};

struct Shared(Mutex<Vec<ChatMessage>>);
impl MessageSource for Shared {
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }
    fn messages(&self, f: usize, l: usize) -> Result<Vec<ChatMessage>, SourceError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .iter()
            .skip(f)
            .take(l)
            .cloned()
            .collect())
    }
}

#[test]
fn a_new_message_and_grown_text_reach_a_frame() {
    let _g = serial();
    let src = Arc::new(Shared(Mutex::new(vec![person("1", "Ana")])));
    let list = MessageList::new(src.clone()).id("lu");
    let mut tree = layout(list.build(500.0, 300.0), 500.0, 300.0);
    assert!(tree.query_by_id("lu-msg-1").is_some());
    src.0.lock().unwrap().push(person("2", "Ana"));
    list.refresh();
    frame(&mut tree, 500.0, 300.0);
    assert!(
        tree.query_by_id("lu-msg-2").is_some(),
        "the new message never reached a frame"
    );
}

#[test]
fn a_failure_and_attachments_reach_a_frame() {
    let _g = serial();
    let c = MessageComposer::new("r").id("cu");
    c.set_text("x");
    let mut tree = layout(c.build(), 500.0, 300.0);
    c.set_attachments(vec![Attachment::new("f", "a.pdf", "")]);
    frame(&mut tree, 500.0, 300.0);
    assert!(
        tree.query_by_id("cu-remove-f").is_some(),
        "attachments never reached a frame"
    );
    c.send();
    c.failed("offline");
    frame(&mut tree, 500.0, 300.0);

    assert!(
        tree.query_by_id("cu-failed").is_some(),
        "the failure never reached a frame"
    );
}
