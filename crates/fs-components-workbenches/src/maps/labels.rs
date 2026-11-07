//! The words the maps workbench shows.
//!
//! Pattern: Value object — the workbench does not know its audience, so every text comes from the
//! caller (translation, ADR 0053, is the caller's). `{}` is replaced by what the text talks about. The
//! defaults are neutral English.

use fs_components_widgets::map_view::MapViewLabels;

/// Texts of the maps workbench.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapLabels {
    /// The signs on the map's buttons.
    pub view: MapViewLabels,
    /// Placeholder of the search field.
    pub search: String,
    /// Placeholder of the field for a position.
    pub go_to_field: String,
    /// Button: go to the position typed.
    pub go_to: String,
    /// Button: a new place at the centre of the map.
    pub create: String,
    /// Button: show the places (where the map takes the room).
    pub places: String,
    /// Button: show the map (where the places take the room).
    pub map: String,
    /// Button: show this place on the map.
    pub show_on_map: String,
    /// Button: use the centre of the map (compact variant).
    pub use_centre: String,
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
    /// Button: keep the place after all.
    pub delete_keep: String,
    /// Button: back to the list.
    pub back: String,
    /// Button: show more places.
    pub more: String,
    /// Button: go on editing after being asked about discarding.
    pub discard_keep: String,
    /// Button: the second action that really discards.
    pub discard_really: String,
    /// Shown when no place is selected.
    pub nothing_selected: String,
    /// Shown when there are no places.
    pub no_places: String,
    /// `{}` = the search text.
    pub none_found: String,
    /// The first press of Delete; `{}` = the place.
    pub delete_confirm: String,
    /// The place shown is no longer there.
    pub gone: String,
    /// Going elsewhere while there are unsaved changes.
    pub unsaved: String,
    /// First cancel with unsaved changes.
    pub discard_confirm: String,
    /// After a save.
    pub saved: String,
    /// After making a place.
    pub created: String,
    /// After a delete; `{}` = the place.
    pub deleted: String,
    /// After a delete of something marked: its name is not repeated.
    pub deleted_marked: String,
    /// Save of a place that is gone: the draft stays.
    pub gone_draft: String,
    /// Save without changes.
    pub no_changes: String,
    /// The place has no position to show.
    pub no_position: String,
    /// After going to a position; `{}` = the position.
    pub went_to: String,
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
    /// `{}` = the reason.
    pub cannot_go: String,
    /// After a conflict; `{}` = what happened.
    pub conflict_confirm: String,
    /// Tiles that could not be read; `{}` = the first, `{count}` is counted in `tiles_more`.
    pub tile_failed: String,
    /// More tiles failed; `{}` = how many more.
    pub tiles_more: String,
    /// Shapes left out because there are too many to draw; `{}` = how many.
    pub too_many: String,
    /// More tiles were needed than are drawn at once.
    pub too_wide: String,
    /// Tiles come from a source that has none here.
    pub no_tiles: String,
    /// The place a pick returned; `{}` = the position text.
    pub picked: String,
    /// `{}` = how many more problems than shown.
    pub more_problems: String,
}

impl Default for MapLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            view: MapViewLabels::default(),
            search: s("Search places"),
            go_to_field: s("Go to (geo: URI)"),
            go_to: s("Go"),
            create: s("New place"),
            places: s("Places"),
            map: s("Map"),
            show_on_map: s("Show on map"),
            use_centre: s("Use the centre"),
            edit: s("Edit"),
            save: s("Save"),
            cancel: s("Cancel"),
            delete: s("Delete"),
            delete_really: s("Really delete"),
            delete_keep: s("Keep"),
            back: s("Back"),
            more: s("Show more"),
            discard_keep: s("Keep editing"),
            discard_really: s("Discard changes"),
            nothing_selected: s("No place selected."),
            no_places: s("There are no places."),
            none_found: s("No place matches \"{}\"."),
            delete_confirm: s("Delete {}? This cannot be taken back here."),
            gone: s("This place no longer exists."),
            unsaved: s("Save or cancel your changes first."),
            discard_confirm: s("You have unsaved changes. Keep editing, or discard them."),
            saved: s("Saved."),
            created: s("Created."),
            deleted: s("{} was deleted."),
            deleted_marked: s("The place was deleted."),
            gone_draft: s(
                "This place no longer exists, so this cannot be saved. Your text is still here.",
            ),
            no_changes: s("Nothing was changed."),
            no_position: s("This place has no position to show on the map."),
            went_to: s("Went to {}."),
            not_saved: s("Not saved: {}"),
            not_deleted: s("Not deleted: {}"),
            cannot_edit: s("Cannot edit this: {}"),
            unreadable: s("This could not be read: {}"),
            list_failed: s("The places could not be read: {}"),
            cannot_go: s("Cannot go there: {}"),
            conflict_confirm: s("{} Your text is kept. Press Save again to write it anyway."),
            tile_failed: s("A tile could not be read: {}"),
            tiles_more: s("and {} more"),
            too_many: s("{} shapes are left out: there are too many to draw at once."),
            too_wide: s("Too much of the map is in view to draw it all: zoom in."),
            no_tiles: s("There is no map here, only places."),
            picked: s("Picked {}."),
            more_problems: s("and {} more places could not be shown"),
        }
    }
}

impl MapLabels {
    /// `template` with its `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}
