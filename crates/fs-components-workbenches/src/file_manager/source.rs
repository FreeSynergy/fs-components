//! Where the file manager gets its files.
//!
//! Pattern: Strategy — the workbench depends on this trait only (an Adapter over a program's core
//! implements it). The trait is synchronous: the workbench calls the long ones (rescan, upload, download, going on
//! with an upload) from a worker, so the window stays free (`jobs.rs`; one at a time), and the short ones on the
//! interface thread, where a slow source holds the window for its time limit (DESIGN.md). Paths are text the person
//! typed: the source
//! checks them, the workbench does not look inside.

use super::error::FileError;
use super::model::{EntryPage, EntryRef, FileEntry, FolderInfo, ListRequest, NameRule, PageMark};
use super::reports::{
    EmptyReport, FileCapabilities, PendingUpload, PreviewBytes, RescanSummary, TrashCount,
};

/// A source of files. Every method answers [`FileError`] with a reason a person can read; a source
/// says [`FileProblem::Unsupported`](super::FileProblem::Unsupported) for what it does not offer.
pub trait FileSource: Send + Sync {
    /// What it can do.
    fn capabilities(&self) -> FileCapabilities;

    /// A page of what `request` asks for, starting at `after`. A page may be short or empty and still
    /// have a next one.
    ///
    /// # Errors
    /// [`FileError`]; one thing that cannot be read goes in [`EntryPage::unreadable`] instead.
    fn list(&self, request: &ListRequest, after: Option<&PageMark>)
    -> Result<EntryPage, FileError>;

    /// One thing as it is now, also in the trash; `None` if it is gone.
    ///
    /// # Errors
    /// [`FileError`].
    fn entry(&self, reference: &EntryRef) -> Result<Option<FileEntry>, FileError>;

    /// One folder; `None` if it is gone.
    ///
    /// # Errors
    /// [`FileError`].
    fn folder(&self, key: &str) -> Result<Option<FolderInfo>, FileError>;

    /// The folders directly in `parent` (`None` = the top), at most `limit`, and whether there are more.
    ///
    /// # Errors
    /// [`FileError`].
    fn subfolders(
        &self,
        parent: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<FolderInfo>, bool), FileError>;

    /// The start of a file, at most `limit` bytes. A source refuses a file that changed outside with
    /// [`FileProblem::ChangedOutside`](super::FileProblem::ChangedOutside).
    ///
    /// # Errors
    /// [`FileError`].
    fn preview(&self, key: &str, limit: usize) -> Result<PreviewBytes, FileError>;

    /// A new folder in `parent`.
    ///
    /// # Errors
    /// [`FileError`]; nothing was made.
    fn create_folder(&self, parent: Option<&str>, name: &str) -> Result<FolderInfo, FileError>;

    /// Give it another name; answers the name it has now.
    ///
    /// # Errors
    /// [`FileError`]; nothing changed.
    fn rename(&self, entry: &EntryRef, name: &str, rule: NameRule) -> Result<String, FileError>;

    /// Move it to `folder`; answers the name it has now.
    ///
    /// # Errors
    /// [`FileError`]; nothing changed.
    fn move_to(
        &self,
        entry: &EntryRef,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError>;

    /// A copy of a file in `folder`; answers the name of the copy.
    ///
    /// # Errors
    /// [`FileError`]; nothing was made.
    fn copy_to(&self, key: &str, folder: Option<&str>, rule: NameRule)
    -> Result<String, FileError>;

    /// Put a file in the trash.
    ///
    /// # Errors
    /// [`FileError`]; it stays.
    fn trash(&self, key: &str) -> Result<(), FileError>;

    /// Delete a folder, which must be empty.
    ///
    /// # Errors
    /// [`FileError`]; it stays.
    fn delete_folder(&self, key: &str) -> Result<(), FileError>;

    /// Bring a file back to where it was; answers the name it has now.
    ///
    /// # Errors
    /// [`FileError`]; it stays in the trash.
    fn restore(&self, key: &str, rule: NameRule) -> Result<String, FileError>;

    /// Bring a file back to `folder`; answers the name it has now.
    ///
    /// # Errors
    /// [`FileError`]; it stays in the trash.
    fn restore_to(
        &self,
        key: &str,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError>;

    /// Delete a file in the trash for good.
    ///
    /// # Errors
    /// [`FileError`]; it stays in the trash.
    fn purge(&self, key: &str) -> Result<(), FileError>;

    /// How much is in the trash.
    ///
    /// # Errors
    /// [`FileError`].
    fn trash_count(&self) -> Result<TrashCount, FileError>;

    /// Delete all that is in the trash for good.
    ///
    /// # Errors
    /// [`FileError`]; what could be purged by then is gone.
    fn empty_trash(&self) -> Result<EmptyReport, FileError>;

    /// Bring in what changed outside, below `folder`. Deletes nothing.
    ///
    /// # Errors
    /// [`FileError`].
    fn rescan(&self, folder: Option<&str>) -> Result<RescanSummary, FileError>;

    /// Upload the file at `path` into `folder`; answers the name it has now. Blocks until it is done.
    ///
    /// # Errors
    /// [`FileError`]; what arrived so far stays as an unfinished upload.
    fn upload(&self, path: &str, folder: Option<&str>, rule: NameRule)
    -> Result<String, FileError>;

    /// The unfinished uploads, oldest first.
    ///
    /// # Errors
    /// [`FileError`].
    fn pending_uploads(&self) -> Result<Vec<PendingUpload>, FileError>;

    /// Go on with an unfinished upload from the file at `path`; answers the name it has now.
    ///
    /// # Errors
    /// [`FileError`]; what arrived stays.
    fn resume_upload(&self, id: &str, path: &str) -> Result<String, FileError>;

    /// Throw an unfinished upload and what arrived of it away.
    ///
    /// # Errors
    /// [`FileError`]; it stays.
    fn drop_upload(&self, id: &str) -> Result<(), FileError>;

    /// Write a file to `path`, which must not exist yet; answers where it went. A marked file is
    /// refused with [`FileProblem::MarkedNeedsRelease`](super::FileProblem::MarkedNeedsRelease) unless
    /// `release_marked` is set, which only an explicit act of the person may do.
    ///
    /// # Errors
    /// [`FileError`]; nothing was written.
    fn download(&self, key: &str, path: &str, release_marked: bool) -> Result<String, FileError>;
}
