//! The file manager: folders, a list or a grid of what is in them, one thing selected and shown, the
//! trash and transfers; and a compact picker that gives back a value.
//!
//! Pattern: none (declares modules and re-exports only) — [`FileManagerWidget`] and [`FilePicker`] are what a host places. The workbench
//! knows no store and no program core, only [`FileSource`], and every word comes from [`FileLabels`].
//! The state machine [`FilesState`] works without a window. A workbench does not run alone (ADR 0056):
//! it has no window function, and gives its host what is current as a plain [`FilesContext`].

mod ask;
mod carry;
mod command;
mod confirm;
mod context;
mod error;
mod filter;
mod input;
mod labels;
mod labels_default;
mod labels_fill;
mod model;
mod navigate;
mod picker;
mod reading;
mod reports;
mod select;
mod side;
mod source;
mod state;
mod transfers;
mod tree;
mod view;
mod widget;
mod widget_actions;

pub use ask::{Ask, Confirm, InputAsk, InputKind, NameTaken};
pub use carry::{CarryKind, Carrying};
pub use command::Command;
pub use context::{FileView, FilesContext, PickedFile, Variant};
pub use error::{FileError, FileProblem};
pub use filter::FilterDraft;
pub use fs_components_widgets::file_preview::{Fact, PreviewContent};
pub use labels::FileLabels;
pub use model::{
    EntryKind, EntryPage, EntryRef, FileEntry, FolderInfo, ListFilter, ListRequest, NameRule,
    PageMark, Scope, SortKey, SortOrder,
};
pub use picker::FilePicker;
pub use reports::{
    EmptyReport, FileCapabilities, PendingUpload, PreviewBytes, RescanSummary, TrashCount,
};
pub use side::{Place, Side, TransferRow};
pub use source::FileSource;
pub use state::{FilesState, Notice, PAGE};
pub use view::styles;
pub use widget::FileManagerWidget;
