//! `list-view`: rows and columns given by the caller, as many as the caller has.
//!
//! Pattern: Strategy ([`RowSource`] supplies the rows) + Adapter over Blinc's scroll container and
//! keyed rows. Only the rows in view exist as elements; they are read a page at a time through
//! the source, so a list of 100,000 rows costs what a list of 40 costs. The widget knows nothing
//! about what its rows mean and has no words or colours of its own.

mod apply;
mod cache;
mod cell;
mod column;
mod head;
mod keys;
mod layout;
mod layout_store;
mod model;
mod offset;
mod position;
mod styles;
mod view;
mod window;

pub use cell::fit_cell;
pub use column::{Column, ColumnWidth};
pub use keys::key_target;
pub use layout::{ColumnLayout, LEAST_WIDTH};
pub use layout_store::{FileStore, LayoutStore, NoStore};
pub use model::{ListView, OnLayout, OnSelect, OnSort, Row, RowSource, VecRows};
pub use position::ListPosition;
pub use styles::STYLES;
pub use window::{DEFAULT_ROW_HEIGHT, column_widths, visible_rows};
