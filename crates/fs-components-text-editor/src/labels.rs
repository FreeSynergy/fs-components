//! The words of the editor. They come from the caller; English by default.
//!
//! Pattern: none — plain data (ADR 0057: no sentence inline in code).

/// Every word the editor draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Button: bold.
    pub bold: String,
    /// Button: italic.
    pub italic: String,
    /// Button: heading; `{level}` = 1 to 3.
    pub heading: String,
    /// Button: list point.
    pub list: String,
    /// Button: link.
    pub link: String,
    /// Button: code.
    pub code: String,
    /// Button: quotation.
    pub quote: String,
    /// Button: undo.
    pub undo: String,
    /// Button: redo.
    pub redo: String,
    /// Placeholder of the empty text area.
    pub placeholder: String,
    /// The counters; `{chars}` and `{words}` are numbers.
    pub counts: String,
    /// Shown while there are changes that are not saved.
    pub unsaved: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            bold: s("Bold"),
            italic: s("Italic"),
            heading: s("H{level}"),
            list: s("List"),
            link: s("Link"),
            code: s("Code"),
            quote: s("Quote"),
            undo: s("Undo"),
            redo: s("Redo"),
            placeholder: s("Write Markdown"),
            counts: s("{chars} characters, {words} words"),
            unsaved: s("Unsaved changes"),
        }
    }
}
