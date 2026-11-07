//! `avatar-group`: several people in little space.
//!
//! Pattern: Adapter over `blinc_cn`'s avatar group (the overlap, the ring and the "+N" circle
//! are Blinc's). What is ours: the neutral [`Person`] (no program's contact type), initials that
//! are safe for any name, and the rule that the people who do not fit are counted, never dropped.

mod model;
mod styles;
mod view;

pub use model::{AvatarGroup, MAX_SHOWN, Person, Presence, Size, initials};
pub use styles::STYLES;
