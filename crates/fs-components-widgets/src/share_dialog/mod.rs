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
mod view;

pub use bindings::{Bindings, Command};
pub use handle::{Labels, ShareDialog};
pub use model::{Choice, Grant, Outcome, Permission, Refusal};
pub use state::Sharing;

/// CSS of the share dialog: only theme variables.
pub const STYLES: &str = r"
.fs-sd { display: flex; flex-direction: column; width: 100%; max-width: 560px; gap: var(--space-3); padding: var(--space-4); border-radius: var(--radius-lg); background: var(--surface); border-width: 1px; border-color: var(--border); }
.fs-sd-title { color: var(--text-primary); font-size: var(--text-lg); }
.fs-sd-list { display: flex; flex-direction: column; gap: var(--space-1); min-height: 0; }
.fs-sd-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); min-width: 0; overflow: hidden; padding: var(--space-2) var(--space-3); border-radius: var(--radius-default); border-width: 2px; border-color: var(--surface); }
.fs-sd-row-cursor { border-color: var(--border-focus); }
.fs-sd-row-on { background: var(--accent-subtle); }
.fs-sd-mark { flex-shrink: 0; width: 16px; height: 16px; border-radius: var(--radius-sm); border-width: 2px; border-color: var(--border); }
.fs-sd-mark-on { background: var(--primary); border-color: var(--primary); }
.fs-sd-pick { display: flex; flex-direction: row; align-items: center; gap: var(--space-3); flex-grow: 1; min-width: 0; overflow: hidden; }
.fs-sd-who { display: flex; flex-direction: column; flex-grow: 1; min-width: 0; overflow: hidden; }
.fs-sd-name { color: var(--text-primary); }
.fs-sd-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-sd-perms { display: flex; flex-direction: row; flex-wrap: wrap; justify-content: flex-end; align-items: center; gap: var(--space-1); flex-shrink: 1; max-width: 65%; }
.fs-sd-unknown { color: var(--error); font-size: var(--text-xs); }
.fs-sd-note { color: var(--text-secondary); font-size: var(--text-sm); padding: var(--space-1) var(--space-2); }
.fs-sd-foot { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
.fs-sd-buttons { display: flex; flex-direction: row; gap: var(--space-2); }
";
