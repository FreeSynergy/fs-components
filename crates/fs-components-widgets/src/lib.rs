//! `fs-components-widgets`: widgets (layer 2) of FreeSynergy.
//!
//! A widget draws something whose data model is known when it is built. Here: [`text`] (the
//! neutral description of a document: blocks and marks), [`text_view`] (draws it) and
//! [`tree_view`] (draws a tree of labels). No widget knows what its data means, and no Blinc type
//! appears in a public signature except where a function must return an element to be placed in
//! a Blinc layout ([`Element`]).
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod avatar_group;
pub mod keys;
mod live;
pub mod sortable_list;
mod styles;
pub mod text;
pub mod text_view;
pub mod transfer_list;
pub mod tree_view;

pub use avatar_group::{AvatarGroup, Person, Presence};
pub use avatar_group::{
    MAX_SHOWN as AVATAR_GROUP_MAX_SHOWN, Size, initials as avatar_group_initials,
};
pub use keys::{Handled, Key, KeyEvent};
pub use sortable_list::{Item, Lane, SortableList};
pub use styles::styles;
pub use text::{Align, Block, Document, Inline, ListItem, Mark, MarkKind};
pub use text_view::{LinkHandler, TextView, open_external};
pub use transfer_list::{Transfer, TransferList};
pub use tree_view::{TreeNode, TreeView};

/// What a widget returns to be placed in a layout: the one place a Blinc type is visible.
pub use blinc_layout::prelude::ElementBuilder as Element;
