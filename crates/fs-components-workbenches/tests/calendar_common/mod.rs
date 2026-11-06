//! A calendar source for the tests of the workbench.
//!
//! Pattern: Strategy (test double) — items in memory with a revision per item, like a store would; it
//! stands for "any source", not for a program's core (that is tested in `fs-calendar`).
#![allow(dead_code)] // each test file uses a different subset

pub mod harness;
mod source;

use std::collections::BTreeMap;
use std::sync::Mutex;

use fs_components_workbenches::SourceError;
use fs_components_workbenches::calendar::{CalendarCapabilities, Day, ItemProblem};

pub fn d(text: &str) -> Day {
    Day::parse(text).unwrap()
}

/// Title, day and revision per key.
pub struct Diary {
    pub entries: Mutex<BTreeMap<String, (String, Day, u32)>>,
    pub capabilities: CalendarCapabilities,
    /// A save, create or delete fails with this reason.
    pub refuse: Mutex<Option<String>>,
    /// The items fail with this reason.
    pub broken: Mutex<Option<String>>,
    pub writes: Mutex<u32>,
    /// Keys to be marked.
    pub marked: Mutex<Vec<String>>,
    /// Reported along with the items.
    pub problems: Mutex<Vec<ItemProblem>>,
    /// Which spans were asked for.
    pub asked: Mutex<Vec<(Day, Day)>>,
    pub today: Day,
    pub week_start: u8,
}

impl Diary {
    pub fn with(entries: &[(&str, &str)]) -> Self {
        Self {
            entries: Mutex::new(
                entries
                    .iter()
                    .map(|(k, day)| ((*k).to_string(), ((*k).to_string(), d(day), 1)))
                    .collect(),
            ),
            capabilities: CalendarCapabilities::all(),
            refuse: Mutex::new(None),
            broken: Mutex::new(None),
            writes: Mutex::new(0),
            marked: Mutex::new(Vec::new()),
            problems: Mutex::new(Vec::new()),
            asked: Mutex::new(Vec::new()),
            today: d("2026-11-06"),
            week_start: 0,
        }
    }

    pub fn other_changes(&self, key: &str, title: &str) {
        let mut entries = self.entries.lock().unwrap();
        let entry = entries.get_mut(key).unwrap();
        entry.0 = title.into();
        entry.2 += 1;
    }

    pub fn other_deletes(&self, key: &str) {
        self.entries.lock().unwrap().remove(key);
    }

    pub fn title(&self, key: &str) -> Option<String> {
        self.entries.lock().unwrap().get(key).map(|e| e.0.clone())
    }

    pub fn writes(&self) -> u32 {
        *self.writes.lock().unwrap()
    }

    pub(crate) fn refused(&self) -> Result<(), SourceError> {
        match self.refuse.lock().unwrap().clone() {
            Some(why) => Err(SourceError::failed(why)),
            None => Ok(()),
        }
    }

    pub(crate) fn marks(&self, key: &str) -> Vec<String> {
        if self.marked.lock().unwrap().iter().any(|k| k == key) {
            vec!["Sensitive".to_string()]
        } else {
            Vec::new()
        }
    }
}
