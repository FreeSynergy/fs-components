//! Fake document sources for the tests.
//!
//! Pattern: Strategy (test doubles) — `Fake` is read-only, `Editable` keeps texts in memory with
//! a revision counter per text, like a store would.

#![allow(dead_code)] // each test file uses a different subset

pub mod draw;

use std::collections::BTreeMap;
use std::sync::Mutex;

use fs_components_workbenches::{
    Block, Document, DocumentSource, EditableText, Entry, Hit, Inline, LinkTarget, SourceError,
};

/// Documents `a`, `dir/b`, and `broken` (which cannot be read).
pub struct Fake;

impl DocumentSource for Fake {
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(["a", "broken", "dir/b"]
            .iter()
            .filter(|p| p.starts_with(prefix))
            .map(|p| Entry::new(*p, p.to_uppercase()))
            .collect())
    }

    fn read(&self, path: &str) -> Result<Document, SourceError> {
        match path {
            "broken" => Err(SourceError::failed("the file is damaged")),
            "a" | "dir/b" => Ok(Document::new(
                path,
                vec![Block::Paragraph(Inline::plain(format!("text of {path}")))],
            )),
            _ => Err(SourceError::not_found("unknown")),
        }
    }

    fn resolve(&self, from: &str, href: &str) -> LinkTarget {
        match href {
            "up" => LinkTarget::Document("a".into()),
            "web" => LinkTarget::External("https://example.org".into()),
            other => LinkTarget::Missing(format!("{from} -> {other}")),
        }
    }

    fn find(&self, term: &str) -> Result<Vec<Hit>, SourceError> {
        if term == "boom" {
            return Err(SourceError::failed("the index is gone"));
        }
        Ok(self
            .entries("")?
            .into_iter()
            .filter(|e| e.title.to_lowercase().contains(&term.to_lowercase()))
            .map(|e| Hit::new(e.path, e.title))
            .collect())
    }

    fn start(&self) -> Option<String> {
        Some("dir/b".into())
    }
}

/// A source that can be edited and exported; keeps its texts in memory.
#[derive(Default)]
pub struct Editable {
    /// path -> (text, revision)
    pub texts: Mutex<BTreeMap<String, (String, u32)>>,
    pub exported: Mutex<Vec<(String, String, bool)>>,
    /// A source without revisions: `save` cannot tell that someone else changed a text.
    pub no_revisions: std::sync::atomic::AtomicBool,
}

impl Editable {
    pub fn with(pages: &[(&str, &str)]) -> Self {
        let s = Self::default();
        for (p, t) in pages {
            s.texts
                .lock()
                .unwrap()
                .insert((*p).into(), ((*t).into(), 1));
        }
        s
    }

    /// Another person saves behind the reader's back.
    pub fn other_saves(&self, path: &str, text: &str) {
        let mut texts = self.texts.lock().unwrap();
        let entry = texts.get_mut(path).unwrap();
        entry.0 = text.into();
        entry.1 += 1;
    }

    pub fn text(&self, path: &str) -> String {
        self.texts.lock().unwrap()[path].0.clone()
    }
}

impl DocumentSource for Editable {
    fn entries(&self, _prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(self
            .texts
            .lock()
            .unwrap()
            .keys()
            .map(|p| Entry::new(p.clone(), p.clone()))
            .collect())
    }
    fn read(&self, path: &str) -> Result<Document, SourceError> {
        let t = self.source_text(path)?;
        let mut doc = Document::new(path, vec![Block::Paragraph(Inline::plain(t.text))]);
        doc.revision = t.revision;
        Ok(doc)
    }
    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        LinkTarget::Missing(href.into())
    }
    fn find(&self, term: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(self
            .texts
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, (t, _))| t.contains(term))
            .map(|(p, _)| Hit::new(p.clone(), p.clone()))
            .collect())
    }
    fn can_edit(&self, path: &str) -> bool {
        path != "locked"
    }
    fn source_text(&self, path: &str) -> Result<EditableText, SourceError> {
        self.texts
            .lock()
            .unwrap()
            .get(path)
            .map(|(t, r)| {
                let rev = (!self.no_revisions.load(std::sync::atomic::Ordering::Relaxed))
                    .then(|| r.to_string());
                EditableText::new(t.clone(), rev)
            })
            .ok_or_else(|| SourceError::not_found(format!("no document {path}")))
    }
    fn save(&self, path: &str, text: &str, revision: Option<&str>) -> Result<(), SourceError> {
        if text.contains("FORBIDDEN") {
            return Err(SourceError::failed("refused"));
        }
        let mut texts = self.texts.lock().unwrap();
        let entry = texts
            .get_mut(path)
            .ok_or_else(|| SourceError::not_found("gone"))?;
        let checked = !self.no_revisions.load(std::sync::atomic::Ordering::Relaxed);
        if checked && revision != Some(entry.1.to_string().as_str()) {
            return Err(SourceError::conflict("Someone else changed this document."));
        }
        entry.0 = text.into();
        entry.1 += 1;
        Ok(())
    }
    fn can_export(&self, _path: &str) -> bool {
        true
    }
    fn export(&self, path: &str, target: &str, overwrite: bool) -> Result<String, SourceError> {
        let mut done = self.exported.lock().unwrap();
        if !overwrite && done.iter().any(|(p, t, _)| p == path && t == target) {
            return Err(SourceError::conflict(format!("{target}/{path} exists.")));
        }
        done.push((path.into(), target.into(), overwrite));
        Ok(format!("Exported {path} to {target}."))
    }
}

/// A source whose list fails.
pub struct Failing;

impl DocumentSource for Failing {
    fn entries(&self, _prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Err(SourceError::failed("the list is unreachable"))
    }
    fn read(&self, _path: &str) -> Result<Document, SourceError> {
        Err(SourceError::not_found("none"))
    }
    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        LinkTarget::Missing(href.into())
    }
    fn find(&self, _term: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(vec![])
    }
}

/// A draft store in memory; `fail` makes every call fail.
#[derive(Default)]
pub struct MemDrafts {
    pub drafts: Mutex<BTreeMap<String, fs_components_workbenches::Draft>>,
    pub fail: std::sync::atomic::AtomicBool,
}

impl MemDrafts {
    fn check(&self) -> Result<(), SourceError> {
        if self.fail.load(std::sync::atomic::Ordering::Relaxed) {
            Err(SourceError::failed("the disk is full"))
        } else {
            Ok(())
        }
    }
    /// Put a draft under `key` although it names another document (a store with lossy keys).
    pub fn plant(&self, key: &str, draft: fs_components_workbenches::Draft) {
        self.drafts.lock().unwrap().insert(key.into(), draft);
    }
    pub fn has(&self, path: &str) -> bool {
        self.drafts.lock().unwrap().contains_key(path)
    }
}

impl fs_components_workbenches::DraftStore for MemDrafts {
    fn save(&self, draft: &fs_components_workbenches::Draft) -> Result<(), SourceError> {
        self.check()?;
        self.drafts
            .lock()
            .unwrap()
            .insert(draft.path.clone(), draft.clone());
        Ok(())
    }
    fn load(&self, path: &str) -> Result<Option<fs_components_workbenches::Draft>, SourceError> {
        self.check()?;
        Ok(self.drafts.lock().unwrap().get(path).cloned())
    }
    fn clear(&self, path: &str) -> Result<(), SourceError> {
        self.check()?;
        self.drafts.lock().unwrap().remove(path);
        Ok(())
    }
}
