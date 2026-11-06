//! A calendar source for the tests of the workbench.
//!
//! Pattern: Strategy (test double) — items in memory with a revision per item, like a store would; it
//! stands for "any source", not for a program's core (that is tested in `fs-calendar`).
#![allow(dead_code)] // each test file uses a different subset

pub mod harness;

use std::collections::BTreeMap;
use std::sync::Mutex;

use fs_components_workbenches::SourceError;
use fs_components_workbenches::calendar::{
    CalendarCapabilities, CalendarItem, CalendarSource, Day, DraftField, FieldKind, ItemDraft,
    ItemSheet, Moment, SheetField, SheetSection,
};

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

    fn refused(&self) -> Result<(), SourceError> {
        match self.refuse.lock().unwrap().clone() {
            Some(why) => Err(SourceError::failed(why)),
            None => Ok(()),
        }
    }

    fn marks(&self, key: &str) -> Vec<String> {
        if self.marked.lock().unwrap().iter().any(|k| k == key) {
            vec!["Sensitive".to_string()]
        } else {
            Vec::new()
        }
    }
}

impl CalendarSource for Diary {
    fn capabilities(&self) -> CalendarCapabilities {
        self.capabilities
    }

    fn now(&self) -> Moment {
        Moment::at(self.today, 10, 30)
    }

    fn week_start(&self) -> u8 {
        self.week_start
    }

    fn items(&self, first: Day, last: Day) -> Result<Vec<CalendarItem>, SourceError> {
        self.asked.lock().unwrap().push((first, last));
        if let Some(why) = self.broken.lock().unwrap().clone() {
            return Err(SourceError::failed(why));
        }
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, e)| first <= e.1 && e.1 <= last)
            .map(|(k, e)| {
                let mut item = CalendarItem::timed(
                    k.clone(),
                    e.0.clone(),
                    Moment::at(e.1, 9, 0),
                    Moment::at(e.1, 10, 0),
                );
                for mark in self.marks(k) {
                    item = item.marked(mark);
                }
                item
            })
            .collect())
    }

    fn read(&self, key: &str) -> Result<ItemSheet, SourceError> {
        if key == "broken" {
            return Err(SourceError::failed("the record is damaged"));
        }
        let entries = self.entries.lock().unwrap();
        let (title, day, _) = entries
            .get(key)
            .ok_or_else(|| SourceError::not_found(format!("nothing called {key}")))?;
        let mut sheet = ItemSheet::new(
            title.clone(),
            *day,
            vec![SheetSection::new(
                "When",
                vec![SheetField::new("Day", day.to_string())],
            )],
        );
        sheet.marks = self.marks(key);
        if key.contains("series") {
            sheet.delete_warning = Some("All of the series goes.".into());
        }
        Ok(sheet)
    }

    fn draft(&self, key: &str) -> Result<ItemDraft, SourceError> {
        let entries = self.entries.lock().unwrap();
        let (title, day, revision) = entries
            .get(key)
            .ok_or_else(|| SourceError::not_found("gone"))?;
        let mut draft = ItemDraft::new(
            Some(revision.to_string()),
            vec![
                DraftField::new("title", "Title", title.clone()),
                DraftField::new("day", "Day", day.to_string()),
                DraftField::new("all_day", "All day", "false").kind(FieldKind::Flag),
                DraftField::new("about", "About", "").kind(FieldKind::LongText),
                DraftField::new("fixed", "Fixed", "kept").locked(),
            ],
        );
        draft.marks = self.marks(key);
        Ok(draft)
    }

    fn blank(&self, day: Day) -> Result<ItemDraft, SourceError> {
        Ok(ItemDraft::new(
            None,
            vec![
                DraftField::new("title", "Title", ""),
                DraftField::new("day", "Day", day.to_string()),
            ],
        ))
    }

    fn save(&self, key: &str, draft: &ItemDraft) -> Result<(), SourceError> {
        self.refused()?;
        let mut entries = self.entries.lock().unwrap();
        let entry = entries
            .get_mut(key)
            .ok_or_else(|| SourceError::not_found("gone"))?;
        if draft.revision.as_deref() != Some(entry.2.to_string().as_str()) {
            return Err(SourceError::conflict("Someone else changed this."));
        }
        entry.0 = draft.value("title").unwrap_or_default().to_string();
        entry.1 = Day::parse(draft.value("day").unwrap_or_default())
            .ok_or_else(|| SourceError::failed("that is not a date"))?;
        entry.2 += 1;
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }

    fn create(&self, draft: &ItemDraft) -> Result<String, SourceError> {
        self.refused()?;
        let title = draft.value("title").unwrap_or_default().to_string();
        let day = Day::parse(draft.value("day").unwrap_or_default())
            .ok_or_else(|| SourceError::failed("that is not a date"))?;
        self.entries
            .lock()
            .unwrap()
            .insert(title.clone(), (title.clone(), day, 1));
        *self.writes.lock().unwrap() += 1;
        Ok(title)
    }

    fn delete(&self, key: &str) -> Result<(), SourceError> {
        self.refused()?;
        if self.entries.lock().unwrap().remove(key).is_none() {
            return Err(SourceError::not_found("gone"));
        }
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }
}
