//! The reader in a window.
//!
//! Pattern: Facade over `WindowedApp`. The state lives in an `Arc<Mutex<Reader>>`; two counters
//! (signals) tell the two live areas, sidebar body and content, to draw again after a change.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_app::prelude::*;
use blinc_app::windowed::WindowedApp;
use blinc_core::BlincContextState;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use fs_components_primitives::SearchField;
use fs_components_widgets::open_external;

use super::error::ShellError;
use super::setup::bundle;
use crate::reader::{Outcome, Reader};
use crate::source::DataSource;
use crate::view::{Actions, content, frame, search_box, side_body, tree_nodes};

/// Window settings.
#[derive(Clone, Debug)]
pub struct Options {
    /// Window title.
    pub title: String,
    /// Initial width in logical pixels.
    pub width: u32,
    /// Initial height in logical pixels.
    pub height: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            title: "Reader".into(),
            width: 1200,
            height: 800,
        }
    }
}

/// Open a window with the reader over `source` and run until it is closed.
///
/// # Errors
/// [`ShellError`] if the window or the graphics cannot be started.
pub fn launch(
    options: &Options,
    source: Arc<dyn DataSource>,
) -> std::result::Result<(), ShellError> {
    super::setup::register_fonts();
    let reader = Arc::new(Mutex::new(Reader::new(source)));
    let tree = Arc::new(tree_nodes(locked(&reader).entries()));
    let field = {
        let reader = Arc::clone(&reader);
        SearchField::new("Search pages").on_change(Arc::new(move |term| {
            locked(&reader).search(term);
            bump("fs-rd-side");
        }))
    };
    let config = WindowConfig {
        title: options.title.clone(),
        width: options.width,
        height: options.height,
        ..Default::default()
    };
    WindowedApp::run_with_theme(
        config,
        bundle(),
        blinc_theme::detect_system_color_scheme(),
        move |ctx| {
            let page_v = ctx.use_state_keyed("fs-rd-page", || 0u64);
            let side_v = ctx.use_state_keyed("fs-rd-side", || 0u64);
            let actions = actions(&reader);
            let side = {
                let (reader, tree, actions) =
                    (Arc::clone(&reader), Arc::clone(&tree), actions.clone());
                stateful::<NoState>()
                    .deps([side_v.signal_id()])
                    .on_state(move |_| side_body(&locked(&reader), &tree, &actions))
            };
            let main = {
                let (reader, actions) = (Arc::clone(&reader), actions.clone());
                stateful::<NoState>()
                    .deps([page_v.signal_id()])
                    .on_state(move |_| content(&locked(&reader), &actions))
            };
            let (back, save) = (Arc::clone(&actions.back), Arc::clone(&actions.save));
            let cancel = Arc::clone(&actions.cancel);
            frame(search_box(&field), side, main, ctx.width, ctx.height).on_key_down(move |e| {
                match e.key_code {
                    0x25 if e.alt => back(),
                    0x53 if e.ctrl => save(),
                    0x1B => cancel(),
                    _ => {}
                }
            })
        },
    )
    .map_err(|e| ShellError(e.to_string()))
}

/// The callbacks that change the shared reader and ask the areas to draw again.
fn actions(reader: &Arc<Mutex<Reader>>) -> Actions {
    let simple = |change: fn(&mut Reader)| -> Arc<dyn Fn() + Send + Sync> {
        let reader = Arc::clone(reader);
        Arc::new(move || {
            change(&mut locked(&reader));
            bump("fs-rd-page");
        })
    };
    let (open_r, follow_r, draft_r) = (Arc::clone(reader), Arc::clone(reader), Arc::clone(reader));
    Actions {
        open: Arc::new(move |path| {
            locked(&open_r).open(path);
            bump("fs-rd-page");
        }),
        follow: Arc::new(move |href| {
            let outcome = locked(&follow_r).follow(href);
            if let Outcome::External(url) = outcome {
                open_external(&url);
            }
            bump("fs-rd-page");
        }),
        back: simple(Reader::back),
        edit: simple(Reader::begin_edit),
        // Typing is remembered but does not redraw: the text area keeps its own cursor.
        draft: Arc::new(move |text| locked(&draft_r).set_draft(text)),
        save: simple(Reader::save),
        cancel: simple(Reader::cancel_edit),
        export: simple(Reader::export),
    }
}

/// Count up the signal `key`, which makes the areas depending on it draw again.
fn bump(key: &str) {
    BlincContextState::get()
        .use_state_keyed(key, || 0u64)
        .update(|v| v + 1);
}

/// The reader, also if a panic elsewhere poisoned the lock (the state is plain data).
fn locked(reader: &Mutex<Reader>) -> MutexGuard<'_, Reader> {
    reader.lock().unwrap_or_else(PoisonError::into_inner)
}
