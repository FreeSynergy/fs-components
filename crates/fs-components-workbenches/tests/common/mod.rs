//! A fake data source for the tests.
//!
//! Pattern: Strategy (test double) — implements `DataSource` over a list in memory.

#![allow(dead_code)] // each test file uses a different subset

use fs_components_workbenches::{
    Block, DataSource, Document, Entry, Hit, Inline, LinkTarget, ReadError,
};

/// Pages `a`, `dir/b`, and `broken` (which cannot be read).
pub struct Fake;

impl DataSource for Fake {
    fn entries(&self, prefix: &str) -> Result<Vec<Entry>, ReadError> {
        Ok(["a", "broken", "dir/b"]
            .iter()
            .filter(|p| p.starts_with(prefix))
            .map(|p| Entry {
                path: (*p).into(),
                title: p.to_uppercase(),
            })
            .collect())
    }

    fn read(&self, path: &str) -> Result<Document, ReadError> {
        match path {
            "broken" => Err(ReadError("the file is damaged".into())),
            "a" | "dir/b" => Ok(Document {
                title: path.into(),
                blocks: vec![Block::Paragraph(Inline::plain(format!("text of {path}")))],
            }),
            _ => Err(ReadError("unknown".into())),
        }
    }

    fn resolve(&self, from: &str, href: &str) -> LinkTarget {
        match href {
            "up" => LinkTarget::Page("a".into()),
            "web" => LinkTarget::External("https://example.org".into()),
            other => LinkTarget::Missing(format!("{from} -> {other}")),
        }
    }

    fn find(&self, term: &str) -> Result<Vec<Hit>, ReadError> {
        if term == "boom" {
            return Err(ReadError("the index is gone".into()));
        }
        Ok(self
            .entries("")?
            .into_iter()
            .filter(|e| e.title.to_lowercase().contains(&term.to_lowercase()))
            .map(|e| Hit {
                path: e.path,
                title: e.title,
            })
            .collect())
    }

    fn start(&self) -> Option<String> {
        Some("dir/b".into())
    }
}

/// A source that can be edited and exported; keeps its texts in memory.
#[derive(Default)]
pub struct Editable {
    pub texts: std::sync::Mutex<std::collections::BTreeMap<String, String>>,
    pub exported: std::sync::Mutex<Vec<String>>,
}

impl Editable {
    pub fn with(pages: &[(&str, &str)]) -> Self {
        let s = Self::default();
        for (p, t) in pages {
            s.texts.lock().unwrap().insert((*p).into(), (*t).into());
        }
        s
    }
}

impl DataSource for Editable {
    fn entries(&self, _prefix: &str) -> Result<Vec<Entry>, ReadError> {
        Ok(self
            .texts
            .lock()
            .unwrap()
            .keys()
            .map(|p| Entry {
                path: p.clone(),
                title: p.clone(),
            })
            .collect())
    }
    fn read(&self, path: &str) -> Result<Document, ReadError> {
        let text = self.source_text(path)?;
        Ok(Document {
            title: path.into(),
            blocks: vec![Block::Paragraph(Inline::plain(text))],
        })
    }
    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        LinkTarget::Missing(href.into())
    }
    fn find(&self, _term: &str) -> Result<Vec<Hit>, ReadError> {
        Ok(Vec::new())
    }
    fn can_edit(&self) -> bool {
        true
    }
    fn source_text(&self, path: &str) -> Result<String, ReadError> {
        self.texts
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| ReadError(format!("no page {path}")))
    }
    fn save(&self, path: &str, text: &str) -> Result<(), ReadError> {
        if text.contains("FORBIDDEN") {
            return Err(ReadError("refused".into()));
        }
        self.texts.lock().unwrap().insert(path.into(), text.into());
        Ok(())
    }
    fn can_export(&self) -> bool {
        true
    }
    fn export(&self, path: &str) -> Result<String, ReadError> {
        self.exported.lock().unwrap().push(path.into());
        Ok(format!("Exported {path}."))
    }
}
