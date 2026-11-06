//! The composer's state: the exact text, and where a send stands.
//!
//! Pattern: State — pure, no Blinc. A failed send changes nothing about the text; a successful one
//! clears it only if it is still what was sent, so text typed meanwhile is never lost.

use super::model::{Outcome, Refusal};

/// What the composer holds.
#[derive(Clone, Debug, Default)]
pub struct Composing {
    text: String,
    sending: Option<String>,
    error: Option<String>,
}

impl Composing {
    /// Nothing typed.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text, exactly.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replace the text (typing, restoring a draft).
    pub fn set_text(&mut self, text: &str) {
        text.clone_into(&mut self.text);
    }

    /// Is a send waiting for its answer?
    pub fn is_sending(&self) -> bool {
        self.sending.is_some()
    }

    /// Why the last send failed, if it did.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Ask to send. The text stays where it is.
    pub fn send(&mut self, attachments: Vec<String>) -> Outcome {
        if self.sending.is_some() {
            return Outcome::Refused(Refusal::Busy);
        }
        if self.text.trim().is_empty() {
            return Outcome::Refused(Refusal::Empty);
        }
        self.sending = Some(self.text.clone());
        self.error = None;
        Outcome::Send {
            text: self.text.clone(),
            attachments,
        }
    }

    /// The host sent it. Clears the text if it is still what was sent; returns whether it did.
    pub fn sent(&mut self) -> bool {
        let Some(sent) = self.sending.take() else {
            return false;
        };
        if self.text == sent {
            self.text.clear();
            true
        } else {
            false
        }
    }

    /// The host could not send it: the text stays, the reason is shown.
    pub fn failed(&mut self, reason: &str) {
        self.sending = None;
        self.error = Some(reason.to_owned());
    }
}
