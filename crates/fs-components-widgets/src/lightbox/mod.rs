//! `lightbox`: show one item large and step through the items the caller gives.
//!
//! Pattern: State + Command + Adapter, like `sortable-list`: a pure [`Viewer`] (which item is
//! shown, stepping, wrapping), optional [`Bindings`] for keys, and the [`LightBox`] handle with
//! the Blinc view. The box draws a backdrop, the picture (Blinc's image, fitted inside), a
//! caption, a counter and buttons; it does not open a window or an overlay: the frontend places
//! it (a modal at window size) and removes it when it hears [`Outcome::Closed`].

mod bindings;
mod handle;
mod model;
mod view;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, LightBox};
pub use model::{Media, Outcome, Refusal, Viewer};

/// CSS of the lightbox: only theme variables.
pub const STYLES: &str = r"
.fs-lb { display: flex; flex-direction: column; width: 100%; height: 100%; background: var(--surface-overlay); padding: var(--space-3); gap: var(--space-3); }
.fs-lb-bar { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
.fs-lb-count { color: var(--text-primary); }
.fs-lb-body { display: flex; flex-direction: row; align-items: center; flex-grow: 1; min-height: 0; gap: var(--space-3); }
.fs-lb-stage { display: flex; flex-direction: column; align-items: center; justify-content: center; flex-grow: 1; flex-basis: 0; min-width: 0; min-height: 0; height: 100%; overflow: hidden; }
.fs-lb-note { color: var(--text-secondary); padding: var(--space-3); }
.fs-lb-caption { color: var(--text-primary); text-align: center; }
";
