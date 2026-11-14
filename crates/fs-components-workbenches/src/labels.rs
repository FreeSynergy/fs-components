//! The words the reader shows.
//!
//! Pattern: none — the workbench does not know its data or its audience, so every text it
//! shows comes from the caller or from `Labels::for_locale` (`MessageFormat` 2 files in German and English, in the module `words`).
//! `{}` in a text is replaced by the thing it talks about. The default is English.

/// Texts of the reader's interface.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Placeholder of the search field.
    pub search: String,
    /// Button: back.
    pub back: String,
    /// Button: edit.
    pub edit: String,
    /// Button: save.
    pub save: String,
    /// Button: cancel the edit.
    pub cancel: String,
    /// Button: export.
    pub export: String,
    /// Shown when nothing is opened.
    pub nothing_open: String,
    /// `{}` = the search text.
    pub nothing_found: String,
    /// `{}` = the reason.
    pub unreadable: String,
    /// `{}` = what the link meant.
    pub missing_target: String,
    /// Going elsewhere while there are unsaved changes.
    pub unsaved: String,
    /// First cancel with unsaved changes.
    pub discard_confirm: String,
    /// After a save.
    pub saved: String,
    /// Save without changes.
    pub no_changes: String,
    /// `{}` = the reason.
    pub not_saved: String,
    /// Shown with the checkboxes of a page that may not be changed (they are locked); the program's words, empty shows nothing.
    pub tasks_locked: String,
    /// `{}` = the reason.
    pub cannot_edit: String,
    /// `{}` = the reason.
    pub not_exported: String,
    /// Export would replace something; `{}` = the reason. Press again to confirm.
    pub overwrite_confirm: String,
    /// Warning before saving a text the text area cannot hold exactly; `{}` = what changes.
    pub lossy: String,
    /// After a conflict; `{}` = what happened. Pressing Save again replaces the other version.
    pub conflict_confirm: String,
    /// Offer of a backed-up draft.
    pub backup_found: String,
    /// Button: use the backed-up draft.
    pub restore: String,
    /// Button: throw the backed-up draft away.
    pub discard: String,
    /// The document changed since the backed-up draft was made.
    pub backup_stale: String,
    /// Restore with typed text present.
    pub restore_confirm: String,
    /// A stored draft names another document; `{}` = that name.
    pub backup_mismatch: String,
    /// `{}` = the reason.
    pub backup_failed: String,
    /// Characters no installed font has; `{}` = the first few, as `U+XXXX`.
    pub glyphs_missing: String,
    /// Characters in titles of the list that it cannot draw; `{}` = the first few.
    pub glyphs_titles: String,
    /// "and N more" after a list of characters; `{}` = N.
    pub glyphs_more: String,
    /// `{}` = the reason.
    pub list_failed: String,
    /// `{}` = the reason.
    pub search_failed: String,
}

impl Labels {
    /// `template` with its `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}
