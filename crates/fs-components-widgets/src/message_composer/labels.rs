//! The words of the composer. They come from the caller; English by default.
//!
//! Pattern: Value object.

/// Every word the composer draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Placeholder of the text area.
    pub placeholder: String,
    /// Button: send.
    pub send: String,
    /// Button label while a send waits for its answer.
    pub sending: String,
    /// Button: attach.
    pub attach: String,
    /// Button: remove an attachment.
    pub remove: String,
    /// Not sent; `{}` = the reason. The text is still here.
    pub failed: String,
    /// A saved draft was put back.
    pub restored: String,
    /// The draft could not be saved; `{}` = the reason.
    pub draft_failed: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            placeholder: s("Write a message"),
            send: s("Send"),
            sending: s("Sending"),
            attach: s("Attach"),
            remove: s("Remove"),
            failed: s("Not sent: {}. Your text is still here."),
            restored: s("Draft restored."),
            draft_failed: s("Draft could not be saved: {}"),
        }
    }
}
