//! What a wiki page knows about itself beyond its text: the languages it is written in, whether the
//! translation is out of date, which revisions it has. A source that has no such things says nothing
//! (the defaults), and the wiki layouts leave those parts out.
//!
//! Pattern: Value objects with builder-style setters; the source fills them (Strategy), the layouts only read.

/// One language of a page, for the language switch.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LangChoice {
    /// The code (`de`, `en`).
    pub code: String,
    /// The name in its own language (`Deutsch`).
    pub name: String,
    /// What the reader follows to get there (a link target of the source).
    pub href: String,
    /// Is this the language shown now?
    pub current: bool,
}

impl LangChoice {
    /// A language.
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        href: impl Into<String>,
        current: bool,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            href: href.into(),
            current,
        }
    }
}

/// How current the text shown is.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageState {
    /// Nothing to say (or nothing known).
    #[default]
    Current,
    /// The translation is older than its original.
    Outdated,
    /// The page does not exist in the language asked for; another one is shown.
    Fallback,
}

/// One revision of a page, a row of the history.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisionRow {
    /// The name of the revision (a number or a time).
    pub label: String,
    /// What it says about itself (who, what), may be empty.
    pub note: String,
}

impl RevisionRow {
    /// A revision row.
    pub fn new(label: impl Into<String>, note: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            note: note.into(),
        }
    }
}

/// What a source says about one page.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageInfo {
    /// The path the page has in the tree (differs from the shown path when that names a language).
    pub canonical: Option<String>,
    /// The languages the page has; fewer than two = no switch.
    pub langs: Vec<LangChoice>,
    /// How current the text is.
    pub state: PageState,
    /// The sentence that explains the state, if there is one.
    pub notice: Option<String>,
    /// When it was last changed, in the words of the source.
    pub updated: Option<String>,
    /// The revisions, newest first.
    pub revisions: Vec<RevisionRow>,
}

/// The words of the wiki layouts (the caller's, like [`Labels`](crate::Labels)).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiLabels {
    /// Name of the program, top left.
    pub program: String,
    /// Heading of the page tree.
    pub pages: String,
    /// Placeholder of the title filter.
    pub filter: String,
    /// Heading of the outline panel.
    pub outline: String,
    /// Shown when a page has no headings.
    pub no_headings: String,
    /// Heading of the history panel.
    pub history: String,
    /// Shown when no revision is known.
    pub no_history: String,
    /// Status word: current.
    pub state_current: String,
    /// Status word: outdated.
    pub state_outdated: String,
    /// Status word: shown in another language.
    pub state_fallback: String,
    /// The language switch's name.
    pub language: String,
    /// Edit button (the block editor).
    pub edit: String,
    /// Share button.
    pub share: String,
    /// Hint in an empty block.
    pub block_hint: String,
    /// `{}` = how many pages the tree holds.
    pub page_count: String,
}

impl Default for WikiLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            program: s("FreeSynergy Wiki"),
            pages: s("Pages"),
            filter: s("Search by title"),
            outline: s("Outline"),
            no_headings: s("No headings on this page."),
            history: s("History"),
            no_history: s("No earlier versions known."),
            state_current: s("Current"),
            state_outdated: s("Outdated"),
            state_fallback: s("Other language"),
            language: s("Language"),
            edit: s("Edit"),
            share: s("Share"),
            block_hint: s("Type, or press / for commands"),
            page_count: s("{} pages"),
        }
    }
}
