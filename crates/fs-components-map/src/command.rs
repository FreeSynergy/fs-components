//! Pan and zoom as values: what a key, a button or a frontend's gesture asks of the view.
//!
//! Pattern: Command — an enum of steps and one pure `apply`; keys and buttons are mapped to it here,
//! gestures (a drag, a wheel, a pinch) are mapped by the frontend (ADR 0037) and arrive as the same
//! values.

use super::position::MapPosition;
use fs_components_widgets::{Key, KeyEvent};

/// Pixels one key press or button press moves the view.
pub const PAN_STEP: f64 = 96.0;
/// Zoom steps one press of a zoom key or button makes.
pub const ZOOM_STEP: f64 = 1.0;

/// A step of the view.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MapCommand {
    /// Move the centre by this many pixels east and south.
    Pan(f64, f64),
    /// Zoom by this many steps around the centre.
    Zoom(f64),
    /// Zoom by this many steps keeping the point at `(x, y)` where it is: a wheel or a pinch.
    ZoomAt(f64, f64, f64),
}

impl MapCommand {
    /// The view after this step.
    #[must_use]
    pub fn apply(self, position: MapPosition, width: f64, height: f64) -> MapPosition {
        match self {
            Self::Pan(dx, dy) => position.panned_by(dx, dy),
            Self::Zoom(steps) => position.zoomed_by(steps),
            Self::ZoomAt(steps, x, y) => position.zoomed_at(steps, x, y, width, height),
        }
    }
}

/// What a key asks for, if anything: the arrows pan, Page Up and `+` zoom in, Page Down and `-` out.
#[must_use]
pub fn command_for_key(event: KeyEvent) -> Option<MapCommand> {
    if event.ctrl || event.alt {
        return None;
    }
    let step = if event.shift {
        PAN_STEP * 3.0
    } else {
        PAN_STEP
    };
    match event.key {
        Key::Left => Some(MapCommand::Pan(-step, 0.0)),
        Key::Right => Some(MapCommand::Pan(step, 0.0)),
        Key::Up => Some(MapCommand::Pan(0.0, -step)),
        Key::Down => Some(MapCommand::Pan(0.0, step)),
        // Page Up, `=`/`+` and numpad plus; Page Down, `-` and numpad minus (DOM key codes; 61
        // and 173 are Firefox's).
        Key::PageUp | Key::Other(187 | 107 | 61) => Some(MapCommand::Zoom(ZOOM_STEP)),
        Key::PageDown | Key::Other(189 | 109 | 173) => Some(MapCommand::Zoom(-ZOOM_STEP)),
        _ => None,
    }
}
