//! Which language a program speaks, and how a person changes it: the interface of the menu and of the settings page.
//!
//! Pattern: Strategy (`LanguageControl`) + Memento holder (the two files of `LanguageSetting`). The menu "Program > Language"
//! changes the **program's own** override (`Scope::Program`); "for all programs" (`Scope::AllPrograms`, the shared file) is
//! for the settings page of the admin frontend, which holds a `LanguageControl` and calls `choose` with that scope: the
//! shell's menu and the page go through the same object, so they cannot disagree. A broken file is reported once in
//! `Resolved::problems` and the chain goes on without it (`load_and_resolve`).

use std::sync::Arc;

use fs_components_words::{Env, LanguageSetting, Locale, Resolved, load_and_resolve};

/// Which setting a choice changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// This program only (the override).
    Program,
    /// All FreeSynergy programs (the shared setting).
    AllPrograms,
}

/// What a setting says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageChoice {
    /// Nothing is set: follow the next link of the chain (the system language).
    System,
    /// This language.
    Language(Locale),
}

/// Reads and changes the language setting of a program.
pub trait LanguageControl: Send + Sync {
    /// The languages the program has texts for, in the order of the menu.
    fn available(&self) -> Vec<Locale>;
    /// What this scope has set now.
    fn current(&self, scope: Scope) -> LanguageChoice;
    /// The language spoken now (the chain's answer), and the problems met on the way.
    fn resolve(&self) -> Resolved;
    /// Set this scope; returns the language spoken from now on.
    ///
    /// # Errors
    /// A message for the person when the file cannot be written; nothing changed then.
    fn choose(&self, scope: Scope, choice: &LanguageChoice) -> Result<Locale, String>;
}

/// The control over the two files; the environment is a parameter so that tests never read the real one.
pub struct FileLanguageControl {
    program: Option<LanguageSetting>,
    shared: Option<LanguageSetting>,
    env: Arc<dyn Env + Send + Sync>,
    available: Vec<Locale>,
}

impl FileLanguageControl {
    /// Over these files (`None` = nowhere to keep that setting), this environment and these languages.
    pub fn new(
        program: Option<LanguageSetting>,
        shared: Option<LanguageSetting>,
        env: Arc<dyn Env + Send + Sync>,
        available: Vec<Locale>,
    ) -> Self {
        Self {
            program,
            shared,
            env,
            available,
        }
    }

    fn file(&self, scope: Scope) -> Option<&LanguageSetting> {
        match scope {
            Scope::Program => self.program.as_ref(),
            Scope::AllPrograms => self.shared.as_ref(),
        }
    }
}

impl LanguageControl for FileLanguageControl {
    fn available(&self) -> Vec<Locale> {
        self.available.clone()
    }

    fn current(&self, scope: Scope) -> LanguageChoice {
        match self.file(scope).map(LanguageSetting::load) {
            Some(Ok(Some(locale))) => LanguageChoice::Language(locale),
            _ => LanguageChoice::System,
        }
    }

    fn resolve(&self) -> Resolved {
        let supported: Vec<&str> = self.available.iter().map(Locale::primary).collect();
        load_and_resolve(
            self.program.as_ref(),
            self.shared.as_ref(),
            self.env.as_ref(),
            &supported,
        )
    }

    fn choose(&self, scope: Scope, choice: &LanguageChoice) -> Result<Locale, String> {
        let file = self
            .file(scope)
            .ok_or("there is no folder to keep the language in")?;
        match choice {
            LanguageChoice::System => file.save(None)?,
            LanguageChoice::Language(locale) => file.save(Some(locale))?,
        }
        Ok(self.resolve().chosen.locale)
    }
}
