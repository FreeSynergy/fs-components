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
mod styles;
mod view;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, LightBox};
pub use model::{Media, Outcome, Refusal, Viewer};
pub use styles::STYLES;
