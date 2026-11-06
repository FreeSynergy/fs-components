//! The words the reader shows.
//!
//! Pattern: Value object — the workbench does not know its data or its audience, so every text it
//! shows comes from the caller (translation, ADR 0053, is the caller's). `{}` in a text is
//! replaced by the thing it talks about. The defaults are neutral English.

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
    /// `{}` = the reason.
    pub backup_failed: String,
    /// `{}` = the reason.
    pub list_failed: String,
    /// `{}` = the reason.
    pub search_failed: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            search: s("Search"),
            back: s("Back"),
            edit: s("Edit"),
            save: s("Save"),
            cancel: s("Cancel"),
            export: s("Export"),
            nothing_open: s("Nothing to show."),
            nothing_found: s("Nothing found for \"{}\"."),
            unreadable: s("This could not be read: {}"),
            missing_target: s("There is nothing at \"{}\"."),
            unsaved: s("Save or cancel your changes first."),
            discard_confirm: s("You have unsaved changes. Press Cancel again to discard them."),
            saved: s("Saved."),
            no_changes: s("Nothing was changed."),
            not_saved: s("Not saved: {}"),
            cannot_edit: s("Cannot edit this: {}"),
            not_exported: s("Not exported: {}"),
            overwrite_confirm: s("{} Press Export again to replace it."),
            lossy: s("Saving will also change: {}."),
            conflict_confirm: s("{} Your text is kept. Press Save again to replace their version."),
            backup_found: s("A draft you did not save was backed up earlier."),
            restore: s("Restore"),
            discard: s("Discard"),
            backup_stale: s(
                "The document changed after this draft was made. Saving asks before it replaces the newer version.",
            ),
            backup_failed: s("Backup of your draft failed: {}"),
            list_failed: s("The list failed: {}"),
            search_failed: s("The search failed: {}"),
        }
    }
}

impl Labels {
    /// `template` with its `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}
