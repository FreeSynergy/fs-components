//! The shelf as a file source.
//!
//! Pattern: Strategy (test double) — see the parent module.

use std::sync::Arc;

use fs_components_workbenches::file_manager::{
    EmptyReport, EntryPage, EntryRef, FileCapabilities, FileEntry, FileError, FileProblem,
    FileSource, FolderInfo, ListRequest, NameRule, PageMark, PendingUpload, PreviewBytes,
    RescanSummary, Scope, TrashCount,
};

use super::{Shelf, Thing, not_found};

impl FileSource for Shelf {
    fn capabilities(&self) -> FileCapabilities {
        FileCapabilities::all()
    }

    fn list(
        &self,
        request: &ListRequest,
        after: Option<&PageMark>,
    ) -> Result<EntryPage, FileError> {
        self.check()?;
        let start = after.map_or(0, |m| *m.0.downcast_ref::<usize>().unwrap());
        let size = *self.page_size.lock().unwrap();
        let all: Vec<FileEntry> = self
            .things
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| match &request.scope {
                Scope::Trash => t.trashed,
                Scope::Folder(f) => !t.trashed && t.folder == *f,
                _ => !t.trashed,
            })
            .filter(|(_, t)| t.name.contains(&request.term))
            .filter(|(_, t)| {
                request
                    .filter
                    .min_size
                    .is_none_or(|m| t.bytes.len() as u64 >= m)
            })
            .map(|(k, t)| Self::entry_of(k, t))
            .collect();
        let entries: Vec<_> = all.iter().skip(start).take(size).cloned().collect();
        let next = (start + size < all.len()).then(|| PageMark(Arc::new(start + size)));
        let mut page = EntryPage::new(entries, next);
        page.unreadable = self.unreadable.lock().unwrap().clone();
        Ok(page)
    }

    fn entry(&self, reference: &EntryRef) -> Result<Option<FileEntry>, FileError> {
        self.check()?;
        Ok(self
            .thing(&reference.key)
            .map(|t| Self::entry_of(&reference.key, &t)))
    }

    fn folder(&self, key: &str) -> Result<Option<FolderInfo>, FileError> {
        self.check()?;
        Ok(self
            .thing(key)
            .filter(|t| t.is_folder)
            .map(|t| FolderInfo::new(key, t.name, t.folder)))
    }

    fn subfolders(
        &self,
        parent: Option<&str>,
        limit: usize,
    ) -> Result<(Vec<FolderInfo>, bool), FileError> {
        self.check()?;
        let all: Vec<FolderInfo> = self
            .things
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| t.is_folder && t.folder.as_deref() == parent)
            .map(|(k, t)| FolderInfo::new(k.clone(), t.name.clone(), t.folder.clone()))
            .collect();
        let more = all.len() > limit;
        Ok((all.into_iter().take(limit).collect(), more))
    }

    fn preview(&self, key: &str, limit: usize) -> Result<PreviewBytes, FileError> {
        self.check()?;
        let thing = self.thing(key).ok_or_else(not_found)?;
        if thing.changed_outside {
            return Err(FileError::new(FileProblem::ChangedOutside, "size differs"));
        }
        let total = thing.bytes.len() as u64;
        Ok(PreviewBytes::new(
            thing.bytes.into_iter().take(limit).collect(),
            total,
        ))
    }

    fn create_folder(&self, parent: Option<&str>, name: &str) -> Result<FolderInfo, FileError> {
        self.check()?;
        let name = self.free_name(parent, name, "", NameRule::Refuse)?;
        let key = format!("new-{name}");
        self.things.lock().unwrap().insert(
            key.clone(),
            Thing {
                name: name.clone(),
                folder: parent.map(str::to_string),
                is_folder: true,
                trashed: false,
                marked: false,
                bytes: Vec::new(),
                changed_outside: false,
            },
        );
        self.wrote();
        Ok(FolderInfo::new(key, name, parent.map(str::to_string)))
    }

    fn rename(&self, entry: &EntryRef, name: &str, rule: NameRule) -> Result<String, FileError> {
        self.check()?;
        let thing = self.thing(&entry.key).ok_or_else(not_found)?;
        let name = self.free_name(thing.folder.as_deref(), name, &entry.key, rule)?;
        self.change(&entry.key, |t| t.name.clone_from(&name));
        self.wrote();
        Ok(name)
    }

    fn move_to(
        &self,
        entry: &EntryRef,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError> {
        self.check()?;
        let thing = self.thing(&entry.key).ok_or_else(not_found)?;
        let name = self.free_name(folder, &thing.name, &entry.key, rule)?;
        self.change(&entry.key, |t| {
            t.folder = folder.map(str::to_string);
            t.name.clone_from(&name);
        });
        self.wrote();
        Ok(name)
    }

    fn copy_to(
        &self,
        key: &str,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError> {
        self.check()?;
        let mut thing = self.thing(key).ok_or_else(not_found)?;
        let name = self.free_name(folder, &thing.name, "", rule)?;
        thing.name.clone_from(&name);
        thing.folder = folder.map(str::to_string);
        self.things
            .lock()
            .unwrap()
            .insert(format!("{key}-copy-{name}"), thing);
        self.wrote();
        Ok(name)
    }

    fn trash(&self, key: &str) -> Result<(), FileError> {
        self.check()?;
        self.thing(key).ok_or_else(not_found)?;
        self.change(key, |t| t.trashed = true);
        self.wrote();
        Ok(())
    }

    fn delete_folder(&self, key: &str) -> Result<(), FileError> {
        self.check()?;
        self.things
            .lock()
            .unwrap()
            .remove(key)
            .ok_or_else(not_found)?;
        self.wrote();
        Ok(())
    }

    fn restore(&self, key: &str, rule: NameRule) -> Result<String, FileError> {
        let thing = self.thing(key).ok_or_else(not_found)?;
        self.restore_to(key, thing.folder.as_deref(), rule)
    }

    fn restore_to(
        &self,
        key: &str,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError> {
        self.check()?;
        let thing = self.thing(key).ok_or_else(not_found)?;
        let name = self.free_name(folder, &thing.name, key, rule)?;
        self.change(key, |t| {
            t.folder = folder.map(str::to_string);
            t.trashed = false;
            t.name.clone_from(&name);
        });
        self.wrote();
        Ok(name)
    }

    fn purge(&self, key: &str) -> Result<(), FileError> {
        self.check()?;
        self.things
            .lock()
            .unwrap()
            .remove(key)
            .ok_or_else(not_found)?;
        self.wrote();
        Ok(())
    }

    fn trash_count(&self) -> Result<TrashCount, FileError> {
        self.check()?;
        let things = self.things.lock().unwrap();
        let files = things.values().filter(|t| t.trashed).count();
        let marked = things.values().filter(|t| t.trashed && t.marked).count();
        Ok(TrashCount::new(files, marked))
    }

    fn empty_trash(&self) -> Result<EmptyReport, FileError> {
        self.check()?;
        let mut things = self.things.lock().unwrap();
        let before = things.len();
        things.retain(|_, t| !t.trashed);
        self.wrote();
        Ok(EmptyReport::new(before - things.len(), Vec::new()))
    }

    fn rescan(&self, _folder: Option<&str>) -> Result<RescanSummary, FileError> {
        self.check()?;
        Ok(RescanSummary::default())
    }

    fn upload(
        &self,
        path: &str,
        folder: Option<&str>,
        rule: NameRule,
    ) -> Result<String, FileError> {
        self.check()?;
        let name = self.free_name(folder, path.rsplit('/').next().unwrap_or(path), "", rule)?;
        self.things.lock().unwrap().insert(
            format!("up-{name}"),
            Thing {
                name: name.clone(),
                folder: folder.map(str::to_string),
                is_folder: false,
                trashed: false,
                marked: false,
                bytes: b"uploaded".to_vec(),
                changed_outside: false,
            },
        );
        self.wrote();
        Ok(name)
    }

    fn pending_uploads(&self) -> Result<Vec<PendingUpload>, FileError> {
        self.check()?;
        Ok(self.pending.lock().unwrap().clone())
    }

    fn resume_upload(&self, id: &str, _path: &str) -> Result<String, FileError> {
        self.check()?;
        self.pending.lock().unwrap().retain(|p| p.id != id);
        self.wrote();
        Ok(id.to_string())
    }

    fn drop_upload(&self, id: &str) -> Result<(), FileError> {
        self.check()?;
        self.pending.lock().unwrap().retain(|p| p.id != id);
        self.wrote();
        Ok(())
    }

    fn download(&self, key: &str, path: &str, release_marked: bool) -> Result<String, FileError> {
        self.check()?;
        let thing = self.thing(key).ok_or_else(not_found)?;
        if thing.marked && !release_marked {
            return Err(FileError::new(FileProblem::MarkedNeedsRelease, "marked"));
        }
        self.downloads
            .lock()
            .unwrap()
            .push((key.into(), path.into(), release_marked));
        Ok(path.to_string())
    }
}
