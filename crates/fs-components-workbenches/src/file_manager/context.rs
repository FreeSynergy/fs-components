//! What the workbench gives back to its host.
//!
//! Pattern: Value objects — a workbench always gives a context back and does not run alone (ADR 0056,
//! point 4). Provisional shape: what is current, as plain values. Nothing here is a handle.

/// How the things in a folder are shown.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileView {
    /// One row each.
    #[default]
    List,
    /// A tile each.
    Grid,
}

/// The whole workbench, or the compact picker that gives back a value (ADR 0037).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    /// Everything the source can do.
    #[default]
    Full,
    /// Look and choose one file; nothing is changed, whatever the source can do.
    Picker,
}

/// What is current in the workbench, for its host.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilesContext {
    /// The current folder; `None` = the top.
    pub folder: Option<String>,
    /// The trash is shown instead of a folder.
    pub in_trash: bool,
    /// The key of what is selected.
    pub selected: Option<String>,
    /// The view shown.
    pub view: FileView,
}

/// What the picker gives back.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickedFile {
    /// The key.
    pub key: String,
    /// The real name.
    pub name: String,
    /// It is marked sensitive and the person chose it knowing that (ADR 0029).
    pub marked: bool,
}
