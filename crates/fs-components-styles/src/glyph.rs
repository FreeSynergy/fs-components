//! The glyphs of the icon set: their names and outlines (24 x 24 grid), drawn by us.
//!
//! Pattern: Value object — a closed enum of glyphs over static path data; how they are drawn is `IconSet`.

/// What an icon shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Glyph {
    /// An envelope: e-mail address.
    Mail,
    /// A handset: phone number.
    Phone,
    /// A head and shoulders: person, name.
    Person,
    /// A map pin: place, postal address.
    Place,
    /// A calendar page: date.
    Calendar,
    /// A magnifier.
    Search,
    /// A plus.
    Plus,
    /// A cross.
    Close,
    /// A tick.
    Check,
    /// A triangle with a mark.
    Warning,
    /// Two sheets.
    Copy,
    /// A folder.
    Folder,
    /// A picture file: a sheet with a hill and a sun.
    FileImage,
    /// A text file: a sheet with lines.
    FileText,
    /// An archive: a sheet with a zip.
    FileArchive,
    /// An audio file: a sheet with a note.
    FileAudio,
    /// A video file: a sheet with a play mark.
    FileVideo,
    /// A document for reading: a sheet with a fold mark.
    FilePdf,
    /// A spreadsheet: a sheet with a grid.
    FileSheet,
    /// A document: a sheet with a heading and lines.
    FileDoc,
    /// A presentation: a sheet with a screen.
    FileSlides,
    /// A code file: a sheet with brackets.
    FileCode,
    /// A file of a kind we do not know: a plain sheet.
    FileUnknown,
    /// Scissors: cut.
    Cut,
    /// A clipboard: paste.
    Paste,
    /// A pencil: rename.
    Rename,
    /// Three dots joined: share.
    Share,
    /// A bin: trash.
    Trash,
    /// Two arrows, up and down: sort.
    Sort,
    /// Lines: list view.
    ViewList,
    /// Four squares: grid view.
    ViewGrid,
    /// A panel at the side: details.
    Details,
    /// An arrow to the left: back.
    Back,
    /// An arrow to the right: forward.
    Forward,
    /// An arrow up: up one folder.
    Up,
    /// A circle with an arrow: reload.
    Reload,
    /// A house: home.
    Home,
    /// A drive: a place to keep files.
    Drive,
    /// An arrow into a tray: upload.
    Upload,
    /// An arrow out of a tray: download.
    Download,
    /// A funnel: filter.
    Filter,
    /// Two arrows, one up and one down: what is being sent and received.
    Transfers,
    /// A chevron pointing down: open, fold out.
    Down,
    /// A chevron pointing right: folded, a step in a path.
    ChevronRight,
    /// A slider: settings.
    Settings,
    /// Three dots: more.
    More,
    /// A circle with an i: information.
    Info,
}

impl Glyph {
    /// All glyphs, for tests and for icon sheets.
    pub const ALL: [Glyph; 47] = [
        Glyph::Mail,
        Glyph::Phone,
        Glyph::Person,
        Glyph::Place,
        Glyph::Calendar,
        Glyph::Search,
        Glyph::Plus,
        Glyph::Close,
        Glyph::Check,
        Glyph::Warning,
        Glyph::Copy,
        Glyph::Folder,
        Glyph::FileImage,
        Glyph::FileText,
        Glyph::FileArchive,
        Glyph::FileAudio,
        Glyph::FileVideo,
        Glyph::FilePdf,
        Glyph::FileSheet,
        Glyph::FileDoc,
        Glyph::FileSlides,
        Glyph::FileCode,
        Glyph::FileUnknown,
        Glyph::Cut,
        Glyph::Paste,
        Glyph::Rename,
        Glyph::Share,
        Glyph::Trash,
        Glyph::Sort,
        Glyph::ViewList,
        Glyph::ViewGrid,
        Glyph::Details,
        Glyph::Back,
        Glyph::Forward,
        Glyph::Up,
        Glyph::Reload,
        Glyph::Home,
        Glyph::Drive,
        Glyph::Upload,
        Glyph::Download,
        Glyph::Filter,
        Glyph::Transfers,
        Glyph::Down,
        Glyph::ChevronRight,
        Glyph::Settings,
        Glyph::More,
        Glyph::Info,
    ];

    /// The outline as SVG path data on a 24 x 24 grid.
    pub const fn path(self) -> &'static str {
        match self {
            Glyph::Mail => "M4 6h16v12H4z M4 7l8 6 8-6",
            Glyph::Phone => {
                "M7 3h3l1.5 4-2 1.5a11 11 0 0 0 5 5L16.5 11.5 21 13v3a2 2 0 0 1-2 2A16 16 0 0 1 5 5a2 2 0 0 1 2-2z"
            }
            Glyph::Person => "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M4 20c0-4 3.5-6 8-6s8 2 8 6",
            Glyph::Place => {
                "M12 21s7-6.2 7-11a7 7 0 1 0-14 0c0 4.8 7 11 7 11z M12 12a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z"
            }
            Glyph::Calendar => "M4 6h16v14H4z M4 10h16 M8 3v4 M16 3v4",
            Glyph::Search => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14z M16 16l4.5 4.5",
            Glyph::Plus => "M12 5v14 M5 12h14",
            Glyph::Close => "M6 6l12 12 M18 6L6 18",
            Glyph::Check => "M5 12.5l4.5 4.5L19 7.5",
            Glyph::Warning => "M12 4L21 20H3z M12 10v4.5 M12 17.2v.3",
            Glyph::Copy => "M8 8h11v12H8z M5 16V4h11",
            Glyph::Folder => {
                "M3 7a2 2 0 0 1 2-2h4.5l2 2.5H19a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
            }
            Glyph::FileImage => "M6 3h8l4 4v14H6z M14 3v4h4 M8.5 17l2.5-3.5 2 2 1.5-1.5 2 3z",
            Glyph::FileText => "M6 3h8l4 4v14H6z M14 3v4h4 M9 12h6 M9 15h6 M9 18h4",
            Glyph::FileArchive => {
                "M6 3h8l4 4v14H6z M14 3v4h4 M11 8h2 M11 11h2 M11 14h2 M10.5 17h3v2h-3z"
            }
            Glyph::FileAudio => {
                "M6 3h8l4 4v14H6z M14 3v4h4 M11 17V11l4-1v6 M9.5 18.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z M13.5 17.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z"
            }
            Glyph::FileVideo => "M6 3h8l4 4v14H6z M14 3v4h4 M10 11l5 3-5 3z",
            Glyph::FilePdf => {
                "M6 3h8l4 4v14H6z M14 3v4h4 M8.5 17.5c1.5-1 3-4 3.5-7 .5 3 2 5 3.5 5.5-2 0-4.5.5-7 1.5z"
            }
            Glyph::FileSheet => "M6 3h8l4 4v14H6z M14 3v4h4 M8 11h8 M8 14h8 M8 17h8 M11 11v8",
            Glyph::FileDoc => "M6 3h8l4 4v14H6z M14 3v4h4 M8.5 11h7 M8.5 14h7 M8.5 17h5",
            Glyph::FileSlides => "M6 3h8l4 4v14H6z M14 3v4h4 M8 11h8v5H8z M12 16v2.5",
            Glyph::FileCode => {
                "M6 3h8l4 4v14H6z M14 3v4h4 M10.5 11l-2 2.5 2 2.5 M13.5 11l2 2.5-2 2.5"
            }
            Glyph::FileUnknown => "M6 3h8l4 4v14H6z M14 3v4h4",
            Glyph::Cut => {
                "M7 9a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M7 20a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M8.8 7.8L19 17 M8.8 16.2L19 7"
            }
            Glyph::Paste => "M7 5h10v16H7z M9.5 3h5v4h-5z M10 12h4 M10 15.5h4",
            Glyph::Rename => "M4 20l1-4L16.5 4.5l3 3L8 19z M14.5 6.5l3 3",
            Glyph::Share => {
                "M18 8a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M6 14.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M18 21a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M8.2 10.8l7.6-4.1 M8.2 13.2l7.6 4.1"
            }
            Glyph::Trash => "M5 7h14 M9 7V4.5h6V7 M7 7l1 13h8l1-13 M10 11v6 M14 11v6",
            Glyph::Sort => "M8 19V5 M4.5 8.5L8 5l3.5 3.5 M16 5v14 M12.5 15.5L16 19l3.5-3.5",
            Glyph::ViewList => "M4 6h16 M4 12h16 M4 18h16",
            Glyph::ViewGrid => {
                "M4 4h6.5v6.5H4z M13.5 4H20v6.5h-6.5z M4 13.5h6.5V20H4z M13.5 13.5H20V20h-6.5z"
            }
            Glyph::Details => "M4 5h16v14H4z M14 5v14 M16.5 9h1 M16.5 12h1",
            Glyph::Back => "M19 12H5 M11 6l-6 6 6 6",
            Glyph::Forward => "M5 12h14 M13 6l6 6-6 6",
            Glyph::Up => "M12 19V5 M6 11l6-6 6 6",
            Glyph::Reload => "M19.5 12a7.5 7.5 0 1 1-2.3-5.4 M19.5 4.5v4h-4",
            Glyph::Home => "M4 11l8-7 8 7 M6 10v10h12V10 M10 20v-6h4v6",
            Glyph::Drive => "M4 14h16v5H4z M6 14l2-8h8l2 8 M16.5 16.5h.5",
            Glyph::Upload => "M12 16V5 M7 9.5L12 5l5 4.5 M5 15v4h14v-4",
            Glyph::Download => "M12 5v11 M7 11.5l5 5 5-5 M5 15v4h14v-4",
            Glyph::Filter => "M4 5h16l-6 8v6l-4-2v-4z",
            Glyph::More => "M5 12h.5 M12 12h.5 M19 12h.5",
            Glyph::Down => "M6 9l6 6 6-6",
            Glyph::ChevronRight => "M9 6l6 6-6 6",
            Glyph::Settings => "M4 7h10 M18 7h2 M14 5v4 M4 17h2 M10 17h10 M6 15v4 M4 12h16",
            Glyph::Transfers => "M7 4v14 M3.5 14.5L7 18l3.5-3.5 M17 20V6 M13.5 9.5L17 6l3.5 3.5",
            Glyph::Info => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z M12 11v6 M12 7.5v.5",
        }
    }

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
        }
    }
}
