//! Test support: a description in the shape of a contact, and a `Backend` over memory.
//!
//! Pattern: Repository (an in-memory implementation of the trait, which is a real one: it checks, stores and refuses
//! like a program would).
#![allow(dead_code, clippy::pedantic)]

mod contact;

use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, Mutex};

use fs_admin_frontend::{Backend, BackendError, ManualClock, Outcome, Record, Values};
use fs_types::{BasicRules, CommandKind, RecordIssue, TypeDescription};

pub use contact::contact;

pub fn values(pairs: &[(&str, &[&str])]) -> Values {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.iter().map(|s| (*s).to_owned()).collect()))
        .collect()
}

pub fn record(id: &str, pairs: &[(&str, &[&str])]) -> Record {
    Record {
        id: id.to_owned(),
        values: values(pairs),
    }
}

/// Records in memory; `down` makes every call fail like an unreachable program.
pub struct Memory {
    pub records: Mutex<Vec<Record>>,
    pub down: Mutex<bool>,
    /// A name the program's own rule refuses (a rule the form cannot know).
    pub forbidden: Mutex<Option<String>>,
    /// After one successful command the program goes away (a reload then fails).
    pub down_after_run: Mutex<bool>,
    pub reject_delete: Mutex<bool>,
    pub reject_search: Mutex<bool>,
    /// Counts the calls of `list`; the call with this number fails (the program goes away for that one call).
    pub list_calls: std::sync::atomic::AtomicUsize,
    /// Every `list` answers this error while it is set.
    pub list_error: Mutex<Option<BackendError>>,
    pub fail_list_call: Mutex<Option<usize>>,
    /// What `create` refuses: 0 nothing, 1 a refusal with no findings, 2 a finding on `name` and one on an unknown
    /// field, 3 a finding on index 0 of `emails` (counted over the values that were sent).
    pub create_refusal: std::sync::atomic::AtomicUsize,
    /// Commands the program does not carry out yet.
    pub unsupported: Mutex<std::collections::BTreeSet<String>>,
    /// The last `Edit` the program received.
    pub last_edit: Mutex<Option<fs_types::Edit>>,
    pub next_id: std::sync::atomic::AtomicUsize,
    /// Counts the calls of `reconnect`; it answers this error while it is set, else it clears `list_error`.
    pub reconnects: std::sync::atomic::AtomicUsize,
    pub reconnect_error: Mutex<Option<BackendError>>,
    /// `reconnect` panics while this is set.
    pub reconnect_panics: std::sync::atomic::AtomicBool,
    /// Every `run` answers this error while it is set.
    pub run_error: Mutex<Option<BackendError>>,
    pub description: TypeDescription,
}

impl Memory {
    pub fn new(records: Vec<Record>) -> Arc<Self> {
        Arc::new(Self {
            records: Mutex::new(records),
            down: Mutex::new(false),
            forbidden: Mutex::new(None),
            down_after_run: Mutex::new(false),
            reject_delete: Mutex::new(false),
            reject_search: Mutex::new(false),
            list_calls: std::sync::atomic::AtomicUsize::new(0),
            list_error: Mutex::new(None),
            fail_list_call: Mutex::new(None),
            create_refusal: std::sync::atomic::AtomicUsize::new(0),
            unsupported: Mutex::new(std::collections::BTreeSet::new()),
            last_edit: Mutex::new(None),
            next_id: std::sync::atomic::AtomicUsize::new(100),
            reconnects: std::sync::atomic::AtomicUsize::new(0),
            reconnect_error: Mutex::new(None),
            reconnect_panics: std::sync::atomic::AtomicBool::new(false),
            run_error: Mutex::new(None),
            description: contact(),
        })
    }

    fn up(&self) -> Result<(), BackendError> {
        if *self.down.lock().unwrap() {
            Err(BackendError::Unavailable("socket closed".into()))
        } else {
            Ok(())
        }
    }
}

impl Backend for Memory {
    fn reconnect(&self) -> Result<(), BackendError> {
        self.reconnects.fetch_add(1, SeqCst);
        assert!(!self.reconnect_panics.load(SeqCst), "a backend that panics");
        if let Some(e) = self.reconnect_error.lock().unwrap().clone() {
            return Err(e);
        }
        *self.list_error.lock().unwrap() = None;
        Ok(())
    }

    fn supported(&self, command: &str) -> bool {
        !self.unsupported.lock().unwrap().contains(command)
    }

    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError> {
        let n = self.list_calls.fetch_add(1, SeqCst) + 1;
        if *self.fail_list_call.lock().unwrap() == Some(n) {
            return Err(BackendError::Unavailable("one call lost".into()));
        }
        if let Some(e) = self.list_error.lock().unwrap().clone() {
            return Err(e);
        }
        self.up()?;
        if *self.reject_search.lock().unwrap() && query.contains_key("term") {
            return Err(BackendError::Rejected(vec![RecordIssue {
                field: "term".into(),
                index: 0,
                problem: fs_types::RecordProblem::Value(fs_types::TypeProblem::TooLong),
            }]));
        }
        let needle = query
            .get("term")
            .and_then(|v| v.first())
            .map(|t| t.to_lowercase());
        Ok(self
            .records
            .lock()
            .unwrap()
            .iter()
            .filter(|r| {
                needle.as_ref().is_none_or(|n| {
                    r.values
                        .values()
                        .flatten()
                        .any(|t| t.to_lowercase().contains(n))
                })
            })
            .cloned()
            .collect())
    }

    fn change(&self, record: &str, edit: &fs_types::Edit) -> Result<Outcome, BackendError> {
        self.up()?;
        // the program checks the edit again, as a program must
        let edit = edit
            .check(&self.description, &BasicRules::standard())
            .map_err(BackendError::Rejected)?;
        *self.last_edit.lock().unwrap() = Some(edit.clone());
        let bad = self.forbidden.lock().unwrap().clone();
        let names = edit.fields.iter().find(|f| f.path == ["name"]);
        if bad.is_some_and(|b| names.is_some_and(|n| n.values.contains(&b))) {
            return Err(BackendError::Rejected(vec![RecordIssue {
                field: "name".into(),
                index: 0,
                problem: fs_types::RecordProblem::Value(fs_types::TypeProblem::Character),
            }]));
        }
        let mut records = self.records.lock().unwrap();
        let r = records
            .iter_mut()
            .find(|r| r.id == record)
            .ok_or(BackendError::NotFound)?;
        edit.apply(&mut r.values);
        let out = Ok(Outcome {
            id: Some(record.to_owned()),
            values: Values::new(),
        });
        if *self.down_after_run.lock().unwrap() {
            *self.down.lock().unwrap() = true;
        }
        out
    }

    fn run(
        &self,
        name: &str,
        record: Option<&str>,
        input: &Values,
    ) -> Result<Outcome, BackendError> {
        if let Some(e) = self.run_error.lock().unwrap().clone() {
            return Err(e);
        }
        self.up()?;
        let command = self
            .description
            .find_command(name)
            .ok_or(BackendError::NotFound)?;
        // the program checks again, as a program must
        let mut as_type = TypeDescription::new("Input", "t", "t", "t");
        as_type.fields = command.input.clone();
        let issues: Vec<RecordIssue> = as_type.check_record(&BasicRules::standard(), input);
        if !issues.is_empty() {
            return Err(BackendError::Rejected(issues));
        }
        if command.kind == CommandKind::Create {
            let issue = |field: &str, index| RecordIssue {
                field: field.into(),
                index,
                problem: fs_types::RecordProblem::Value(fs_types::TypeProblem::Character),
            };
            match self.create_refusal.load(SeqCst) {
                1 => return Err(BackendError::Rejected(vec![])),
                2 => {
                    return Err(BackendError::Rejected(vec![
                        issue("name", 0),
                        issue("nope", 0),
                    ]));
                }
                3 => return Err(BackendError::Rejected(vec![issue("emails", 0)])),
                4 => {
                    return Err(BackendError::Rejected(vec![
                        issue("nope", 0),
                        issue("nada", 1),
                    ]));
                }
                _ => {}
            }
        }
        let bad = self.forbidden.lock().unwrap().clone();
        if bad.is_some_and(|b| input.get("name").is_some_and(|n| n.contains(&b))) {
            return Err(BackendError::Rejected(vec![RecordIssue {
                field: "name".into(),
                index: 0,
                problem: fs_types::RecordProblem::Value(fs_types::TypeProblem::Character),
            }]));
        }
        let mut records = self.records.lock().unwrap();
        let out = match command.kind {
            CommandKind::Create => {
                let id = format!("urn:uuid:{:08}", self.next_id.fetch_add(1, SeqCst));
                records.push(Record {
                    id: id.clone(),
                    values: input.clone(),
                });
                Ok(Outcome {
                    id: Some(id),
                    values: Values::new(),
                })
            }
            CommandKind::Delete => {
                if *self.reject_delete.lock().unwrap() {
                    return Err(BackendError::Rejected(vec![RecordIssue {
                        field: "id".into(),
                        index: 0,
                        problem: fs_types::RecordProblem::Value(fs_types::TypeProblem::Character),
                    }]));
                }
                let id = record.ok_or(BackendError::NotFound)?.to_owned();
                let before = records.len();
                records.retain(|r| r.id != id);
                if records.len() == before {
                    return Err(BackendError::NotFound);
                }
                Ok(Outcome::default())
            }
            CommandKind::Suggest => {
                let mut domains: Vec<String> = records
                    .iter()
                    .filter_map(|r| r.values.get("emails"))
                    .flatten()
                    .filter_map(|e| e.split_once('@').map(|(_, d)| d.to_owned()))
                    .collect();
                domains.sort();
                domains.dedup();
                Ok(Outcome {
                    id: None,
                    values: values(&[(
                        "domains",
                        &domains.iter().map(String::as_str).collect::<Vec<_>>(),
                    )]),
                })
            }
            _ => Err(BackendError::NotFound),
        };
        if out.is_ok() && *self.down_after_run.lock().unwrap() {
            *self.down.lock().unwrap() = true;
        }
        out
    }
}

pub fn clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::default())
}
