//! What the guard keeps of a drawn question.
//!
//! Pattern: none.

use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::SharedScrollPhysics;

use super::model::ToolCall;

pub(super) type Call = (String, String);
pub(super) type EntryKey = (String, String, String);

/// The scroll position of a question's text window.
#[derive(Clone)]
pub(crate) struct Body {
    pub scroll: ScrollRef,
    pub physics: SharedScrollPhysics,
}

pub(super) struct Entry {
    pub(super) generation: u64,
    pub(super) revision: u64,
    pub(super) digest: Vec<u8>,
    pub(super) text: String,
    pub(super) slot: usize,
    pub(super) at: u64,
    pub(super) needs_read: bool,
    pub(super) body: Body,
    pub(super) cursor: usize,
}

pub(super) struct Press {
    pub(super) button: String,
    pub(super) generation: u64,
}

pub(super) fn call_key(message: &str, call: &str) -> Call {
    (message.to_owned(), call.to_owned())
}

pub(super) fn entry_key(place: &str, message: &str, call: &str) -> EntryKey {
    (place.to_owned(), message.to_owned(), call.to_owned())
}

/// What makes two drawings of a question the same question to a person.
pub(super) fn text_of(call: &ToolCall) -> String {
    let options: Vec<String> = call
        .options
        .iter()
        .map(|o| format!("{}={}", o.id, o.label))
        .collect();
    format!(
        "{}\u{1}{}\u{1}{}\u{1}{}",
        call.name,
        call.summary,
        call.detail,
        options.join("\u{2}")
    )
}
