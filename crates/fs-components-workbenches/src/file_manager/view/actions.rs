//! What the person can do, as callbacks.
//!
//! Pattern: none of the table fits (plain closures, no listener trait) — the widget decides what each does: change the state and
//! draw again. A view asks for the callback of a transition by naming it.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::{ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::widgets::button as layout_button;
use fs_components_widgets::icon::{Glyph, Tone, icon};

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

/// A button whose words wrap inside `room` px instead of making the button wider (Blinc's button keeps its
/// label on one line). It is built the way `cn::button` builds its own (the same layout button, the same
/// classes, so the theme gives it the same look in every state), with a label that wraps.
pub(super) fn wrapping_button(
    id: String,
    label: &str,
    variant: ButtonVariant,
    act: &Act,
    room: f32,
) -> Div {
    let act = Arc::clone(act);
    let label = label.to_string();
    let kind = match variant {
        ButtonVariant::Primary => "cn-button--primary",
        ButtonVariant::Destructive => "cn-button--destructive",
        ButtonVariant::Outline => "cn-button--outline",
        ButtonVariant::Ghost => "cn-button--ghost",
        ButtonVariant::Link => "cn-button--link",
        ButtonVariant::Secondary => "cn-button--secondary",
    };
    let inner = layout_button::Button::with_content(move |_| {
        div()
            .padding_x_px(12.0)
            .padding_y_px(4.0)
            .max_w(room)
            .pointer_events_none()
            .child(
                blinc_layout::text(label.clone())
                    .size(13.0)
                    .pointer_events_none()
                    .no_cursor(),
            )
    })
    .key(format!("_fs_files_btn_{id}"))
    .items_center()
    .justify_center()
    .class("cn-button")
    .class("tabindex=0")
    .class(kind)
    .class("cn-button--sm")
    .on_click(move |_| act());
    div()
        .id(id)
        .w_fit()
        .flex_shrink_0()
        .max_w(room)
        .child(inner)
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

/// A button that is only an icon, with its words as a tooltip. `on` shows it pressed (a view that is the
/// one in force), `enabled` false shows it dimmed and does nothing. A tab stop that the window presses with
/// Enter and Space, like the other buttons.
pub(super) fn icon_button(
    id: String,
    glyph: Glyph,
    tip: &str,
    (on, enabled): (bool, bool),
    act: &Act,
) -> Div {
    let act = Arc::clone(act);
    let key = blinc_layout::InstanceKey::explicit(format!("fs-files-tip-{id}"));
    let trigger = move || {
        let mut face = div()
            .class("fs-files-icon-btn")
            .flex_row()
            .items_center()
            .justify_center()
            .w(ICON_BUTTON)
            .h(ICON_BUTTON)
            .child(div().pointer_events_none().child(icon(
                glyph,
                20.0,
                if enabled { Tone::Normal } else { Tone::Muted },
            )));
        if on {
            face = face.class("fs-files-icon-btn-on");
        }
        if !enabled {
            face = face.class("fs-files-icon-btn-off");
        }
        face
    };
    let with_tip =
        blinc_cn::components::tooltip::TooltipBuilder::with_key(trigger, key).text(tip.to_string());
    div()
        .id(id)
        .flex_shrink_0()
        .w(ICON_BUTTON)
        .h(ICON_BUTTON)
        .attr("tabindex", "0")
        .on_click(move |_| {
            if enabled {
                act();
            }
        })
        .child(with_tip)
}

/// The width and height of an icon button.
pub(in crate::file_manager) const ICON_BUTTON: f32 = 36.0;
