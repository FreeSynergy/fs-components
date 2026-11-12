//! The stable name of a glyph (for tests, for translation keys of tooltips).
//!
//! Pattern: none (a pure function over the closed enum; kept apart so `glyph.rs` stays within its size).

use crate::glyph::Glyph;

impl Glyph {
    /// A stable name (for tests, for translation keys of tooltips).
    pub const fn name(self) -> &'static str {
        match self {
            Glyph::Mail => "mail",
            Glyph::Phone => "phone",
            Glyph::Person => "person",
            Glyph::Place => "place",
            Glyph::Calendar => "calendar",
            Glyph::Search => "search",
            Glyph::Plus => "plus",
            Glyph::Close => "close",
            Glyph::Check => "check",
            Glyph::Warning => "warning",
            Glyph::Copy => "copy",
            Glyph::Folder => "folder",
            Glyph::FileImage => "file-image",
            Glyph::FileText => "file-text",
            Glyph::FileArchive => "file-archive",
            Glyph::FileAudio => "file-audio",
            Glyph::FileVideo => "file-video",
            Glyph::FilePdf => "file-pdf",
            Glyph::FileSheet => "file-sheet",
            Glyph::FileDoc => "file-doc",
            Glyph::FileSlides => "file-slides",
            Glyph::FileCode => "file-code",
            Glyph::FileUnknown => "file-unknown",
            Glyph::Cut => "cut",
            Glyph::Paste => "paste",
            Glyph::Rename => "rename",
            Glyph::Share => "share",
            Glyph::Trash => "trash",
            Glyph::Sort => "sort",
            Glyph::ViewList => "view-list",
            Glyph::ViewGrid => "view-grid",
            Glyph::Details => "details",
            Glyph::Back => "back",
            Glyph::Forward => "forward",
            Glyph::Up => "up",
            Glyph::Reload => "reload",
            Glyph::Home => "home",
            Glyph::Drive => "drive",
            Glyph::Upload => "upload",
            Glyph::Download => "download",
            Glyph::Filter => "filter",
            Glyph::Transfers => "transfers",
            Glyph::More => "more",
            Glyph::Down => "down",
            Glyph::ChevronRight => "chevron-right",
            Glyph::Settings => "settings",
            Glyph::Info => "info",
            Glyph::History => "history",
            Glyph::Language => "language",
            Glyph::Star => "star",
            Glyph::Edit => "edit",
            Glyph::Outline => "outline",
        }
    }
}
