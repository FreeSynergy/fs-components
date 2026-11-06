//! `fs-components-workbenches`: workbenches (layer 3) of FreeSynergy.
//!
//! A workbench receives its content only at run time and knows nothing about what it means. The
//! [`reader`] is an open surface that lists, opens, follows, searches and edits documents of any
//! [`DocumentSource`]. [`ReaderWidget`] is the reader to place in a program's own window (ADR
//! 0005, 0037); [`launch`] makes a bare app (ADR 0056: an app is a workbench over the whole screen; this one has no menu and no other parts, so it is for trying and for the simplest program; a real app uses `fs-components-app`), [`snapshot`] a picture drawn without a
//! window. Every text it shows comes from the caller ([`Labels`]).
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod calendar;
mod error;
mod labels;
pub mod llm_chat;
pub mod people;
pub mod reader;
pub mod shell;
mod source;
pub mod tasks;
mod view;

pub use error::{ErrorKind, SourceError};
pub use labels::Labels;
pub use reader::{Draft, DraftStore, Editing, Outcome, Reader, Shown};
pub use shell::{
    Options, Picture, ReaderWidget, ScrollInfo, ShellError, Snapshot, Step, launch, register_fonts,
    snapshot, styles,
};
pub use source::{DocumentSource, EditableText, Entry, Hit, LinkTarget};

/// The documents the reader shows, in the neutral model of the widgets.
pub use fs_components_widgets::{Align, Block, Document, Inline, ListItem, Mark, MarkKind};
