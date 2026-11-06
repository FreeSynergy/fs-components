//! The words the file manager shows.
//!
//! Pattern: Value object — the workbench does not know its audience, so every text comes from the
//! caller (translation, ADR 0053, is the caller's). `{}` is replaced by what the text talks about. The
//! defaults are neutral English; the file `labels_default.rs` keeps them.

/// Texts of the file manager. Build by `Default` and assigning fields.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileLabels {
    /// Default: `Search all folders`.
    pub search: String,
    /// Default: `Files`.
    pub root: String,
    /// Default: `Trash`.
    pub trash_place: String,
    /// Default: `List`.
    pub view_list: String,
    /// Default: `Grid`.
    pub view_grid: String,
    /// Default: `Trash`.
    pub trash: String,
    /// Default: `Back to files`.
    pub leave_trash: String,
    /// Default: `Tools`.
    pub tools: String,
    /// Default: `Filter`.
    pub filter: String,
    /// Default: `Filter is on`.
    pub filter_on: String,
    /// Default: `Name`.
    pub sort_name: String,
    /// Default: `Modified`.
    pub sort_modified: String,
    /// Default: `Size`.
    pub sort_size: String,
    /// Default: `Type`.
    pub sort_type: String,
    /// Default: `Ascending`.
    pub ascending: String,
    /// Default: `Descending`.
    pub descending: String,
    /// Default: `New folder`.
    pub new_folder: String,
    /// Default: `Upload`.
    pub upload: String,
    /// Default: `Rescan`.
    pub rescan: String,
    /// Default: `Transfers`.
    pub transfers: String,
    /// Default: `Empty trash`.
    pub empty_trash: String,
    /// Default: `Back to the list`.
    pub back_to_list: String,
    /// Default: `Open folder`.
    pub open: String,
    /// Default: `Rename`.
    pub rename: String,
    /// Default: `Move`.
    pub move_to: String,
    /// Default: `Copy`.
    pub copy_to: String,
    /// Default: `Download`.
    pub download: String,
    /// Default: `Delete`.
    pub delete: String,
    /// Default: `Delete folder`.
    pub delete_folder: String,
    /// Default: `Restore`.
    pub restore: String,
    /// Default: `Restore to`.
    pub restore_to: String,
    /// Default: `Delete forever`.
    pub purge: String,
    /// Default: `Keep it`.
    pub keep: String,
    /// Default: `Show preview`.
    pub show_preview: String,
    /// Default: `Choose`.
    pub choose: String,
    /// Default: `Choose anyway`.
    pub choose_marked: String,
    /// Default: `Show more`.
    pub more: String,
    /// Default: `Try again`.
    pub retry: String,
    /// Default: `Apply`.
    pub apply: String,
    /// Default: `Clear`.
    pub clear: String,
    /// Default: `Cancel`.
    pub cancel: String,
    /// Default: `Keep typing`.
    pub keep_typing: String,
    /// Default: `Discard what I typed`.
    pub discard: String,
    /// Default: `Keep both`.
    pub keep_both: String,
    /// Default: `Back`.
    pub back: String,
    /// Default: `Move here`.
    pub move_here: String,
    /// Default: `Copy here`.
    pub copy_here: String,
    /// Default: `Restore here`.
    pub restore_here: String,
    /// Default: `Name`.
    pub col_name: String,
    /// Default: `Type`.
    pub col_type: String,
    /// Default: `Size`.
    pub col_size: String,
    /// Default: `Modified`.
    pub col_modified: String,
    /// Default: `Folder`.
    pub folder_word: String,
    /// Default: `File`.
    pub unknown_type: String,
    /// Default: `Sensitive`.
    pub marked: String,
    /// Default: `Type`.
    pub fact_type: String,
    /// Default: `Size`.
    pub fact_size: String,
    /// Default: `Modified`.
    pub fact_modified: String,
    /// Default: `Marked as`.
    pub fact_marks: String,
    /// Default: `In the trash`.
    pub in_trash: String,
    /// Default: `Nothing is selected.`.
    pub nothing_selected: String,
    /// Default: `This folder is empty.`.
    pub nothing: String,
    /// Default: `The trash is empty.`.
    pub trash_empty: String,
    /// Default: `Nothing found for <what>.`.
    pub nothing_found: String,
    /// Default: `Could not be shown: <what>`.
    pub unreadable: String,
    /// Default: `Could not read the list: <what>`.
    pub list_failed: String,
    /// Default: `Could not read the folders: <what>`.
    pub tree_failed: String,
    /// Default: `More folders: open the parent`.
    pub tree_more: String,
    /// Default: `<what> is no longer there.`.
    pub gone: String,
    /// Default: `That folder is no longer there.`.
    pub folder_gone: String,
    /// Default: `Changes made outside appear after Rescan.`.
    pub outside_hint: String,
    /// Default: `The name is taken: <what>`.
    pub name_conflict: String,
    /// Default: `This name cannot be used: <what>`.
    pub invalid_name: String,
    /// Default: `The file was changed outside the program: <what> Rescan to bring it in.`.
    pub changed_outside: String,
    /// Default: `This is not offered: <what>`.
    pub unsupported: String,
    /// Default: `That did not work: <what>`.
    pub failed: String,
    /// Default: `Not a size: <what>. Try 500, 20 KB, 3 MB or 1 GB.`.
    pub bad_size: String,
    /// Default: `Not a date: <what>. Write it as 2026-11-30.`.
    pub bad_date: String,
    /// Default: `Name of the new folder`.
    pub label_new_folder: String,
    /// Default: `New name of <what>`.
    pub label_rename: String,
    /// Default: `Path of the file to upload (there is no file dialog yet)`.
    pub label_upload: String,
    /// Default: `Path to write <what> to, a new file (there is no file dialog yet)`.
    pub label_download: String,
    /// Default: `Path of the file to go on with for <what>; it must be the same file`.
    pub label_resume: String,
    /// Default: `At least (size)`.
    pub filter_min: String,
    /// Default: `At most (size)`.
    pub filter_max: String,
    /// Default: `Changed on or after (date)`.
    pub filter_from: String,
    /// Default: `Changed before (date)`.
    pub filter_before: String,
    /// Default: `You have typed something. Discard it?`.
    pub discard_confirm: String,
    /// Default: `<what> Nothing was changed. Keep both? The incoming one gets a free name; noth...`.
    pub name_taken_question: String,
    /// Default: `Put <what> in the trash?`.
    pub trash_confirm: String,
    /// Default: `Yes, put in the trash`.
    pub trash_really: String,
    /// Default: `Delete the folder <what>? Only an empty folder can be deleted; there is no way...`.
    pub delete_folder_confirm: String,
    /// Default: `Yes, delete the folder`.
    pub delete_folder_really: String,
    /// Default: `Delete <what> forever? There is no way back.`.
    pub purge_confirm: String,
    /// Default: `Yes, delete forever`.
    pub purge_really: String,
    /// Default: `Delete all <what> files in the trash forever? There is no way back.`.
    pub empty_confirm: String,
    /// Default: `Delete all <what> files in the trash forever, also the ones marked sensitive? ...`.
    pub empty_confirm_marked: String,
    /// Default: `Yes, delete all forever`.
    pub empty_really: String,
    /// Default: `The trash has changed since you asked. Nothing was deleted; look at the number...`.
    pub empty_changed: String,
    /// Default: `<what> is marked sensitive. Write it to the path you gave, outside the program?`.
    pub download_marked_confirm: String,
    /// Default: `Yes, write this marked file`.
    pub download_marked_really: String,
    /// Default: `Throw away the unfinished upload <what> and what arrived of it?`.
    pub drop_confirm: String,
    /// Default: `Yes, throw it away`.
    pub drop_really: String,
    /// Default: `<what> is marked sensitive. Give it to the program that asked?`.
    pub choose_marked_confirm: String,
    /// Default: `Moving <what>: open the folder it should go to.`.
    pub carrying_move: String,
    /// Default: `Copying <what>: open the folder the copy should go to.`.
    pub carrying_copy: String,
    /// Default: `Restoring <what>: open the folder it should go to.`.
    pub carrying_restore: String,
    /// Default: `Made the folder <what>.`.
    pub created: String,
    /// Default: `It is now called <what>.`.
    pub renamed: String,
    /// Default: `Moved <what>.`.
    pub moved: String,
    /// Default: `The copy is called <what>.`.
    pub copied: String,
    /// Default: `<what> is in the trash.`.
    pub trashed: String,
    /// Default: `Deleted the folder <what>.`.
    pub folder_deleted: String,
    /// Default: `Restored as <what>.`.
    pub restored: String,
    /// Default: `<what> is gone for good.`.
    pub purged: String,
    /// Default: `<what> files are gone for good.`.
    pub emptied: String,
    /// Default: `These stay in the trash, they could not be deleted: <what>`.
    pub emptied_failed: String,
    /// Default: `Uploaded as <what>.`.
    pub uploaded: String,
    /// Default: `Wrote <what>.`.
    pub downloaded: String,
    /// Default: `Threw away the upload <what>.`.
    pub dropped: String,
    /// Default: `Rescan done. See the report.`.
    pub rescan_done: String,
    /// Default: `Taken over`.
    pub rescan_added: String,
    /// Default: `Changed`.
    pub rescan_updated: String,
    /// Default: `Missing: the record stays, nothing was deleted`.
    pub rescan_missing: String,
    /// Default: `Not taken over`.
    pub rescan_skipped: String,
    /// Default: `and <what> more`.
    pub rescan_more: String,
    /// Default: `Changes from outside only come in by Rescan; nothing watches the folder. Resca...`.
    pub rescan_blocks: String,
    /// Default: `Marked sensitive: not shown unless you ask.`.
    pub preview_marked: String,
    /// Default: `A file in the trash cannot be read. Restore it first.`.
    pub preview_trash: String,
    /// Default: `No preview of this kind of file.`.
    pub preview_binary: String,
    /// Default: `Too big for a preview.`.
    pub preview_too_big: String,
    /// Default: `Only the start is shown.`.
    pub preview_cut: String,
    /// Default: `Unfinished: try again to go on, or dismiss to throw it away`.
    pub unfinished: String,
    /// Default: `Transfers`.
    pub transfers_title: String,
    /// Default: `Transfers hold the window until they are done, so no progress shows while they...`.
    pub transfers_note: String,
    /// Default: `Filter this folder by size and date`.
    pub filter_title: String,
    /// Default: `What the rescan found`.
    pub report_title: String,
}

impl FileLabels {
    /// `text` with `{}` replaced by `what`.
    pub fn fill(text: &str, what: &str) -> String {
        text.replacen("{}", what, 1)
    }
}
