//! The standard menus of every window (ADR 0065): Program, View, Help, with the language and the frontends in them.
//!
//! Pattern: Builder (`StandardMenus` is what a program says; `build` makes the `Menu`s from it) + Facade (`AppWidget` keeps the
//! state, so a program only calls `set_standard_menus` once and `set_frontends` when its frontends change). The shell's own
//! words come from `texts/{en,de}.mf2` in the language of the program (`StandardMenus::locale`), so a bar is never half German.
//! Menu structure, with the ids the program never sees (the shell answers them): Program = Settings (`fs.settings`, goes to
//! the program), Language > System language, one line per language (`fs.language.*`), Quit (`fs.quit`, the shell closes the
//! window); View = the program's own views, then Frontends > one line per frontend, the active one checked (`fs.frontend.*`);
//! Help = Help, About (`fs.help`, `fs.about`, go to the program). Views (tiles, list) and frontends are two different things:
//! views stay the program's, frontends are the shell's. The methods of `AppWidget` that install and answer them are in
//! `standard_widget.rs`.

use std::sync::{Arc, Mutex};

use fs_components_mf2::Mf2Texts;
use fs_components_words::{Args, Locale, TextId, Words};

use crate::language::{LanguageChoice, LanguageControl, Scope};
use crate::menu::{Menu, MenuEntry};
use crate::texts::{language_name, shell_texts};

const SETTINGS: &str = "fs.settings";
pub(crate) const QUIT: &str = "fs.quit";
const HELP: &str = "fs.help";
const ABOUT: &str = "fs.about";
pub(crate) const FRONTEND: &str = "fs.frontend.";
pub(crate) const LANGUAGE: &str = "fs.language.";

/// One frontend of the program, as the menu shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendItem {
    /// Stable name; the callback receives it.
    pub id: String,
    /// What the menu shows (the name of the frontend, in the program's language).
    pub label: String,
    /// Shown, but cannot be chosen (not installed completely, not supported here).
    pub disabled: bool,
}

impl FrontendItem {
    /// A frontend that can be chosen.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// The same, shown but not to be chosen.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// What a program says about the standard menus.
#[derive(Clone)]
pub struct StandardMenus {
    /// The language of the shell's own words; the program passes what it resolved.
    pub locale: Locale,
    /// The program's own views (tiles, list ...) at the top of the menu View; their ids must not start with `fs.`.
    pub views: Vec<MenuEntry>,
    /// Settings in the menu Program (the program opens its settings when it gets `fs.settings`).
    pub settings: bool,
    /// The frontends the program has loaded, in the order of the menu.
    pub frontends: Vec<FrontendItem>,
    /// The id of the frontend in use.
    pub active_frontend: Option<String>,
    /// The language setting; without it there is no entry Language.
    pub language: Option<Arc<dyn LanguageControl>>,
}

impl StandardMenus {
    /// Standard menus in this language, with Settings and nothing else yet.
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            views: Vec::new(),
            settings: true,
            frontends: Vec::new(),
            active_frontend: None,
            language: None,
        }
    }
}

type Callback<T> = Arc<dyn Fn(&T) + Send + Sync>;

/// What the shell calls when a person chooses something it does not answer itself.
#[derive(Clone)]
pub struct StandardCallbacks {
    pub(crate) entry: Callback<str>,
    pub(crate) frontend: Callback<str>,
    pub(crate) language: Callback<Locale>,
}

impl StandardCallbacks {
    /// `on_entry` gets the id of a line of the program: its own views, `fs.settings`, `fs.help`, `fs.about`.
    pub fn new(on_entry: impl Fn(&str) + Send + Sync + 'static) -> Self {
        Self {
            entry: Arc::new(move |id: &str| on_entry(id)),
            frontend: Arc::new(|_| {}),
            language: Arc::new(|_| {}),
        }
    }

    /// `on_frontend` gets the id of the frontend chosen; the shell then marks it.
    #[must_use]
    pub fn on_frontend(mut self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.frontend = Arc::new(move |id: &str| f(id));
        self
    }

    /// `on_language` gets the language spoken from now on, after the setting was written and the menus changed.
    #[must_use]
    pub fn on_language(mut self, f: impl Fn(&Locale) + Send + Sync + 'static) -> Self {
        self.language = Arc::new(move |l: &Locale| f(l));
        self
    }
}

/// What `AppWidget` keeps.
pub(crate) struct Installed {
    pub(crate) standard: StandardMenus,
    pub(crate) callbacks: StandardCallbacks,
}

/// The shell state for the standard menus.
pub(crate) type Slot = Arc<Mutex<Option<Installed>>>;

pub(crate) fn say(texts: &Mf2Texts, id: &str) -> String {
    texts.say(&TextId::new(id), &Args::new()).text().to_owned()
}

fn language_menu(texts: &Mf2Texts, control: &dyn LanguageControl) -> MenuEntry {
    let now = control.current(Scope::Program);
    let mut children = vec![
        MenuEntry::new(
            format!("{LANGUAGE}system"),
            say(texts, "shell-language-system"),
        )
        .checked(now == LanguageChoice::System),
    ];
    for language in control.available() {
        let checked = now == LanguageChoice::Language(language.clone());
        children.push(
            MenuEntry::new(
                format!("{LANGUAGE}{language}"),
                language_name(texts, &language),
            )
            .checked(checked),
        );
    }
    MenuEntry::submenu("fs.language", say(texts, "shell-language"), children)
}

/// The three menus.
pub(crate) fn build(standard: &StandardMenus) -> Vec<Menu> {
    let texts = shell_texts(&standard.locale);
    let mut program = Vec::new();
    if standard.settings {
        program.push(MenuEntry::new(SETTINGS, say(&texts, "shell-settings")));
    }
    if let Some(control) = &standard.language {
        program.push(language_menu(&texts, control.as_ref()));
    }
    program.push(MenuEntry::new(QUIT, say(&texts, "shell-quit")));
    let mut view = standard.views.clone();
    if !standard.frontends.is_empty() {
        let children = standard
            .frontends
            .iter()
            .map(|f| {
                let entry = MenuEntry::new(format!("{FRONTEND}{}", f.id), f.label.clone())
                    .checked(standard.active_frontend.as_deref() == Some(f.id.as_str()));
                if f.disabled { entry.disabled() } else { entry }
            })
            .collect();
        view.push(MenuEntry::submenu(
            "fs.frontends",
            say(&texts, "shell-frontends"),
            children,
        ));
    }
    vec![
        Menu::new(say(&texts, "shell-menu-program"), program),
        Menu::new(say(&texts, "shell-menu-view"), view),
        Menu::new(
            say(&texts, "shell-menu-help"),
            vec![
                MenuEntry::new(HELP, say(&texts, "shell-help")),
                MenuEntry::new(ABOUT, say(&texts, "shell-about")),
            ],
        ),
    ]
}
