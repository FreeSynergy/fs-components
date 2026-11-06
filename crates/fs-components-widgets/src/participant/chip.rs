//! The mark itself.
//!
//! Pattern: Adapter — kind and words in, a small Blinc box out. Every kind gets a mark, a person
//! too, so the absence of a mark never means "person".

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::model::{KindLabels, SenderKind};

/// The mark for `kind`, with its word.
pub(crate) fn chip(kind: SenderKind, labels: &KindLabels) -> Div {
    let class = match kind {
        SenderKind::Person => "fs-pk-person",
        SenderKind::Software => "fs-pk-software",
        SenderKind::Ai => "fs-pk-ai",
        _ => "fs-pk-unknown",
    };
    div().class("fs-pk").class(class).child(
        text(labels.word(kind).to_string())
            .class("fs-pk-text")
            .no_wrap(),
    )
}
