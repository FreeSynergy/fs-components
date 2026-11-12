//! Where the layout of a list is kept so that it survives a restart.
//!
//! Pattern: Strategy (`LayoutStore`) with one adapter (`FileStore`) — ADR 0069: what is restored is the choice, keyed by
//! the view (never a position). The real `Restorable` of the shell comes later and implements this trait; until then
//! the file store keeps one small text file per view. A store that keeps nothing is `NoStore` (persistence off).

use std::path::{Path, PathBuf};

/// Keeps and gives back the text of a layout by the key of a view.
pub trait LayoutStore: Send + Sync {
    /// The text kept for this view, if there is one.
    fn load(&self, view: &str) -> Option<String>;
    /// Keep this text for this view.
    fn save(&self, view: &str, text: &str);
}

/// A store that keeps nothing: the layout is gone at the restart.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoStore;

impl LayoutStore for NoStore {
    fn load(&self, _view: &str) -> Option<String> {
        None
    }
    fn save(&self, _view: &str, _text: &str) {}
}

/// One file per view in a folder.
#[derive(Clone, Debug)]
pub struct FileStore(PathBuf);

impl FileStore {
    /// A store in this folder (made when the first layout is kept).
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self(dir.as_ref().to_path_buf())
    }

    fn file(&self, view: &str) -> PathBuf {
        // A view key is a name, never a path: what is not a letter, digit, dash or dot becomes an underscore.
        let safe: String = view
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || "-._".contains(c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.0.join(format!("columns-{safe}.txt"))
    }
}

impl LayoutStore for FileStore {
    fn load(&self, view: &str) -> Option<String> {
        std::fs::read_to_string(self.file(view)).ok()
    }

    fn save(&self, view: &str, text: &str) {
        if std::fs::create_dir_all(&self.0).is_ok() {
            let aside = self.file(view).with_extension("tmp");
            if std::fs::write(&aside, text).is_ok() {
                let _ = std::fs::rename(aside, self.file(view));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list_view::ColumnLayout;

    fn dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("fs-columns-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn defaults() -> ColumnLayout {
        ColumnLayout::new(&["name", "size", "modified"]).keep_shown("name")
    }

    #[test]
    fn order_hidden_and_width_survive_the_restart() {
        let d = dir("restart");
        let store = FileStore::new(&d);
        let mut l = defaults();
        l.toggle("size");
        l.move_by("modified", -1);
        l.set_width("name", 250.0);
        store.save("files/list", &l.to_text());
        // the restart: a new store over the same folder, a new layout from the caller's defaults
        let again = FileStore::new(&d);
        let back = defaults().with_text(&again.load("files/list").unwrap());
        assert_eq!(back, l);
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn counter_test_with_no_store_nothing_survives() {
        let mut l = defaults();
        l.toggle("size");
        NoStore.save("files/list", &l.to_text());
        let back = NoStore
            .load("files/list")
            .map_or_else(defaults, |t| defaults().with_text(&t));
        assert_ne!(
            back, l,
            "with persistence off the choice is gone: the test must be red if it were kept"
        );
        assert_eq!(back, defaults());
    }

    #[test]
    fn a_view_key_cannot_leave_the_folder() {
        let d = dir("path");
        let store = FileStore::new(&d);
        store.save("../../evil", "x");
        assert!(store.load("../../evil").is_some());
        assert!(
            d.join("columns-.._.._evil.txt").exists() || d.join("columns-.._.._evil.txt").is_file()
        );
        let _ = std::fs::remove_dir_all(d);
    }
}
