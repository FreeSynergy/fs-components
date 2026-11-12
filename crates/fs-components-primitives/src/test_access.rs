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
//! rights of the owner only (mode 0600, made new with `create_new` and `O_NOFOLLOW`: an existing file or a link of
//! that name is replaced, never written through); it is replaced as a whole (written aside, then renamed), so a
//! reader never sees half. (5) It only reads: nothing here changes a field. (6) A build without debug assertions
//! (a release build) refuses the feature at compile time (`compile_error!` in `lib.rs`). (7) Fields are noted only
//! once `FS_TEST_ACCESS_FILE` is set (`start_from_env`, or `enable()` in a test): without it the registry stays empty
//! and cannot grow; with it, dead entries are dropped as the list grows. (8) The writer thread lives until the
//! process ends (it has no stop; it only reads and writes the one file); that is the intended end of a test window.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
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
static ENABLED: AtomicBool = AtomicBool::new(false);

/// How many entries the registry may hold before the dead ones are dropped on the next note.
const PRUNE_AT: usize = 256;

/// Switch the registry on (`start_from_env` does it when the variable is set; a test may do it by hand). Fields made
/// before this are not noted.
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

/// Whether fields are noted at all.
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn note(id: u64, held: Held) {
    if !enabled() {
        return;
    }
    let mut registry = REGISTRY.lock().unwrap_or_else(PoisonError::into_inner);
    if registry.len() >= PRUNE_AT {
        registry.retain(|(_, h)| alive(h));
    }
    registry.push((id, held));
}

fn alive(held: &Held) -> bool {
    match held {
        Held::Input(w) => w.strong_count() > 0,
        Held::Area(w) => w.strong_count() > 0,
    }
}

/// How many entries the registry holds now (live or not yet dropped).
pub fn registry_len() -> usize {
    REGISTRY
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .len()
}

pub(crate) fn note_input(id: u64, data: &SharedTextInputData) {
    note(id, Held::Input(Arc::downgrade(data)));
}

pub(crate) fn note_area(id: u64, state: &Arc<Mutex<TextAreaState>>) {
    note(id, Held::Area(Arc::downgrade(state)));
}

/// The text of every live field that is not secret, in the order the fields were made.
pub fn field_texts() -> Vec<FieldText> {
    let mut registry = REGISTRY.lock().unwrap_or_else(PoisonError::into_inner);
    registry.retain(|(_, held)| alive(held));
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

/// Write `text` to a new file beside `path` and rename it over `path`. The side file is made new (a stale one or a
/// link of that name is removed first, never followed), with mode 0600 and without following a link at the last
/// part of the name; the rename replaces whatever `path` is (a file or a link), it does not write through it.
#[cfg(unix)]
pub fn write_aside(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let aside = path.with_extension("tmp");
    match std::fs::remove_file(&aside) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&aside)?;
    file.write_all(text.as_bytes())?;
    drop(file);
    std::fs::rename(aside, path)
}

#[cfg(not(unix))]
pub fn write_aside(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let aside = path.with_extension("tmp");
    let _ = std::fs::remove_file(&aside);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&aside)?;
    std::io::Write::write_all(&mut file, text.as_bytes())?;
    drop(file);
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
        enable();
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
