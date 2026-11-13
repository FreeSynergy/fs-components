//! The language setting in a file: read it, write it atomically.
//!
//! Pattern: Memento holder (as `Preference` of the tasks program) — one small file `language` holding a BCP 47 tag.
//! There are two: the one shared by all FreeSynergy programs, `$XDG_CONFIG_HOME/freesynergy/language` (else
//! `$HOME/.config/freesynergy/language`), and a program's own override, `<data>/freesynergy/<program>/language` where
//! `<data>` is `$XDG_DATA_HOME` (else `$HOME/.local/share`), the folder the programs already keep their state in. As the XDG
//! rule says, a relative `$XDG_*` is ignored. A missing file is "no setting"; an unreadable or broken one is an error the
//! caller shows once and then treats as "no setting".

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Locale;

/// One settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSetting {
    file: PathBuf,
}

fn base(xdg: Option<OsString>, home: Option<OsString>, below_home: &str) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            home.map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join(below_home))
        })
}

impl LanguageSetting {
    /// The setting kept in `file`.
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    /// The setting shared by all programs, from `$XDG_CONFIG_HOME` and `$HOME` (pass the variables).
    pub fn shared(xdg_config: Option<OsString>, home: Option<OsString>) -> Option<Self> {
        let dir = base(xdg_config, home, ".config")?;
        Some(Self::new(dir.join("freesynergy").join("language")))
    }

    /// The override of `program`, from `$XDG_DATA_HOME` and `$HOME`.
    pub fn program(program: &str, xdg_data: Option<OsString>, home: Option<OsString>) -> Option<Self> {
        let dir = base(xdg_data, home, ".local/share")?;
        Some(Self::new(dir.join("freesynergy").join(program).join("language")))
    }

    /// The file.
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The saved language; `Ok(None)` when there is no file.
    ///
    /// # Errors
    /// A message for the person when the file cannot be read or holds no language tag.
    pub fn load(&self) -> Result<Option<Locale>, String> {
        let text = match std::fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("cannot read {}: {e}", self.file.display())),
        };
        let tag = text.trim();
        if tag.is_empty() {
            return Ok(None);
        }
        Locale::parse(tag)
            .map(Some)
            .map_err(|e| format!("{} holds no language: {e}", self.file.display()))
    }

    /// Remember `locale`; `None` removes the file (follow the next link of the chain). The new text is written to a file
    /// beside the old one, synced, then renamed over it: a crash leaves the old choice or the new one, never half.
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
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir).map_err(fail)?;
        }
        let temp = self.file.with_extension(format!("{}.tmp", std::process::id()));
        let written = std::fs::File::create(&temp).and_then(|mut f| {
            f.write_all(format!("{locale}\n").as_bytes())?;
            f.sync_all()
        });
        written
            .and_then(|()| std::fs::rename(&temp, &self.file))
            .map_err(|e| {
                let _ = std::fs::remove_file(&temp);
                fail(e)
            })
    }
}
