//! The names of the glyphs: the freedesktop Icon Naming Specification where it has a name, our own `freesynergy-` where not.
//!
//! Pattern: Value object — a table of (glyph, standard name, context). The same table gives the theme package
//! (directory by context, file by name) and the way back from a name to a glyph, so a name that a theme does not know
//! is still drawn by the built-in set.

use crate::glyph::Glyph;

/// The context (the directory of a theme, by the Icon Theme Specification) a glyph belongs to.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconContext {
    /// Things people do: edit-copy, go-previous ...
    Actions,
    /// Kinds of file: text-x-generic, image-x-generic ...
    MimeTypes,
    /// Places: folder, user-home, user-trash ...
    Places,
    /// Devices: drive-harddisk ...
    Devices,
    /// Status: dialog-warning ...
    Status,
    /// Categories: preferences-system ...
    Categories,
}

impl IconContext {
    /// The directory name of the context in a theme (`actions`, `mimetypes`, ...).
    pub const fn directory(self) -> &'static str {
        match self {
            IconContext::Actions => "actions",
            IconContext::MimeTypes => "mimetypes",
            IconContext::Places => "places",
            IconContext::Devices => "devices",
            IconContext::Status => "status",
            IconContext::Categories => "categories",
        }
    }

    /// The `Context=` of the directory in `index.theme`.
    pub const fn context(self) -> &'static str {
        match self {
            IconContext::Actions => "Actions",
            IconContext::MimeTypes => "MimeTypes",
            IconContext::Places => "Places",
            IconContext::Devices => "Devices",
            IconContext::Status => "Status",
            IconContext::Categories => "Categories",
        }
    }
}

const NAMES: &[(Glyph, &str, IconContext)] = &[
    (Glyph::Mail, "mail-message-new", IconContext::Actions),
    (Glyph::Phone, "call-start", IconContext::Actions),
    (Glyph::Person, "avatar-default", IconContext::Status),
    (Glyph::Place, "freesynergy-place", IconContext::Places),
    (Glyph::Calendar, "x-office-calendar", IconContext::MimeTypes),
    (Glyph::Search, "system-search", IconContext::Actions),
    (Glyph::Plus, "list-add", IconContext::Actions),
    (Glyph::Close, "window-close", IconContext::Actions),
    (Glyph::Check, "freesynergy-check", IconContext::Status),
    (Glyph::Warning, "dialog-warning", IconContext::Status),
    (Glyph::Copy, "edit-copy", IconContext::Actions),
    (Glyph::Folder, "folder", IconContext::Places),
    (Glyph::FileImage, "image-x-generic", IconContext::MimeTypes),
    (Glyph::FileText, "text-x-generic", IconContext::MimeTypes),
    (
        Glyph::FileArchive,
        "package-x-generic",
        IconContext::MimeTypes,
    ),
    (Glyph::FileAudio, "audio-x-generic", IconContext::MimeTypes),
    (Glyph::FileVideo, "video-x-generic", IconContext::MimeTypes),
    (Glyph::FilePdf, "application-pdf", IconContext::MimeTypes),
    (
        Glyph::FileSheet,
        "x-office-spreadsheet",
        IconContext::MimeTypes,
    ),
    (Glyph::FileDoc, "x-office-document", IconContext::MimeTypes),
    (
        Glyph::FileSlides,
        "x-office-presentation",
        IconContext::MimeTypes,
    ),
    (Glyph::FileCode, "text-x-script", IconContext::MimeTypes),
    (
        Glyph::FileUnknown,
        "freesynergy-file-unknown",
        IconContext::MimeTypes,
    ),
    (Glyph::Cut, "edit-cut", IconContext::Actions),
    (Glyph::Paste, "edit-paste", IconContext::Actions),
    (
        Glyph::Rename,
        "freesynergy-edit-rename",
        IconContext::Actions,
    ),
    (Glyph::Share, "freesynergy-share", IconContext::Actions),
    (Glyph::Trash, "user-trash", IconContext::Places),
    (Glyph::Sort, "view-sort-ascending", IconContext::Actions),
    (Glyph::ViewList, "view-list-details", IconContext::Actions),
    (Glyph::ViewGrid, "view-grid", IconContext::Actions),
    (
        Glyph::Details,
        "freesynergy-view-details",
        IconContext::Actions,
    ),
    (Glyph::Back, "go-previous", IconContext::Actions),
    (Glyph::Forward, "go-next", IconContext::Actions),
    (Glyph::Up, "go-up", IconContext::Actions),
    (Glyph::Reload, "view-refresh", IconContext::Actions),
    (Glyph::Home, "user-home", IconContext::Places),
    (Glyph::Drive, "drive-harddisk", IconContext::Devices),
    (Glyph::Upload, "freesynergy-upload", IconContext::Actions),
    (
        Glyph::Download,
        "freesynergy-download",
        IconContext::Actions,
    ),
    (Glyph::Filter, "freesynergy-filter", IconContext::Actions),
    (
        Glyph::Transfers,
        "freesynergy-transfers",
        IconContext::Actions,
    ),
    (Glyph::Down, "go-down", IconContext::Actions),
    (
        Glyph::ChevronRight,
        "freesynergy-chevron-right",
        IconContext::Actions,
    ),
    (
        Glyph::Settings,
        "preferences-system",
        IconContext::Categories,
    ),
    (Glyph::More, "freesynergy-more", IconContext::Actions),
    (Glyph::Info, "dialog-information", IconContext::Status),
    (Glyph::History, "document-open-recent", IconContext::Actions),
    (
        Glyph::Language,
        "freesynergy-language",
        IconContext::Actions,
    ),
    (Glyph::Star, "starred", IconContext::Status),
    (Glyph::Edit, "document-edit", IconContext::Actions),
    (Glyph::Outline, "freesynergy-outline", IconContext::Actions),
];

impl Glyph {
    /// The icon name: the standard one, or ours with the prefix `freesynergy-` where the standard has none.
    pub fn icon_name(self) -> &'static str {
        NAMES
            .iter()
            .find(|(g, ..)| *g == self)
            .map_or("freesynergy-file-unknown", |(_, n, _)| n)
    }

    /// The context of the glyph in a theme.
    pub fn icon_context(self) -> IconContext {
        NAMES
            .iter()
            .find(|(g, ..)| *g == self)
            .map_or(IconContext::Actions, |(.., c)| *c)
    }

    /// The glyph of an icon name: its own name, or one of the other standard names that mean the same.
    pub fn from_icon_name(name: &str) -> Option<Glyph> {
        const ALIASES: &[(&str, Glyph)] = &[
            ("document-new", Glyph::Plus),
            ("folder-new", Glyph::Folder),
            ("edit-delete", Glyph::Trash),
            ("edit-find", Glyph::Search),
            ("go-home", Glyph::Home),
            ("drive-removable-media", Glyph::Drive),
            ("view-sort-descending", Glyph::Sort),
            ("document-send", Glyph::Upload),
            ("application-x-zip", Glyph::FileArchive),
            ("text-html", Glyph::FileCode),
        ];
        NAMES
            .iter()
            .find(|(_, n, _)| *n == name)
            .map(|(g, ..)| *g)
            .or_else(|| ALIASES.iter().find(|(n, _)| *n == name).map(|(_, g)| *g))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_has_one_name_and_the_name_leads_back() {
        let mut seen = Vec::new();
        for g in Glyph::ALL {
            let name = g.icon_name();
            assert!(!seen.contains(&name), "{name} is used twice");
            seen.push(name);
            assert_eq!(Glyph::from_icon_name(name), Some(g), "{name}");
        }
        assert_eq!(NAMES.len(), Glyph::ALL.len());
    }

    #[test]
    fn our_own_names_have_the_prefix_and_the_standard_ones_are_the_standard() {
        for g in Glyph::ALL {
            let name = g.icon_name();
            let own = name.starts_with("freesynergy-");
            assert!(own || !name.contains(' '), "{name}");
        }
        assert_eq!(Glyph::Copy.icon_name(), "edit-copy");
        assert_eq!(Glyph::Back.icon_name(), "go-previous");
        assert_eq!(Glyph::from_icon_name("edit-delete"), Some(Glyph::Trash));
    }
}
