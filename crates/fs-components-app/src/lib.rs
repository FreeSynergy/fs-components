//! `fs-components-app`: the shell every FreeSynergy program puts around its workbenches.
//!
//! A program describes its [`Screen`]s, each a [`Surface`], and calls [`launch`]. The shell brings
//! the window, the title bar with the menus, the navigation and the status line (ADR 0005: the
//! difference between widget and app is the shell). It brings no colours of its own (they come from the theme) and only the
//! words of the standard menus (`texts/`, German and English, ADR 0065); every other label comes from the program. Programs see no Blinc type except [`Element`] (ADR 0009).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod args;
mod error;
mod language;
mod menu;
mod menu_overlay;
mod menu_view;
#[cfg(test)]
mod menu_view_tests;
mod new_file;
mod quit;
mod shell;
mod standard;
mod standard_widget;
mod status_panel;
mod styles;
mod surface;
mod texts;
mod view;
mod widget;
mod widget_menu;
mod window;

pub use args::parse_size;
pub use error::AppError;
pub use fs_components_widgets::Element;
pub use language::{FileLanguageControl, LanguageChoice, LanguageControl, Scope};
pub use menu::{Menu, MenuEntry, MenuHandler};
pub use new_file::{write_new_file, write_new_private_file};
pub use shell::{Shell, Status};
pub use standard::{FrontendItem, StandardCallbacks, StandardMenus};
pub use styles::{MIN_WINDOW, min_surface};
pub use surface::{Screen, Surface};
pub use texts::TEXTS;
pub use widget::AppWidget;
pub use window::{WindowOptions, launch};
