//! The small parts of the Craft look: capital mono labels, square buttons, joined tabs, badges, numbered boxes, the progress bar,
//! an icon cell, a fact row.
//!
//! Pattern: Adapter (state to elements). Every part takes its look from the `craft-*` classes of `fs-components-styles` and the
//! theme variables; nothing is drawn here. Clickable parts are tab stops that Enter and Space press (a click handler on a tab stop).

use std::sync::Arc;

use blinc_cn::components::tooltip::TooltipBuilder;
use blinc_layout::InstanceKey;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_styles::Rgba;
use fs_components_styles::styles::craft::{ACCENTS, ACCENTS_TEXT, accent_index};
use fs_components_styles::styles::craft_sheet::caps;

use crate::icon::{Glyph, Tone, icon};

/// What a press does.
pub type Act = Arc<dyn Fn() + Send + Sync>;

/// A small label in capitals, mono, spaced (the sheet gives face, size and spacing).
pub fn label(words: &str) -> impl crate::Element + use<> {
    text(caps(words)).class("craft-caps").no_wrap()
}

/// The same, dim.
pub fn label_dim(words: &str) -> impl crate::Element + use<> {
    text(caps(words)).class("craft-caps-dim").no_wrap()
}

/// How a square button looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// White, black capitals.
    Fill,
    /// An outline.
    Line,
}

/// A square button: capitals in the mono face, 34 px high. `enabled` false dims it and does nothing (a locked part says why as a tooltip).
pub fn button(id: &str, words: &str, look: Look, enabled: bool, act: Act) -> Div {
    let mut b = div()
        .id(id.to_owned())
        .class("craft-btn")
        .flex_row()
        .items_center()
        .justify_center()
        .px(3.5)
        .attr("tabindex", "0")
        .child(match look {
            Look::Fill => text(caps(words)).class("craft-btn-fill-text").no_wrap(),
            Look::Line => text(caps(words)).class("craft-caps").no_wrap(),
        });
    b = match look {
        Look::Fill => b.class("craft-btn-fill"),
        Look::Line => b.class("craft-btn-line"),
    };
    if !enabled {
        b = b.class("craft-btn-off");
    }
    b.on_click(move |_| {
        if enabled {
            act();
        }
    })
}

/// A line of sub text (`shown`) with `tip` as its tooltip: the whole text when the line was cut.
pub fn with_tip(id: &str, shown: &str, tip: &str) -> Div {
    let key = InstanceKey::explicit(format!("fs-store-sum-{id}"));
    let shown = shown.to_owned();
    let trigger = move || div().child(text(shown.clone()).class("craft-sub").no_wrap());
    div().child(TooltipBuilder::with_key(trigger, key).text(tip.to_owned()))
}

/// A square cell that holds an icon or a few characters (refresh, pause, the three dots), 34 px high.
pub fn cell_button(id: &str, glyph: Glyph, tip: &str, enabled: bool, act: Act) -> Div {
    let key = InstanceKey::explicit(format!("fs-store-tip-{id}"));
    let tone = if enabled { Tone::Normal } else { Tone::Muted };
    let trigger = move || {
        let mut face = div()
            .class("craft-btn")
            .flex_row()
            .items_center()
            .justify_center()
            .w(36.0)
            .child(div().pointer_events_none().child(icon(glyph, 16.0, tone)));
        if !enabled {
            face = face.class("craft-btn-off");
        }
        face
    };
    div()
        .id(id.to_owned())
        .flex_shrink_0()
        .attr("tabindex", "0")
        .on_click(move |_| {
            if enabled {
                act();
            }
        })
        .child(TooltipBuilder::with_key(trigger, key).text(tip.to_owned()))
}

/// One tab of a joined row of tabs: the one in force is white with black capitals; `count` is shown dim after the words.
pub fn tab(id: &str, words: &str, count: Option<usize>, on: bool, act: Act) -> Div {
    let mut t = div()
        .id(id.to_owned())
        .class("craft-tab")
        .flex_row()
        .items_center()
        .gap(1.5)
        .attr("tabindex", "0");
    if on {
        t = t
            .class("craft-tab-on")
            .child(text(caps(words)).class("craft-pill-text").no_wrap());
        if let Some(n) = count {
            t = t.child(text(n.to_string()).class("craft-pill-text").no_wrap());
        }
    } else {
        t = t.child(label(words));
        if let Some(n) = count {
            t = t.child(text(n.to_string()).class("craft-caps-dim").no_wrap());
        }
    }
    t.on_click(move |_| act())
}

/// A row of tabs inside one box with a line around it (the last tab has no line to its right).
pub fn tabs(items: Vec<Div>) -> Div {
    let mut row = div().class("craft-box").flex_row().flex_shrink_0();
    for item in items {
        row = row.child(item);
    }
    row
}

/// The colour of a badge's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    /// Done, fine (green).
    Ok,
    /// Waiting, take care (amber).
    Warn,
    /// Offered, going on (blue).
    Info,
    /// Wrong, missing (red).
    Bad,
    /// Nothing to note (the weak text colour).
    Quiet,
}

impl BadgeTone {
    fn class(self) -> &'static str {
        match self {
            BadgeTone::Ok => "craft-t-ok",
            BadgeTone::Warn => "craft-t-warn",
            BadgeTone::Info => "craft-t-info",
            BadgeTone::Bad => "craft-t-bad",
            BadgeTone::Quiet => "craft-t-quiet",
        }
    }
}

/// A badge: a small box with a 1 px line and a deep ground, the words in capitals in the colour of the state.
pub fn badge(words: &str, tone: BadgeTone) -> Div {
    div().class("craft-badge").flex_shrink_0().child(
        text(caps(words))
            .class("craft-caps")
            .class(tone.class())
            .no_wrap(),
    )
}

fn colour(c: Rgba) -> blinc_core::Color {
    blinc_core::Color::rgb(
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
    )
}

/// The accent colour of a name (the box of a tile, the bar of its progress): the same name always has the same one.
pub fn accent(name: &str) -> Rgba {
    ACCENTS[accent_index(name)]
}

/// The lighter shade of the accent of a name, for words on the dark ground (the second half of a title).
pub fn accent_text(name: &str) -> Rgba {
    ACCENTS_TEXT[accent_index(name)]
}

/// An accent as a Blinc colour, for `.color(...)` and `.bg(...)`.
pub fn blinc_colour(c: Rgba) -> blinc_core::Color {
    colour(c)
}

/// The numbered box of a tile: `number` in a block of `accent` (black bold digits).
pub fn number_box(number: usize, accent: Rgba) -> Div {
    div()
        .class("craft-number")
        .flex_row()
        .items_center()
        .justify_center()
        .bg(colour(accent))
        .child(
            text(format!("{number:02}"))
                .class("craft-number-text")
                .no_wrap(),
        )
}

/// A thin bar `share` (0..1) full, `width` pixels wide, filled in `accent`.
pub fn progress(share: f32, width: f32, accent: Rgba) -> Div {
    div().class("craft-progress-track").w(width).child(
        div()
            .h(4.0)
            .w((width * share.clamp(0.0, 1.0)).max(1.0))
            .bg(colour(accent)),
    )
}

/// A label above its value: the label in dim capitals (long labels and long values both fit).
pub fn fact(name: &str, value: &str) -> Div {
    div()
        .flex_col()
        .gap(1.0)
        .child(label_dim(name))
        .child(text(value.to_owned()))
}

/// A heading of a block: dim capitals.
pub fn heading(words: &str) -> Div {
    div().class("craft-section").child(label_dim(words))
}
