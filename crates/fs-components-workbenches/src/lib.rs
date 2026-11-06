//! `fs-components-workbenches`: workbenches (layer 3) of FreeSynergy.
//!
//! A workbench receives its content only at run time and knows nothing about what it means. The
//! [`reader`] is an open surface that lists, opens, follows and searches documents of any
//! [`DataSource`]. [`shell`] puts it into a window (or into a picture, for looking at it without
//! a screen). Blinc types do not appear in this API.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod reader;
pub mod shell;
mod source;
mod view;

pub use reader::{Outcome, Reader, Shown};
pub use shell::{Options, ShellError, Step, launch, snapshot};
pub use source::{DataSource, Entry, Hit, LinkTarget, ReadError};

/// The documents the reader shows, in the neutral model of the widgets.
pub use fs_components_widgets::{Block, Document, Inline, ListItem, Mark, MarkKind};
