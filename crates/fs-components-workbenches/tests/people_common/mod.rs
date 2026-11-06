//! A people source for the tests of the workbench.
//!
//! Pattern: Strategy (test double) — people in memory with a revision per person, like a store
//! would; it stands for "any source", not for a program's core (that is tested in `fs-people`).
#![allow(dead_code)] // each test file uses a different subset

use std::collections::BTreeMap;
use std::sync::Mutex;

use fs_components_workbenches::SourceError;
use fs_components_workbenches::people::{
    DraftField, DraftGroup, PeopleCapabilities, PeopleSource, PersonDraft, PersonField, PersonRow,
    PersonSection, PersonSheet,
};

/// Name and e-mail per key, with a revision.
pub struct Book {
    pub people: Mutex<BTreeMap<String, (String, String, u32)>>,
    pub capabilities: PeopleCapabilities,
    /// A save, create or delete fails with this reason.
    pub refuse: Mutex<Option<String>>,
    /// The list fails with this reason.
    pub broken_list: Mutex<Option<String>>,
    pub writes: Mutex<u32>,
    /// Keys of people to be marked and opened only when chosen.
    pub guarded: Mutex<Vec<String>>,
    /// Said with a delete that went through.
    pub delete_remark: Mutex<Option<String>>,
    /// Lists fail from this offset on.
    pub broken_from: Mutex<Option<usize>>,
}

impl Book {
    pub fn with(names: &[&str]) -> Self {
        let people = names
            .iter()
            .map(|n| {
                (
                    (*n).to_string(),
                    ((*n).to_string(), format!("{n}@x.org"), 1),
                )
            })
            .collect();
        Self {
            people: Mutex::new(people),
            capabilities: PeopleCapabilities::all(),
            refuse: Mutex::new(None),
            broken_list: Mutex::new(None),
            writes: Mutex::new(0),
            guarded: Mutex::new(Vec::new()),
            delete_remark: Mutex::new(None),
            broken_from: Mutex::new(None),
        }
    }

    pub fn numbered(count: usize) -> Self {
        let names: Vec<String> = (0..count).map(|i| format!("p{i:05}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        Self::with(&names)
    }

    /// Another person changes `key` behind the workbench's back.
    pub fn other_changes(&self, key: &str, name: &str) {
        let mut people = self.people.lock().unwrap();
        let entry = people.get_mut(key).unwrap();
        entry.0 = name.into();
        entry.2 += 1;
    }

    /// Another person deletes `key` behind the workbench's back.
    pub fn other_deletes(&self, key: &str) {
        self.people.lock().unwrap().remove(key);
    }

    pub fn name(&self, key: &str) -> Option<String> {
        self.people.lock().unwrap().get(key).map(|p| p.0.clone())
    }

    pub fn writes(&self) -> u32 {
        *self.writes.lock().unwrap()
    }

    fn refused(&self) -> Result<(), SourceError> {
        match self.refuse.lock().unwrap().clone() {
            Some(why) => Err(SourceError::failed(why)),
            None => Ok(()),
        }
    }
}

fn value(draft: &PersonDraft, field: &str) -> String {
    draft
        .fields()
        .find(|f| f.key == field)
        .map(|f| f.value.clone())
        .unwrap_or_default()
}

impl PeopleSource for Book {
    fn capabilities(&self) -> PeopleCapabilities {
        self.capabilities
    }

    fn rows(&self, term: &str, offset: usize, limit: usize) -> Result<Vec<PersonRow>, SourceError> {
        if let Some(why) = self.broken_list.lock().unwrap().clone() {
            return Err(SourceError::failed(why));
        }
        if self
            .broken_from
            .lock()
            .unwrap()
            .is_some_and(|from| offset >= from)
        {
            return Err(SourceError::failed("the next page failed"));
        }
        Ok(self
            .people
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, p)| p.0.contains(term))
            .skip(offset)
            .take(limit)
            .map(|(k, p)| {
                let row = PersonRow::new(k.clone(), vec![p.0.clone(), p.1.clone()]);
                if self.guarded.lock().unwrap().contains(k) {
                    row.guarded("Sensitive")
                } else {
                    row
                }
            })
            .collect())
    }

    fn read(&self, key: &str) -> Result<PersonSheet, SourceError> {
        if key == "broken" {
            return Err(SourceError::failed("the record is damaged"));
        }
        let people = self.people.lock().unwrap();
        let (name, mail, _) = people
            .get(key)
            .ok_or_else(|| SourceError::not_found(format!("nobody called {key}")))?;
        Ok(PersonSheet::new(
            name.clone(),
            vec![PersonSection::new(
                "Contact",
                vec![PersonField::new("E-mail", mail.clone())],
            )],
        ))
    }

    fn draft(&self, key: &str) -> Result<PersonDraft, SourceError> {
        let people = self.people.lock().unwrap();
        let (name, mail, revision) = people
            .get(key)
            .ok_or_else(|| SourceError::not_found("gone"))?;
        Ok(PersonDraft::new(
            Some(revision.to_string()),
            vec![
                DraftGroup::new(
                    "name",
                    "Name",
                    false,
                    vec![DraftField::new("name", "Name", name.clone())],
                ),
                DraftGroup::new(
                    "mail",
                    "E-mail",
                    true,
                    vec![DraftField::new("mail/1", "E-mail", mail.clone())],
                ),
            ],
        ))
    }

    fn blank(&self) -> Result<PersonDraft, SourceError> {
        Ok(PersonDraft::new(
            None,
            vec![DraftGroup::new(
                "name",
                "Name",
                false,
                vec![DraftField::new("name", "Name", "")],
            )],
        ))
    }

    fn save(&self, key: &str, draft: &PersonDraft) -> Result<(), SourceError> {
        self.refused()?;
        let mut people = self.people.lock().unwrap();
        let entry = people
            .get_mut(key)
            .ok_or_else(|| SourceError::not_found("gone"))?;
        if draft.revision.as_deref() != Some(entry.2.to_string().as_str()) {
            return Err(SourceError::conflict("Someone else changed this person."));
        }
        entry.0 = value(draft, "name");
        entry.2 += 1;
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }

    fn create(&self, draft: &PersonDraft) -> Result<String, SourceError> {
        self.refused()?;
        let name = value(draft, "name");
        self.people
            .lock()
            .unwrap()
            .insert(name.clone(), (name.clone(), String::new(), 1));
        *self.writes.lock().unwrap() += 1;
        Ok(name)
    }

    fn delete(&self, key: &str) -> Result<Option<String>, SourceError> {
        self.refused()?;
        if self.people.lock().unwrap().remove(key).is_none() {
            return Err(SourceError::not_found("gone"));
        }
        *self.writes.lock().unwrap() += 1;
        Ok(self.delete_remark.lock().unwrap().clone())
    }
}

pub mod harness;
