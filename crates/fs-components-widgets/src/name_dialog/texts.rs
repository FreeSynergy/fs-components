//! The texts of the name dialog in English and German, as tables of `fs-components-words`.
//!
//! Pattern: none (data) — an app with its own catalog may use [`IDS`] to see what to translate.

use fs_components_words::{Catalog, EnglishPlural};

/// Title for a new page.
pub const NEW_PAGE: &str = "name-dialog.new-page";
/// Title for a sub-page (`{about}` = the parent).
pub const SUB_PAGE: &str = "name-dialog.sub-page";
/// Title for a new name (`{about}` = the old name).
pub const RENAME: &str = "name-dialog.rename";
/// Placeholder of the field.
pub const PLACEHOLDER: &str = "name-dialog.placeholder";
/// Button: take the name.
pub const OK: &str = "name-dialog.ok";
/// Button: leave.
pub const CANCEL: &str = "name-dialog.cancel";

/// Every identifier the dialog asks for.
pub const IDS: [&str; 6] = [NEW_PAGE, SUB_PAGE, RENAME, PLACEHOLDER, OK, CANCEL];

/// The English texts.
pub fn english() -> Catalog {
    Catalog::from_table(
        EnglishPlural,
        &[
            (NEW_PAGE, "New page"),
            (SUB_PAGE, "New sub-page of {about}"),
            (RENAME, "Rename {about}"),
            (PLACEHOLDER, "Name of the page"),
            (OK, "OK"),
            (CANCEL, "Cancel"),
        ],
    )
}

/// The German texts.
pub fn german() -> Catalog {
    Catalog::from_table(
        EnglishPlural,
        &[
            (NEW_PAGE, "Neue Seite"),
            (SUB_PAGE, "Neue Unterseite von {about}"),
            (RENAME, "{about} umbenennen"),
            (PLACEHOLDER, "Name der Seite"),
            (OK, "OK"),
            (CANCEL, "Abbrechen"),
        ],
    )
}
