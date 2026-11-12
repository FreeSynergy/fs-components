//! A read-only way for a test to ask what is typed into the text fields of a window (Cargo feature `test-access`).
//!
//! Pattern: Observer over a registry — every text field gets its data through `FieldData` (one door, see
//! `field_data.rs`), so that door is the one place that notes the fields. A test (the typing test of `fs-release`)
//! then compares the TEXT, character by character, with what it typed, and uses the picture only to see that it is
//! drawn. Risks and limits, named: (1) The feature is off by default and a release build does not have it: without it
//! this file is not compiled and no field is noted. (2) With the feature the text of every field that is not secret
//! can be read by whoever sets `FS_TEST_ACCESS_FILE` for the process, so only a test build has it. (3) A field that
//! holds a secret (input type `Password`, or masked) is never read and never written: it is not in the answer at all,
//! not even empty. (4) Nothing is written unless the environment variable names a file; the file is made with the
//! rights of the owner only; it is replaced as a whole (written aside, then renamed), so a reader never sees half.
//! (5) It only reads: nothing here changes a field.

use std::fmt::Write as _;
use std::sync::{Mutex, OnceLock, PoisonError, Weak};
use std::time::Duration;

use blinc_layout::widgets::text_area::TextAreaState;
use blinc_layout::widgets::text_input::{InputType, SharedTextInputData, TextInputData};
use std::sync::Arc;

/// What one field holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldText {
    /// The number of the field, in the order the fields were made.
    pub id: u64,
    /// What is typed in it.
    pub text: String,
}

enum Held {
    Input(Weak<Mutex<TextInputData>>),
    Area(Weak<Mutex<TextAreaState>>),
}

static REGISTRY: Mutex<Vec<(u64, Held)>> = Mutex::new(Vec::new());

pub(crate) fn note_input(id: u64, data: &SharedTextInputData) {
    REGISTRY
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((id, Held::Input(Arc::downgrade(data))));
}

pub(crate) fn note_area(id: u64, state: &Arc<Mutex<TextAreaState>>) {
    REGISTRY
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((id, Held::Area(Arc::downgrade(state))));
}

/// The text of every live field that is not secret, in the order the fields were made.
pub fn field_texts() -> Vec<FieldText> {
    let mut registry = REGISTRY.lock().unwrap_or_else(PoisonError::into_inner);
    registry.retain(|(_, held)| match held {
        Held::Input(w) => w.strong_count() > 0,
        Held::Area(w) => w.strong_count() > 0,
    });
    registry
        .iter()
        .filter_map(|(id, held)| {
            let text = match held {
                Held::Input(w) => {
                    let data = w.upgrade()?;
                    let data = data.lock().unwrap_or_else(PoisonError::into_inner);
                    if data.masked || data.input_type == InputType::Password {
                        return None;
                    }
                    data.value.clone()
                }
                Held::Area(w) => w
                    .upgrade()?
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .value(),
            };
            Some(FieldText { id: *id, text })
        })
        .collect()
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// The answer as text (JSON): `{"fields":[{"id":1,"text":"..."}]}`.
pub fn fields_json() -> String {
    let items: Vec<String> = field_texts()
        .iter()
        .map(|f| format!("{{\"id\":{},\"text\":\"{}\"}}", f.id, escape(&f.text)))
        .collect();
    format!("{{\"fields\":[{}]}}\n", items.join(","))
}

#[cfg(unix)]
fn write_aside(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let aside = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&aside)?;
    file.write_all(text.as_bytes())?;
    std::fs::rename(aside, path)
}

#[cfg(not(unix))]
fn write_aside(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let aside = path.with_extension("tmp");
    std::fs::write(&aside, text)?;
    std::fs::rename(aside, path)
}

/// Start the writer if the environment variable `FS_TEST_ACCESS_FILE` names a file: a thread that looks at the fields
/// every 40 ms and writes the answer when it changed. Does nothing otherwise, and only once.
pub fn start_from_env() {
    static ONCE: OnceLock<()> = OnceLock::new();
    let Some(path) = std::env::var_os("FS_TEST_ACCESS_FILE").map(std::path::PathBuf::from) else {
        return;
    };
    ONCE.get_or_init(|| {
        std::thread::spawn(move || {
            let mut last = String::new();
            loop {
                let now = fields_json();
                if now != last && write_aside(&path, &now).is_ok() {
                    last = now;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        });
    });
}
