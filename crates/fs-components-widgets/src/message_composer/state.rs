//! The composer's state: the exact text, and where a send stands.
//!
//! Pattern: State — pure, no Blinc. A failed send changes nothing about the text; a successful one
//! clears it only if it is still what was sent, so text typed meanwhile is never lost.

use super::model::{Outcome, Refusal};

/// What the composer holds.
#[derive(Clone, Debug, Default)]
pub struct Composing {
    text: String,
    sending: Option<(u64, String)>,
    attempts: u64,
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
        self.attempts += 1;
        self.sending = Some((self.attempts, self.text.clone()));
        self.error = None;
        Outcome::Send {
            text: self.text.clone(),
            attachments,
            attempt: self.attempts,
        }
    }

    /// The host sent attempt `attempt`. Clears the text if that is the attempt waiting and the text
    /// is still what was sent; returns whether it did. An answer for an older attempt does nothing.
    pub fn sent(&mut self, attempt: u64) -> bool {
        match self.sending.take() {
            Some((n, sent)) if n == attempt => {
                if self.text == sent {
                    self.text.clear();
                    true
                } else {
                    false
                }
            }
            other => {
                self.sending = other;
                false
            }
        }
    }

    /// The host could not send attempt `attempt`: the text stays, the reason is shown. An answer for
    /// an older attempt does nothing.
    pub fn failed(&mut self, attempt: u64, reason: &str) {
        if self.sending.as_ref().is_some_and(|(n, _)| *n == attempt) {
            self.sending = None;
            self.error = Some(reason.to_owned());
        }
    }

    /// The number of the attempt waiting for its answer.
    pub fn waiting(&self) -> Option<u64> {
        self.sending.as_ref().map(|(n, _)| *n)
    }
}
