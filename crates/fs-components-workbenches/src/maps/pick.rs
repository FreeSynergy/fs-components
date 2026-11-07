//! What the workbench gives back: its context, and in the compact variant a picked value.
//!
//! Pattern: Value objects out — a workbench always returns a context and never runs alone (ADR 0056
//! point 4); it hands a plain value to the program around it and calls no function of its window. A
//! picked place is only its key: of a marked place no name and no position leave with it (ADR 0029).

use super::model::{MapContext, Pick};
use super::shown::{Shown, Variant};
use super::state::MapState;

impl MapState {
    /// What the workbench is showing: the place open or being edited, and the view.
    pub fn context(&self) -> MapContext {
        MapContext {
            place: match &self.editing {
                Some(edit) => edit.key.clone(),
                None => self.selected().map(str::to_string),
            },
            position: self.position,
        }
    }

    /// In the compact variant, choosing a place is picking it.
    pub(super) fn pick_selected(&mut self) {
        if self.variant != Variant::Compact {
            return;
        }
        if let Shown::Place { key, sheet } = &self.shown {
            self.picked = Some(Pick::Place {
                key: key.clone(),
                marked: !sheet.marks.is_empty(),
            });
        }
    }

    /// In the compact variant: pick the centre of the map as a point.
    pub fn pick_centre(&mut self) {
        if self.variant == Variant::Compact {
            self.picked = Some(Pick::Point {
                lon: self.position.lon(),
                lat: self.position.lat(),
            });
        }
    }

    /// What was picked last, if anything.
    pub fn picked(&self) -> Option<&Pick> {
        self.picked.as_ref()
    }

    /// What was picked, handed over: it is forgotten here.
    pub fn take_picked(&mut self) -> Option<Pick> {
        self.picked.take()
    }
}
