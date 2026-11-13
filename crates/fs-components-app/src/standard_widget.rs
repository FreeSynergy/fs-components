//! `AppWidget` and the standard menus: install them, change the frontends, answer a chosen line (see `standard.rs` for the shape).
//!
//! Pattern: Facade over `Shell` and the state kept in `AppWidget::standard`. No lock is held while a callback of the program runs.

use std::sync::Arc;

use fs_components_words::{Args, Locale, TextId, Words};

use crate::AppWidget;
use crate::error::AppError;
use crate::language::{LanguageChoice, LanguageControl, Scope};
use crate::shell::Status;
use crate::standard::{
    FRONTEND, FrontendItem, Installed, LANGUAGE, QUIT, StandardCallbacks, StandardMenus, build, say,
};
use crate::texts::shell_texts;

impl AppWidget {
    /// Put the standard menus of ADR 0065 into the title bar (see the module for their shape). Replaces menus set before. Call it
    /// on the interface thread; the callbacks run with no lock held.
    ///
    /// # Errors
    /// [`AppError::DuplicateMenuEntry`] if the program's views use an id twice; [`AppError::ReservedMenuId`] if one starts with
    /// `fs.`; [`AppError::UnknownFrontend`] if `active_frontend` is not among the `frontends`. Nothing changes then.
    pub fn set_standard_menus(
        &self,
        standard: StandardMenus,
        callbacks: StandardCallbacks,
    ) -> Result<(), AppError> {
        standard.check()?;
        *self
            .standard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Installed {
            standard,
            callbacks,
        });
        self.install_standard()
    }

    /// The frontends of the program changed (one was loaded or removed): the submenu View > Frontends is made again. `on_choose`
    /// replaces the callback for a chosen frontend.
    ///
    /// # Errors
    /// [`AppError::NoStandardMenus`] if `set_standard_menus` was not called; [`AppError::UnknownFrontend`] if `active` is not in
    /// `items` (nothing changes).
    pub fn set_frontends(
        &self,
        items: Vec<FrontendItem>,
        active: &str,
        on_choose: impl Fn(&str) + Send + Sync + 'static,
    ) -> Result<(), AppError> {
        if !items.iter().any(|f| f.id == active) {
            return Err(AppError::UnknownFrontend(active.to_owned()));
        }
        self.change(|i| {
            i.standard.frontends = items;
            i.standard.active_frontend = Some(active.to_owned());
            i.callbacks.frontend = Arc::new(move |id: &str| on_choose(id));
            Ok(())
        })
    }

    /// Mark another frontend as the one in use (the program switched without a click in the menu).
    ///
    /// # Errors
    /// [`AppError::NoStandardMenus`] if `set_standard_menus` was not called; [`AppError::UnknownFrontend`] if `id` is not one of
    /// the frontends.
    pub fn set_active_frontend(&self, id: &str) -> Result<(), AppError> {
        self.change(|i| {
            if !i.standard.frontends.iter().any(|f| f.id == id) {
                return Err(AppError::UnknownFrontend(id.to_owned()));
            }
            i.standard.active_frontend = Some(id.to_owned());
            Ok(())
        })
    }

    fn change(
        &self,
        edit: impl FnOnce(&mut Installed) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        match self
            .standard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_mut()
        {
            Some(installed) => edit(installed)?,
            None => return Err(AppError::NoStandardMenus),
        }
        self.install_standard()
    }

    fn install_standard(&self) -> Result<(), AppError> {
        let menus = {
            let guard = self
                .standard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(installed) = guard.as_ref() else {
                return Err(AppError::NoStandardMenus);
            };
            build(&installed.standard)
        };
        let this = self.clone();
        self.set_menus(menus, move |id| this.answer(id))
    }

    /// One id of the standard menus: the shell answers its own, the rest goes to the program.
    fn answer(&self, id: &str) {
        let (callbacks, control) = {
            let guard = self
                .standard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(i) = guard.as_ref() else { return };
            (i.callbacks.clone(), i.standard.language.clone())
        };
        if id == QUIT {
            self.quit();
        } else if let Some(frontend) = id.strip_prefix(FRONTEND) {
            let _ = self.set_active_frontend(frontend);
            (callbacks.frontend)(frontend);
        } else if let Some(tag) = id.strip_prefix(LANGUAGE) {
            if let Some(control) = control {
                self.change_language(control.as_ref(), tag, &callbacks);
            }
        } else {
            (callbacks.entry)(id);
        }
    }

    fn change_language(
        &self,
        control: &dyn LanguageControl,
        tag: &str,
        callbacks: &StandardCallbacks,
    ) {
        let choice = if tag == "system" {
            LanguageChoice::System
        } else if let Ok(locale) = Locale::parse(tag) {
            LanguageChoice::Language(locale)
        } else {
            return;
        };
        match control.choose(Scope::Program, &choice) {
            Ok(now) => {
                let _ = self.change(|i| {
                    i.standard.locale = now.clone();
                    Ok(())
                });
                let message = say(&shell_texts(&now), "shell-language-saved");
                self.set_status(Some(Status::Info(message)));
                (callbacks.language)(&now);
            }
            Err(why) => {
                let locale = self.with_standard(|s| s.locale.clone());
                let text = shell_texts(&locale.unwrap_or_else(Locale::english));
                let message = text.say(
                    &TextId::new("shell-language-failed"),
                    &Args::new().text("reason", why),
                );
                self.set_status(Some(Status::Problem(message.text().to_owned())));
            }
        }
    }

    fn with_standard<T>(&self, read: impl FnOnce(&StandardMenus) -> T) -> Option<T> {
        let guard = self
            .standard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.as_ref().map(|i| read(&i.standard))
    }
}
