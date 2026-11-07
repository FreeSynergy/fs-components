//! What the person can do, as callbacks.
//!
//! Pattern: Observer — the widget decides what each does: change the state and
//! draw again. A view asks for the callback of a transition by naming it.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::{ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};

use crate::file_manager::context::PickedFile;
use crate::file_manager::state::FilesState;

/// A plain action.
pub(in crate::file_manager) type Act = Arc<dyn Fn() + Send + Sync>;
/// An action about something named by a key.
pub(in crate::file_manager) type ActOn = Arc<dyn Fn(&str) + Send + Sync>;
/// The person typed this into this field.
pub(in crate::file_manager) type Typed = Arc<dyn Fn(&str, &str) + Send + Sync>;
/// Called with the file chosen in the picker.
pub type OnPick = Arc<dyn Fn(&PickedFile) + Send + Sync>;

/// A change of the state, done by the widget.
pub(in crate::file_manager) type Apply =
    Arc<dyn Fn(&(dyn Fn(&mut FilesState) + Send + Sync)) + Send + Sync>;

/// The callbacks of one widget.
#[derive(Clone)]
pub(in crate::file_manager) struct FileActions {
    pub apply: Apply,
    /// Text typed into the line of the band: does not draw again.
    pub typed: Arc<dyn Fn(&str) + Send + Sync>,
    /// Text typed into a field of the filter: does not draw again.
    pub typed_filter: Typed,
}

impl FileActions {
    /// Callbacks that do nothing, for working out what would be drawn.
    pub fn silent() -> Self {
        Self {
            apply: Arc::new(|_| {}),
            typed: Arc::new(|_| {}),
            typed_filter: Arc::new(|_, _| {}),
        }
    }

    /// The callback of this transition.
    pub fn plain(&self, change: fn(&mut FilesState)) -> Act {
        let apply = Arc::clone(&self.apply);
        Arc::new(move || apply(&change))
    }

    /// The callback of this transition about a key.
    pub fn on_key(&self, change: fn(&mut FilesState, &str)) -> ActOn {
        let apply = Arc::clone(&self.apply);
        Arc::new(move |key: &str| {
            let key = key.to_string();
            apply(&move |state: &mut FilesState| change(state, &key));
        })
    }

    /// The callback of a closure over the state.
    pub fn with(&self, change: impl Fn(&mut FilesState) + Send + Sync + 'static) -> Act {
        let apply = Arc::clone(&self.apply);
        Arc::new(move || apply(&change))
    }
}

/// A small button with an element id on a wrapper, so a host or a test can find it. Blinc's button is a
/// tab stop; the window turns Enter and Space on it into its click.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = Arc::clone(act);
    div().id(id).w_fit().flex_shrink_0().child(
        cn::button(label.to_string())
            .variant(variant)
            .size(ButtonSize::Small)
            .on_click(move |_| act()),
    )
}

/// A thing to press in a live area, where Blinc's button must not be built: a plain element and a tab
/// stop, which the window also presses with Enter and Space.
pub(super) fn plain_button(id: String, label: &str, act: &Act) -> Div {
    let act = Arc::clone(act);
    div()
        .id(id)
        .class("fs-files-press")
        .w_fit()
        .attr("tabindex", "0")
        .on_click(move |_| act())
        .child(blinc_layout::text(label).no_cursor())
}
