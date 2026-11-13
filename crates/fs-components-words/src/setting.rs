//! The language setting in a file: read it, write it atomically.
//!
//! Pattern: Memento holder (as `Preference` of the tasks program) — one small file `language` holding a BCP 47 tag.
//! There are two, both below `$XDG_CONFIG_HOME` (else `$HOME/.config`): the one shared by all FreeSynergy programs,
//! `freesynergy/language`, and a program's own override, `freesynergy/<program>/language`. As the XDG rule says, a relative
//! `$XDG_*` is ignored. A missing file is "no setting"; an unreadable, oversized or broken one is an error. Use
//! [`load_and_resolve`](crate::load_and_resolve) so that every caller treats such an error the same way: report it once and go
//! on down the chain.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::Locale;

/// One settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSetting {
    file: PathBuf,
}

/// The most a language file may hold: a tag is at most a few dozen bytes; a link to a device or a huge file is refused.
const MAX_BYTES: u64 = 64;

fn base(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join(".config"))
        })
}

/// A program name is an identifier: lower-case letters, digits and `-`, starting with a letter or digit. It becomes a folder
/// name, so no `/`, no `..`, no absolute path.
fn name_ok(name: &str) -> bool {
    name.bytes()
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl LanguageSetting {
    /// The setting kept in `file`.
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    /// The setting shared by all programs, from `$XDG_CONFIG_HOME` and `$HOME` (pass the variables).
    pub fn shared(xdg_config: Option<OsString>, home: Option<OsString>) -> Option<Self> {
        let dir = base(xdg_config, home)?;
        Some(Self::new(dir.join("freesynergy").join("language")))
    }

    /// The override of `program`, from `$XDG_CONFIG_HOME` and `$HOME`. `None` if the name is no identifier (see
    /// `name_ok`: `a-z`, `0-9`, `-`) or there is no folder to keep it in.
    pub fn program(
        program: &str,
        xdg_config: Option<OsString>,
        home: Option<OsString>,
    ) -> Option<Self> {
        if !name_ok(program) {
            return None;
        }
        let dir = base(xdg_config, home)?;
        Some(Self::new(
            dir.join("freesynergy").join(program).join("language"),
        ))
    }

    /// The file.
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The saved language; `Ok(None)` when there is no file or it is blank.
    ///
    /// # Errors
    /// A message for the person when the file is no regular file, is larger than 64 bytes, cannot be read or holds no
    /// language tag.
    pub fn load(&self) -> Result<Option<Locale>, String> {
        let shown = self.file.display();
        let mut file = match std::fs::File::open(&self.file) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("cannot read {shown}: {e}")),
        };
        // Checked on the open file, so a link to a pipe or a device is refused before anything is read.
        if !file.metadata().is_ok_and(|m| m.is_file()) {
            return Err(format!("cannot read {shown}: not a regular file"));
        }
        let mut bytes = Vec::new();
        let read = (&mut file).take(MAX_BYTES + 1).read_to_end(&mut bytes);
        if let Err(e) = read {
            return Err(format!("cannot read {shown}: {e}"));
        }
        if bytes.len() as u64 > MAX_BYTES {
            return Err(format!("{shown} holds no language: too large"));
        }
        let text =
            String::from_utf8(bytes).map_err(|_| format!("{shown} holds no language: not text"))?;
        let tag = text.trim();
        if tag.is_empty() {
            return Ok(None);
        }
        Locale::parse(tag)
            .map(Some)
            .map_err(|e| format!("{shown} holds no language: {e}"))
    }

    /// Remember `locale`; `None` removes the file (follow the next link of the chain). The new text is written to a new file
    /// beside the old one (`create_new`: an existing name, even a link, is never followed), synced, renamed over it, and the
    /// folder is synced: a crash leaves the old choice or the new one, never half.
    ///
    /// # Errors
    /// A message for the person when the folder or the file cannot be written.
    pub fn save(&self, locale: Option<&Locale>) -> Result<(), String> {
        let fail = |e: std::io::Error| format!("{}: {e}", self.file.display());
        let Some(locale) = locale else {
            return match std::fs::remove_file(&self.file) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(fail(e)),
                _ => Ok(()),
            };
        };
        let dir = self
            .file
            .parent()
            .ok_or_else(|| format!("{}: no folder", self.file.display()))?;
        std::fs::create_dir_all(dir).map_err(fail)?;
        let temp = self
            .file
            .with_extension(format!("{}.tmp", std::process::id()));
        let written = create_new(&temp).and_then(|mut f| {
            f.write_all(format!("{locale}\n").as_bytes())?;
            f.sync_all()
        });
        written
            .and_then(|()| std::fs::rename(&temp, &self.file))
            .and_then(|()| std::fs::File::open(dir)?.sync_all())
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp);
                fail(e)
            })
    }
}

/// A new file; a leftover of a crashed run under the same name (not a folder) is removed once, a link is removed, not followed.
fn create_new(path: &Path) -> std::io::Result<std::fs::File> {
    let open = || {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
    };
    match open() {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            std::fs::remove_file(path)?;
            open()
        }
        other => other,
    }
}
