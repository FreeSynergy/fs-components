//! The words of the name dialog.
//!
//! Pattern: Adapter — the texts are asked of a `Words` by identifier (ADR 0057), so the language is the caller's.

use fs_components_words::{Args, TextId, Words};

use super::texts::{CANCEL, NEW_PAGE, OK, PLACEHOLDER, RENAME, SUB_PAGE};

/// The texts of the dialog; `{}` in a title is replaced by what the dialog is about (the parent page, the old name).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Title for a new page.
    pub new_page: String,
    /// Title for a sub-page; `{}` = the parent.
    pub sub_page: String,
    /// Title for a new name; `{}` = the old name.
    pub rename: String,
    /// Shown while the field is empty.
    pub placeholder: String,
    /// Button: take the name.
    pub ok: String,
    /// Button: leave.
    pub cancel: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            new_page: s("New page"),
            sub_page: s("New sub-page of {}"),
            rename: s("Rename {}"),
            placeholder: s("Name"),
            ok: s("OK"),
            cancel: s("Cancel"),
        }
    }
}

impl Labels {
    /// The texts of `words` (a title with `{}` where the thing it talks about goes, filled by the dialog).
    pub fn from_words(words: &dyn Words) -> Self {
        let say = |id: &str, args: &Args| words.say(&TextId::new(id), args).text().to_owned();
        let hole = |id: &str| say(id, &Args::new().text("about", "{}"));
        Self {
            new_page: say(NEW_PAGE, &Args::new()),
            sub_page: hole(SUB_PAGE),
            rename: hole(RENAME),
            placeholder: say(PLACEHOLDER, &Args::new()),
            ok: say(OK, &Args::new()),
            cancel: say(CANCEL, &Args::new()),
        }
    }
}
