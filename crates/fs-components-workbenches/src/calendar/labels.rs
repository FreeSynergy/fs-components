//! The words the calendar workbench shows.
//!
//! Pattern: Value object — the workbench does not know its audience, so every text comes from the
//! caller (translation, ADR 0053, is the caller's). `{}` is replaced by what the text talks about.
//! The defaults are neutral English.

use fs_components_widgets::CalendarNames;

/// Texts of the calendar workbench.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarLabels {
    /// Names of days and months and the words of the calendar view.
    pub names: CalendarNames,
    /// Button: go to today.
    pub today: String,
    /// Button: one step back.
    pub previous: String,
    /// Button: one step on.
    pub next: String,
    /// Button: show a month.
    pub month: String,
    /// Button: show a week.
    pub week: String,
    /// Button: show a day.
    pub day: String,
    /// Button: show the agenda.
    pub agenda: String,
    /// Button: a new item.
    pub create: String,
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
    /// Button: keep the item after all.
    pub delete_keep: String,
    /// Button: back to the calendar, where the detail takes its place.
    pub back: String,
    /// A flag that is on.
    pub yes: String,
    /// A flag that is off.
    pub no: String,
    /// The title of a month; `{month}` and `{year}` are filled in.
    pub month_title: String,
    /// Shown when nothing is selected.
    pub nothing_selected: String,
    /// The first press of Delete; `{}` = the item.
    pub delete_confirm: String,
    /// The item shown is no longer there.
    pub gone: String,
    /// Going elsewhere while there are unsaved changes.
    pub unsaved: String,
    /// First cancel with unsaved changes.
    pub discard_confirm: String,
    /// After a save.
    pub saved: String,
    /// After making a new item.
    pub created: String,
    /// After a delete; `{}` = the item.
    pub deleted: String,
    /// Save without changes.
    pub no_changes: String,
    /// The calendar does not go further; `{}` = the year.
    pub edge: String,
    /// `{}` = the reason.
    pub not_saved: String,
    /// `{}` = the reason.
    pub not_deleted: String,
    /// `{}` = the reason.
    pub cannot_edit: String,
    /// `{}` = the reason.
    pub unreadable: String,
    /// `{}` = the reason.
    pub items_failed: String,
    /// After a conflict; `{}` = what happened. Pressing Save again writes over it knowingly.
    pub conflict_confirm: String,
}

impl Default for CalendarLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            names: CalendarNames::new(
                ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].map(s),
                [
                    "January",
                    "February",
                    "March",
                    "April",
                    "May",
                    "June",
                    "July",
                    "August",
                    "September",
                    "October",
                    "November",
                    "December",
                ]
                .map(s),
                "all day",
                "Nothing here.",
            ),
            today: s("Today"),
            previous: s("<"),
            next: s(">"),
            month: s("Month"),
            week: s("Week"),
            day: s("Day"),
            agenda: s("Agenda"),
            create: s("New"),
            edit: s("Edit"),
            save: s("Save"),
            cancel: s("Cancel"),
            delete: s("Delete"),
            delete_really: s("Really delete"),
            delete_keep: s("Keep"),
            back: s("Back"),
            yes: s("Yes"),
            no: s("No"),
            month_title: s("{month} {year}"),
            nothing_selected: s("Nothing selected."),
            delete_confirm: s("Delete {}? This cannot be taken back here."),
            gone: s("This item no longer exists."),
            unsaved: s("Save or cancel your changes first."),
            discard_confirm: s("You have unsaved changes. Press Cancel again to discard them."),
            saved: s("Saved."),
            created: s("Created."),
            deleted: s("{} was deleted."),
            no_changes: s("Nothing was changed."),
            edge: s("The calendar goes no further than the year {}."),
            not_saved: s("Not saved: {}"),
            not_deleted: s("Not deleted: {}"),
            cannot_edit: s("Cannot edit this: {}"),
            unreadable: s("This could not be read: {}"),
            items_failed: s("The calendar could not be read: {}"),
            conflict_confirm: s("{} Your text is kept. Press Save again to write it anyway."),
        }
    }
}

impl CalendarLabels {
    /// `template` with its `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}
