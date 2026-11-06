//! The test diary as a calendar source.
//!
//! Pattern: Strategy (test double), see the parent module.

use fs_components_workbenches::SourceError;
use fs_components_workbenches::calendar::{
    CalendarCapabilities, CalendarItem, CalendarSource, Day, DraftField, FieldKind, ItemDraft,
    ItemSheet, Moment, SheetField, SheetSection,
};

use super::Diary;

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
