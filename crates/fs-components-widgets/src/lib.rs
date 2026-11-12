//! `fs-components-widgets`: widgets (layer 2) of FreeSynergy.
//!
//! A widget draws something whose data model is known when it is built. Here: [`text`] (the
//! neutral description of a document: blocks and marks), [`text_view`] (draws it) and
//! [`tree_view`] (draws a tree of labels). No widget knows what its data means, and no Blinc type
//! appears in a public signature except where a function must return an element to be placed in
//! a Blinc layout ([`Element`]).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod agent_tree;
pub mod avatar_group;
pub mod badge;
pub mod breadcrumb;
pub mod calendar_view;
pub mod craft;
pub mod drafts;
pub mod file_preview;
pub mod file_tile;
#[cfg(feature = "test-access")]
pub use fs_components_primitives::test_access;
pub mod icon;
mod icon_export;
mod icon_kind;
mod icon_named;
pub mod icon_theme;
pub mod keys;
pub mod lightbox;
pub mod list_view;
pub mod live;
pub mod message_composer;
pub mod message_list;
pub mod participant;
mod picture;
pub mod share_dialog;
mod softwrap;
pub mod sortable_list;
pub mod source_error;
mod styles;
pub mod task_card;
pub mod text;
pub mod text_view;
pub mod tool_call;
pub mod transfer_list;
pub mod tree_view;

pub use avatar_group::{AvatarGroup, Person, Presence};
pub use avatar_group::{
    MAX_SHOWN as AVATAR_GROUP_MAX_SHOWN, Size, initials as avatar_group_initials,
};
pub use badge::{badge_count, badge_dot};
pub use breadcrumb::{Breadcrumb, Crumb};
pub use calendar_view::{
    CalendarItem, CalendarNames, CalendarPosition, CalendarView, Day, Moment, Picked, ViewRange,
};
pub use file_preview::{Fact, FilePreview, PreviewContent, SafeText, picture_fits, safe_text};
pub use file_tile::{FileTile, TileGrid};
pub use icon::{Glyph, Tone, icon};
pub use icon_export::{export_theme, index_theme};
pub use icon_kind::{PinnedKind, pinned_icon};
pub use icon_named::{icon_named, set_icon_theme};
pub use icon_theme::{IconTheme, default_base_dirs};
pub use keys::{Handled, Key, KeyEvent};
pub use lightbox::{LightBox, Media};
pub use list_view::{
    Column, ColumnLayout, ColumnWidth, FileStore, LayoutStore, ListPosition, ListView, NoStore,
    Row, RowSource, VecRows,
};
pub use participant::{KindLabels, Sender, SenderKind};
pub use share_dialog::{Choice, Permission, ShareDialog};
pub use sortable_list::{Content as SortableContent, Item, Lane, SortableList};
pub use styles::styles;
pub use text::{Align, Block, Document, Inline, ListItem, Mark, MarkKind};
pub use text_view::{LinkHandler, TextView, fallback_family, missing_glyphs, open_external};
pub use tool_call::{ToolCall, ToolCallView};
pub use transfer_list::{Transfer, TransferList};
pub use tree_view::{TreeNode, TreeView};

/// What a widget returns to be placed in a layout: the one place a Blinc type is visible.
pub use blinc_layout::prelude::ElementBuilder as Element;
