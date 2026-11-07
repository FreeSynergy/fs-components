//! `share-dialog`: choose people or groups and a permission for each. Only the dialog.
//!
//! Pattern: State + Command + Adapter, like `sortable-list`. There is no Sharing interface yet
//! (`Sharing` in the contracts is an empty placeholder), so this widget invents no permission
//! model and stores no rights: the caller hands over the list of choices and the names of the
//! permissions, and gets one value back, [`Outcome::Confirmed`], with the pairs the person
//! picked. What a permission means, and where it is saved, is the caller's.

mod bindings;
mod handle;
mod model;
mod state;
mod styles;
mod view;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, ShareDialog};
pub use model::{Choice, Grant, Outcome, Permission, Refusal};
pub use state::Sharing;
pub use styles::STYLES;
