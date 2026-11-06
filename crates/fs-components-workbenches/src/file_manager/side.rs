//! Small states of the workbench: where the person is, what the side area shows, one transfer.
//!
//! Pattern: State — plain enums and values the state machine moves between.

use fs_components_widgets::transfer_list::{Direction, State as TransferState, Transfer};

/// Where the person is.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// In this folder; `None` = the top.
    Folder(Option<String>),
    /// In the trash.
    Trash,
}

/// What the area beside the list shows.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    /// What is selected.
    #[default]
    Item,
    /// Uploads and downloads.
    Transfers,
    /// The filter.
    Filter,
    /// What the last rescan found.
    Report,
}

/// One row of the transfers, and what is needed to go on with it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferRow {
    /// What the list shows.
    pub shown: Transfer,
    /// For an unfinished upload: the id to go on with.
    pub pending: Option<String>,
    /// For one that failed: the path it used, to try again.
    pub path: Option<String>,
    /// For a download that failed: the file.
    pub file: Option<String>,
}

impl TransferRow {
    pub(super) fn upload(key: String, name: &str, path: &str, state: TransferState) -> Self {
        Self {
            shown: Transfer::new(key, name, Direction::Upload).with_state(state),
            pending: None,
            path: Some(path.to_string()),
            file: None,
        }
    }

    pub(super) fn download(key: String, name: &str, file: &str, state: TransferState) -> Self {
        Self {
            shown: Transfer::new(key, name, Direction::Download).with_state(state),
            pending: None,
            path: None,
            file: Some(file.to_string()),
        }
    }
}
