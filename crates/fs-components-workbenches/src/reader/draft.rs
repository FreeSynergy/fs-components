//! A draft as backed up, and where backups go (they live in the widgets crate, shared with the
//! message composer).
//!
//! Pattern: Facade — the old path keeps working for every program that names it.

pub use fs_components_widgets::drafts::{Draft, DraftStore};
