//! The state of the timeline: zoom, selection, today, and what a key does.
//!
//! Pattern: State — one struct with transitions; pure, no Blinc. Buttons, clicks and keys end in
//! the same transitions.

use fs_components_widgets::Key;

use crate::model::{Day, Item, Zoom};

/// What a step came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Nothing happened.
    Ignored,
    /// The zoom changed.
    Zoomed(Zoom),
    /// An item was chosen.
    Select(String),
}

/// Everything the timeline remembers.
#[derive(Clone, Debug)]
pub struct TimelineState {
    /// Zoom level.
    pub zoom: Zoom,
    /// Chosen item.
    pub selected: Option<String>,
    /// Today, drawn as a line; `None` draws none.
    pub today: Option<Day>,
}

impl TimelineState {
    /// Zoomed to weeks, nothing chosen.
    pub fn new(today: Option<Day>) -> Self {
        Self {
            zoom: Zoom::Week,
            selected: None,
            today,
        }
    }

    /// Set the zoom.
    pub fn set_zoom(&mut self, zoom: Zoom) -> Step {
        if zoom == self.zoom {
            return Step::Ignored;
        }
        self.zoom = zoom;
        Step::Zoomed(zoom)
    }

    /// Choose an item.
    pub fn select(&mut self, key: &str) -> Step {
        self.selected = Some(key.to_owned());
        Step::Select(key.to_owned())
    }

    /// A key: Up/Down walk the rows, `+` and `-` zoom (also on the number pad).
    pub fn key(&mut self, items: &[Item], key: Key) -> Step {
        let at = self
            .selected
            .as_deref()
            .and_then(|k| items.iter().position(|i| i.key == k));
        let to = match (key, at) {
            (Key::Down, None) | (Key::Home, _) => Some(0),
            (Key::Up, None) | (Key::End, _) => items.len().checked_sub(1),
            (Key::Down, Some(i)) => Some((i + 1).min(items.len().saturating_sub(1))),
            (Key::Up, Some(i)) => Some(i.saturating_sub(1)),
            (Key::Other(187 | 107), _) => return self.set_zoom(self.zoom.finer()),
            (Key::Other(189 | 109), _) => return self.set_zoom(self.zoom.coarser()),
            _ => return Step::Ignored,
        };
        match to.and_then(|i| items.get(i)) {
            Some(item) if Some(item.key.as_str()) != self.selected.as_deref() => {
                self.select(&item.key.clone())
            }
            _ => Step::Ignored,
        }
    }
}
