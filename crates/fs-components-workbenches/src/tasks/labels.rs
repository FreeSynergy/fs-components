//! The words the tasks workbench shows.
//!
//! Pattern: Value object — the workbench does not know its audience, so every text comes from the
//! caller (translation, ADR 0053, is the caller's). `{}` is replaced by what the text talks about.
//! The defaults are neutral English.

/// Texts of the tasks workbench.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskLabels {
    /// Placeholder of the search field.
    pub search: String,
    /// Button: the list view.
    pub view_list: String,
    /// Button: the view by status.
    pub view_status: String,
    /// The headings of the list; each row has one text per column.
    pub columns: Vec<String>,
    /// Button: load more rows.
    pub more: String,
    /// Button: a new task.
    pub create: String,
    /// Button: open the card under the keyboard's cursor.
    pub open: String,
    /// Button: edit.
    pub edit: String,
    /// Button: save.
    pub save: String,
    /// Button: cancel the edit.
    pub cancel: String,
    /// Button: delete.
    pub delete: String,
    /// Button: the second action that really deletes.
    pub delete_really: String,
    /// Button: keep the task after all.
    pub delete_keep: String,
    /// Shown when no task is selected.
    pub nothing_selected: String,
    /// The list is empty.
    pub nothing: String,
    /// The search found nothing; `{}` = the search text.
    pub nothing_found: String,
    /// The first press of Delete; `{}` = the task.
    pub delete_confirm: String,
    /// The task shown is no longer there.
    pub gone: String,
    /// Going elsewhere while there are unsaved changes.
    pub unsaved: String,
    /// Cancel with unsaved changes.
    pub discard_confirm: String,
    /// Button: go on editing after being asked about discarding.
    pub discard_keep: String,
    /// Button: the second action that really discards.
    pub discard_really: String,
    /// The task edited is gone, the draft stays.
    pub gone_draft: String,
    /// After a save.
    pub saved: String,
    /// After making a new task.
    pub created: String,
    /// After a delete; `{}` = the task.
    pub deleted: String,
    /// Save without changes.
    pub no_changes: String,
    /// `{}` = the reason.
    pub not_saved: String,
    /// `{}` = the reason.
    pub not_deleted: String,
    /// `{}` = the reason.
    pub cannot_edit: String,
    /// `{}` = the reason.
    pub unreadable: String,
    /// `{}` = the reason.
    pub list_failed: String,
    /// After a conflict; `{}` = what happened. Pressing Save again writes over it knowingly.
    pub conflict_confirm: String,
    /// Heading of the buttons that move the task shown.
    pub move_to: String,
    /// Shown wherever a task can be moved: what moving does.
    pub move_warning: String,
    /// After a move; `{}` = the task, then the column.
    pub moved: String,
    /// `{}` = the reason.
    pub not_moved: String,
    /// A card was put elsewhere in its own column.
    pub order_is_fixed: String,
    /// In a column without cards.
    pub column_empty: String,
    /// Cards before the drawn ones; `{}` = how many.
    pub earlier: String,
    /// Cards after the drawn ones; `{}` = how many.
    pub later: String,
}

impl Default for TaskLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            search: s("Search"),
            view_list: s("List"),
            view_status: s("Status"),
            columns: vec![s("Title")],
            more: s("Show more"),
            create: s("New"),
            open: s("Open"),
            edit: s("Edit"),
            save: s("Save"),
            cancel: s("Cancel"),
            delete: s("Delete"),
            delete_really: s("Really delete"),
            delete_keep: s("Keep"),
            nothing_selected: s("No task selected."),
            nothing: s("No tasks yet."),
            nothing_found: s("Nothing found for \"{}\"."),
            delete_confirm: s("Delete {}? This cannot be taken back here."),
            gone: s("This task no longer exists."),
            unsaved: s("Save or cancel your changes first."),
            discard_confirm: s("You have unsaved changes. Keep editing, or discard them."),
            discard_keep: s("Keep editing"),
            discard_really: s("Discard changes"),
            gone_draft: s(
                "This task no longer exists. What you typed is still here; it cannot be saved.",
            ),
            saved: s("Saved."),
            created: s("Created."),
            deleted: s("{} was deleted."),
            no_changes: s("Nothing was changed."),
            not_saved: s("Not saved: {}"),
            not_deleted: s("Not deleted: {}"),
            cannot_edit: s("Cannot edit this: {}"),
            unreadable: s("This could not be read: {}"),
            list_failed: s("The list failed: {}"),
            conflict_confirm: s("{} Your text is kept. Press Save again to write it anyway."),
            move_to: s("Move to"),
            move_warning: s(
                "Moving a card changes the status of the task itself, for everyone. A status worked out from the participants becomes fixed and cannot go back to that here.",
            ),
            moved: s("{} now has the status {}, for everyone."),
            not_moved: s("Not moved: {}"),
            order_is_fixed: s("The order within a column is not kept. Nothing was changed."),
            column_empty: s("Empty"),
            earlier: s("{} more above"),
            later: s("{} more below"),
        }
    }
}

impl TaskLabels {
    /// `template` with its first `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }

    /// `template` with its first `{}` replaced by `first` and its second by `second`, whatever those
    /// two hold.
    pub fn fill_two(template: &str, first: &str, second: &str) -> String {
        match template.split_once("{}") {
            Some((head, tail)) => format!("{head}{first}{}", tail.replacen("{}", second, 1)),
            None => template.to_string(),
        }
    }
}
