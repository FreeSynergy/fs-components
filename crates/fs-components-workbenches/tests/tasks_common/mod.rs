//! A task source for the tests of the workbench.
//!
//! Tests: a test double — tasks in memory with a revision each, like a store would; it
//! stands for "any source", not for a program's core (that is tested in `fs-tasks`).
#![allow(dead_code)] // each test file uses a different subset

pub mod harness;

use std::collections::BTreeMap;
use std::sync::Mutex;

use fs_components_workbenches::SourceError;
use fs_components_workbenches::tasks::{
    DraftField, FieldKind, FieldOption, RowQuery, TaskCapabilities, TaskColumn, TaskDraft, TaskRow,
    TaskSheet, TaskSource,
};

/// One task: title, column, revision.
#[derive(Clone, Debug)]
pub struct Item {
    pub title: String,
    pub column: String,
    pub revision: u32,
}

/// Tasks per key.
pub struct Shelf {
    pub tasks: Mutex<BTreeMap<String, Item>>,
    pub capabilities: TaskCapabilities,
    /// A save, create, delete or move fails with this reason.
    pub refuse: Mutex<Option<String>>,
    pub broken_from: Mutex<Option<usize>>,
    pub writes: Mutex<u32>,
    /// Keys to be marked and opened only when chosen.
    pub guarded: Mutex<Vec<String>>,
    /// Keys that cannot be moved.
    pub fixed: Mutex<Vec<String>>,
}

impl Shelf {
    pub fn with(keys: &[&str]) -> Self {
        let tasks = keys
            .iter()
            .map(|key| {
                let item = Item {
                    title: key.to_uppercase(),
                    column: "todo".into(),
                    revision: 1,
                };
                ((*key).to_string(), item)
            })
            .collect();
        Self {
            tasks: Mutex::new(tasks),
            capabilities: TaskCapabilities::all(),
            refuse: Mutex::new(None),
            broken_from: Mutex::new(None),
            writes: Mutex::new(0),
            guarded: Mutex::new(Vec::new()),
            fixed: Mutex::new(Vec::new()),
        }
    }

    pub fn numbered(count: usize) -> Self {
        let keys: Vec<String> = (0..count).map(|i| format!("t{i:05}")).collect();
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        Self::with(&keys)
    }

    pub fn column_of(&self, key: &str) -> Option<String> {
        self.tasks
            .lock()
            .unwrap()
            .get(key)
            .map(|t| t.column.clone())
    }

    pub fn title_of(&self, key: &str) -> Option<String> {
        self.tasks.lock().unwrap().get(key).map(|t| t.title.clone())
    }

    pub fn writes(&self) -> u32 {
        *self.writes.lock().unwrap()
    }

    /// Someone else changes `key` behind the workbench's back.
    pub fn other_changes(&self, key: &str, title: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        let entry = tasks.get_mut(key).unwrap();
        entry.title = title.into();
        entry.revision += 1;
    }

    /// Someone else deletes `key` behind the workbench's back.
    pub fn other_deletes(&self, key: &str) {
        self.tasks.lock().unwrap().remove(key);
    }

    fn refused(&self) -> Result<(), SourceError> {
        match self.refuse.lock().unwrap().clone() {
            Some(why) => Err(SourceError::failed(why)),
            None => Ok(()),
        }
    }

    fn get(&self, key: &str) -> Result<Item, SourceError> {
        self.tasks
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| SourceError::not_found("no such task"))
    }

    fn row(&self, key: &str, item: &Item) -> TaskRow {
        let mut row = TaskRow::new(key, item.title.clone(), item.column.clone());
        row.cells = vec![item.title.clone()];
        row.revision = Some(item.revision.to_string());
        if self.fixed.lock().unwrap().iter().any(|k| k == key) {
            row.move_refusal = Some("it repeats".into());
        }
        if self.guarded.lock().unwrap().iter().any(|k| k == key) {
            row = row.guarded("Sensitive");
        }
        row
    }
}

fn column_kind() -> FieldKind {
    FieldKind::Choice(vec![
        FieldOption::new("todo", "To do"),
        FieldOption::new("doing", "Doing"),
        FieldOption::new("done", "Done"),
    ])
}

impl TaskSource for Shelf {
    fn capabilities(&self) -> TaskCapabilities {
        self.capabilities
    }

    /// The rule of this shelf: a title may not hold a `!`.
    fn check(&self, key: &str, text: &str) -> Option<String> {
        (key == "title" && text.contains('!')).then(|| "no exclamation marks".to_string())
    }

    fn columns(&self) -> Vec<TaskColumn> {
        vec![
            TaskColumn::new("todo", "To do"),
            TaskColumn::new("doing", "Doing"),
            TaskColumn::new("done", "Done"),
        ]
    }

    fn rows(
        &self,
        query: &RowQuery,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<TaskRow>, SourceError> {
        if self
            .broken_from
            .lock()
            .unwrap()
            .is_some_and(|from| offset >= from)
        {
            return Err(SourceError::failed("the store is gone"));
        }
        let term = query.term.to_lowercase();
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| t.title.to_lowercase().contains(&term))
            .filter(|(_, t)| !query.open_only || t.column != "done")
            .skip(offset)
            .take(limit)
            .map(|(key, t)| self.row(key, t))
            .collect())
    }

    fn read(&self, key: &str) -> Result<TaskSheet, SourceError> {
        let item = self.get(key)?;
        let row = self.row(key, &item);
        let mut sheet = TaskSheet::new(item.title, Vec::new());
        sheet.marks = row.marks;
        sheet.column = item.column;
        sheet.move_refusal = row.move_refusal;
        sheet.revision = row.revision;
        Ok(sheet)
    }

    fn draft(&self, key: &str) -> Result<TaskDraft, SourceError> {
        let item = self.get(key)?;
        Ok(TaskDraft::new(
            Some(item.revision.to_string()),
            vec![
                DraftField::new("title", "Title", item.title),
                DraftField::new("column", "Status", item.column).kind(column_kind()),
                DraftField::new("notes", "Notes", "").kind(FieldKind::LongText),
            ],
        ))
    }

    fn blank(&self) -> Result<TaskDraft, SourceError> {
        Ok(TaskDraft::new(
            None,
            vec![DraftField::new("title", "Title", "")],
        ))
    }

    fn save(&self, key: &str, draft: &TaskDraft) -> Result<(), SourceError> {
        self.refused()?;
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(key)
            .ok_or_else(|| SourceError::not_found("no such task"))?;
        if draft.revision.as_deref() != Some(task.revision.to_string().as_str()) {
            return Err(SourceError::conflict("Someone else changed this."));
        }
        task.title = draft.value("title").unwrap_or_default().to_string();
        task.column = draft.value("column").unwrap_or_default().to_string();
        task.revision += 1;
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }

    fn create(&self, draft: &TaskDraft) -> Result<String, SourceError> {
        self.refused()?;
        let title = draft.value("title").unwrap_or_default().to_string();
        let key = title.to_lowercase();
        self.tasks.lock().unwrap().insert(
            key.clone(),
            Item {
                title,
                column: "todo".into(),
                revision: 1,
            },
        );
        *self.writes.lock().unwrap() += 1;
        Ok(key)
    }

    fn delete(&self, key: &str) -> Result<(), SourceError> {
        self.refused()?;
        self.get(key)?;
        self.tasks.lock().unwrap().remove(key);
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }

    fn move_to(&self, key: &str, column: &str, revision: Option<&str>) -> Result<(), SourceError> {
        self.refused()?;
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .get_mut(key)
            .ok_or_else(|| SourceError::not_found("no such task"))?;
        if revision != Some(task.revision.to_string().as_str()) {
            return Err(SourceError::conflict("Someone else changed this."));
        }
        task.column = column.to_string();
        task.revision += 1;
        *self.writes.lock().unwrap() += 1;
        Ok(())
    }
}
