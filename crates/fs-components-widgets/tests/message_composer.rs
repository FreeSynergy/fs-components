//! The composer: exact text, send as an outcome, nothing lost, attachments, drafts.
//!
//! Pattern: state tests, pixel tests, an in-process mouse. Typing itself (and the input method)
//! needs a real window; `typed` is what the text area calls.

mod common;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use blinc_layout::widgets::text_area::text_area_state;
use common::{Input, draw, layout, serial};
use fs_components_widgets::drafts::{Draft, DraftStore};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_widgets::message_composer::{
    Attachment, Composing, MessageComposer, Outcome, Refusal,
};
use fs_components_widgets::source_error::{ErrorKind, SourceError};

fn sent_text(o: &Outcome) -> &str {
    match o {
        Outcome::Send { text, .. } => text,
        other => panic!("not a send: {other:?}"),
    }
}

#[test]
fn the_text_goes_out_exactly_and_stays_until_the_host_says_it_was_sent() {
    let exact = "  line one\r\n\tline two  \n\n\u{1F600} \u{e9}\u{301}\n  ";
    let mut c = Composing::new();
    c.set_text(exact);
    let out = c.send(vec![]);
    assert_eq!(sent_text(&out), exact);
    assert_eq!(c.text(), exact, "sending must not clear anything");
    assert!(c.is_sending());
    assert_eq!(c.send(vec![]), Outcome::Refused(Refusal::Busy));
    c.failed(1, "no network");
    assert_eq!(c.text(), exact, "a failed send lost text");
    assert_eq!(c.error(), Some("no network"));
    assert!(!c.is_sending());
    // again, and this time it works
    assert_eq!(sent_text(&c.send(vec![])), exact);
    assert!(c.sent(2));
    assert_eq!(c.text(), "");
}

#[test]
fn a_late_answer_of_an_older_attempt_changes_nothing() {
    let mut c = Composing::new();
    c.set_text("again");
    c.send(vec![]);
    c.failed(1, "slow");
    c.send(vec![]);
    assert!(
        !c.sent(1),
        "a late sent() of attempt 1 cleared attempt 2's text"
    );
    assert_eq!(c.text(), "again");
    assert!(c.is_sending(), "attempt 2 is still waiting");
    c.failed(1, "late failure");
    assert!(c.is_sending() && c.error().is_none());
    assert!(c.sent(2));
}

#[test]
fn text_typed_while_sending_is_never_cleared() {
    let mut c = Composing::new();
    c.set_text("first");
    c.send(vec![]);
    c.set_text("first, and more");
    assert!(!c.sent(1));
    assert_eq!(c.text(), "first, and more");
}

#[test]
fn nothing_or_white_space_is_not_sent() {
    let mut c = Composing::new();
    assert_eq!(c.send(vec![]), Outcome::Refused(Refusal::Empty));
    c.set_text(" \n\t ");
    assert_eq!(c.send(vec![]), Outcome::Refused(Refusal::Empty));
    assert!(!c.is_sending());
}

#[test]
fn what_the_text_area_does_to_text_is_known() {
    // The composer holds its own copy of what the area reported; this records what the area says.
    let area = text_area_state();
    for text in ["a\nb", "a\n", "a\r\nb", "a  ", "\u{1F600}x"] {
        area.lock().unwrap().set_value(text);
        let back = area.lock().unwrap().value();
        println!("area: {text:?} -> {back:?}");
    }
}

#[test]
fn the_send_button_reports_the_text_and_a_failure_is_shown_with_the_text_kept() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let c = MessageComposer::new("room")
        .id("c1")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    c.set_text("Hello\nworld ");
    let mut tree = layout(c.build(), 520.0, 260.0);
    assert!(got.lock().unwrap().is_empty());
    Input::new().click(&mut tree, "c1-send");
    let seen = got.lock().unwrap().clone();
    assert_eq!(sent_text(&seen[0]), "Hello\nworld ");
    assert!(c.is_sending());
    let busy = draw(c.build(), 520, 260, "composer-sending.png");
    c.failed("server unreachable");
    assert_eq!(c.text(), "Hello\nworld ");
    let failed = draw(c.build(), 520, 260, "composer-failed.png");
    assert_ne!(busy, failed);
    let red = failed
        .pixels()
        .filter(|p| i32::from(p.0[0]) > i32::from(p.0[1]) + 5 && p.0[0] > 200)
        .count();
    assert!(red > 300, "the failure is not visible: {red}");
    // the send button works again
    let mut tree = layout(c.build(), 520.0, 260.0);
    Input::new().click(&mut tree, "c1-send");
    assert_eq!(got.lock().unwrap().len(), 2);
    c.sent();
    assert_eq!(c.text(), "");
}

#[test]
fn attachments_are_the_hosts_list_and_removal_is_an_outcome() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let c = MessageComposer::new("room")
        .id("c2")
        .attach_button(true)
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    c.set_text("see files");
    c.set_attachments(vec![
        Attachment::new("f1", "plan.pdf", "1.2 MB"),
        Attachment::new("f2", "photo.png", ""),
    ]);
    let mut tree = layout(c.build(), 520.0, 260.0);
    draw(c.build(), 520, 260, "composer-attachments.png");
    let mut input = Input::new();
    input.click(&mut tree, "c2-remove-f1");
    input.click(&mut tree, "c2-attach");
    input.click(&mut tree, "c2-send");
    let seen = got.lock().unwrap().clone();
    assert_eq!(seen[0], Outcome::RemoveAttachment { key: "f1".into() });
    assert_eq!(seen[1], Outcome::AttachRequested);
    assert_eq!(
        seen[2],
        Outcome::Send {
            text: "see files".into(),
            attachments: vec!["f1".into(), "f2".into()],
            attempt: 1
        }
    );
    assert_eq!(
        c.attachments().len(),
        2,
        "the composer must not drop the host's list itself"
    );
}

#[test]
fn ctrl_enter_sends_only_when_the_frontend_binds_it() {
    let plain = MessageComposer::new("r");
    plain.set_text("x");
    let mut ctrl = KeyEvent::new(Key::Enter);
    ctrl.ctrl = true;
    assert!(plain.press(&ctrl).is_none());
    let bound = MessageComposer::new("r").send_key(Key::Enter);
    bound.set_text("x");
    assert!(bound.press(&KeyEvent::new(Key::Enter)).is_none());
    assert!(matches!(bound.press(&ctrl), Some(Outcome::Send { .. })));
}

#[derive(Default)]
struct Store {
    drafts: Mutex<HashMap<String, String>>,
    fail: Mutex<bool>,
}

impl DraftStore for Store {
    fn save(&self, draft: &Draft) -> Result<(), SourceError> {
        if *self.fail.lock().unwrap() {
            return Err(SourceError::new(ErrorKind::Failed, "disk full"));
        }
        self.drafts
            .lock()
            .unwrap()
            .insert(draft.path.clone(), draft.text.clone());
        Ok(())
    }

    fn load(&self, path: &str) -> Result<Option<Draft>, SourceError> {
        Ok(self
            .drafts
            .lock()
            .unwrap()
            .get(path)
            .map(|t| Draft::new(path, t.as_str(), "", None)))
    }

    fn clear(&self, path: &str) -> Result<(), SourceError> {
        self.drafts.lock().unwrap().remove(path);
        Ok(())
    }
}

#[test]
fn the_draft_is_kept_restored_and_removed_only_after_a_real_send() {
    let _g = serial();
    let store = Arc::new(Store::default());
    let c = MessageComposer::new("room-1").drafts(store.clone());
    c.typed("half a thou");
    c.save_draft_now();
    assert_eq!(
        store.drafts.lock().unwrap().get("room-1").unwrap(),
        "half a thou"
    );
    // a new composer finds it
    let again = MessageComposer::new("room-1").drafts(store.clone());
    assert_eq!(again.text(), "half a thou");
    // another conversation does not
    assert_eq!(
        MessageComposer::new("room-2").drafts(store.clone()).text(),
        ""
    );
    // send, fail: draft stays; send, succeed: draft goes
    again.send();
    again.failed("offline");
    assert!(store.drafts.lock().unwrap().contains_key("room-1"));
    again.send();
    again.sent();
    assert!(!store.drafts.lock().unwrap().contains_key("room-1"));
}

#[test]
fn a_draft_that_cannot_be_written_is_shown() {
    let _g = serial();
    let store = Arc::new(Store::default());
    *store.fail.lock().unwrap() = true;
    let c = MessageComposer::new("r").id("c3").drafts(store);
    c.typed("text");
    c.save_draft_now();
    let ok = MessageComposer::new("r").id("c3");
    ok.set_text("text");
    let shown = draw(c.build(), 520, 260, "composer-draft-failed.png");
    let plain = draw(ok.build(), 520, 260, "composer-plain.png");
    assert_ne!(shown, plain, "the failed draft is not shown");
}

#[test]
fn the_compact_form_is_smaller() {
    let _g = serial();
    let big = layout(MessageComposer::new("r").id("a").build(), 520.0, 400.0);
    let small = layout(
        MessageComposer::new("r").id("b").compact(true).build(),
        520.0,
        400.0,
    );
    let h = |t: &blinc_layout::prelude::RenderTree, id: &str| {
        t.get_absolute_bounds(t.query_by_id(id).unwrap())
            .unwrap()
            .height
    };
    assert!(
        h(&small, "b-send") < h(&big, "a-send"),
        "{} vs {}",
        h(&small, "b-send"),
        h(&big, "a-send")
    );
    draw(
        MessageComposer::new("r").id("b").compact(true).build(),
        520,
        120,
        "composer-compact.png",
    );
}

#[test]
fn many_attachments_do_not_push_send_out_of_reach() {
    let _g = serial();
    let c = MessageComposer::new("r").id("ma");
    c.set_attachments(
        (0..24)
            .map(|i| Attachment::new(format!("f{i}"), format!("file-{i}.pdf"), "1 MB"))
            .collect(),
    );
    let tree = layout(c.build(), 500.0, 360.0);
    let n = tree.query_by_id("ma-send").expect("send");
    let b = tree.get_absolute_bounds(n).unwrap();
    assert!(b.y + b.height <= 360.0, "Send is out of reach at y={}", b.y);
}
