//! The one entry point of fs-components for programs.
//!
//! Blinc is pinned exactly once, in the workspace `Cargo.toml` of fs-components, as a git dependency on a tag of
//! our fork. Cargo applies `[patch]` only at the top of a build, so a program could never inherit one; a git
//! dependency is inherited. A program therefore depends on `fs-components` (and the `fs-components-*` crates it
//! needs), writes `use fs_components::blinc::layout::...`, and carries no `blinc_*` line and no `[patch]` line.

/// Blinc, as pinned by fs-components. Each module is the crate of the same name, without the `blinc_` prefix.
pub mod blinc {
    pub use blinc_animation as animation;
    pub use blinc_app as app;
    pub use blinc_cn as cn;
    pub use blinc_core as core;
    pub use blinc_layout as layout;
    pub use blinc_noto_emoji as noto_emoji;
    pub use blinc_theme as theme;
}
