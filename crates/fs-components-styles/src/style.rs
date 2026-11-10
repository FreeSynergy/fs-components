//! A style as a whole, and the way to find one.
//!
//! Pattern: Registry (`StyleId` to `&'static Style`) behind the trait [`StyleCatalog`]; `Style` is a
//! value object. Why: a frontend names a style, the views read it; a later style (5 to 10 are
//! planned, the FreeSynergy style among them) is one more entry, not a change to any view.

use crate::effects::{Effects, Surface};
use crate::icons::IconSet;
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

/// The styles there are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleId {
    /// Dense business data, SAP Fiori (Horizon) measures; generated admin frontends.
    Admin,
    /// Few large things, high contrast; for people with little experience.
    Simple,
    /// Measures after Apple's design language.
    Apple,
    /// Measures after Microsoft's Fluent 2.
    Windows,
}

impl StyleId {
    /// All styles in a fixed order.
    pub const ALL: [StyleId; 4] = [
        StyleId::Admin,
        StyleId::Simple,
        StyleId::Apple,
        StyleId::Windows,
    ];

    /// Stable name for settings files and tests.
    pub const fn name(self) -> &'static str {
        match self {
            StyleId::Admin => "admin",
            StyleId::Simple => "simple",
            StyleId::Apple => "apple",
            StyleId::Windows => "windows",
        }
    }

    /// The id with this name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// Where the numbers of a style come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Every number is in a measurement sheet (`docs/en/components/styles/<style>.md`) or is our own design.
    Sheet,
    /// The sheet gives some numbers (sizes, spacing, radii, focus); the rest (named in the style's file) was
    /// written from the public guideline by memory and waits for a measurement from the programs' screenshots.
    Partial,
}

/// Things a style asks of the GUI engine; each can be measured and reported when it cannot be done.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Demand {
    /// Blur of what is behind a surface, radius in pixels.
    Glass(f32),
    /// A surface tinted by the wallpaper.
    Backdrop,
    /// A soft shadow with this blur radius.
    Shadow(f32),
    /// List rows of exactly this height.
    RowHeight(f32),
    /// A one-line control of exactly this height.
    ControlHeight(f32),
    /// A focus ring outside the control.
    FocusRing,
    /// A font family that must be available (the first of the list).
    Font(&'static str),
}

/// One complete style.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// Which style this is.
    pub id: StyleId,
    /// Where the numbers come from.
    pub provenance: Provenance,
    /// Font families in order of preference (all open licences).
    pub fonts: &'static [&'static str],
    /// Gaps.
    pub spacing: Spacing,
    /// Heights.
    pub heights: Heights,
    /// Font sizes.
    pub text: Type,
    /// Corners.
    pub radii: Radii,
    /// Colours, light.
    pub light: Palette,
    /// Colours, dark.
    pub dark: Palette,
    /// Surfaces, shadows, focus.
    pub effects: Effects,
    /// Icons.
    pub icons: IconSet,
}

impl Style {
    /// The palette for the appearance.
    pub fn palette(&self, dark: bool) -> &Palette {
        if dark { &self.dark } else { &self.light }
    }

    /// What this style asks of the engine, one entry per distinct request.
    pub fn demands(&self) -> Vec<Demand> {
        let mut d = vec![
            Demand::Font(self.fonts[0]),
            Demand::ControlHeight(self.heights.control),
            Demand::RowHeight(self.heights.row),
            Demand::FocusRing,
        ];
        for surface in [self.effects.chrome, self.effects.overlay] {
            match surface {
                Surface::Solid => {}
                Surface::Glass { blur, .. } => d.push(Demand::Glass(blur)),
                Surface::Backdrop { .. } => d.push(Demand::Backdrop),
            }
        }
        for s in [self.effects.card_shadow, self.effects.overlay_shadow]
            .into_iter()
            .flatten()
        {
            d.push(Demand::Shadow(s.blur));
        }
        d.dedup();
        d
    }
}

/// Something that knows styles.
pub trait StyleCatalog {
    /// The style with this id, if the catalog has it.
    fn get(&self, id: StyleId) -> Option<&Style>;
}

/// The four styles built in.
#[derive(Debug, Clone, Copy, Default)]
pub struct Builtin;

impl StyleCatalog for Builtin {
    fn get(&self, id: StyleId) -> Option<&Style> {
        Some(style(id))
    }
}

/// The built-in style with this id.
pub fn style(id: StyleId) -> &'static Style {
    use std::sync::OnceLock;
    static ALL: OnceLock<[Style; 4]> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        [
            crate::styles::admin::build(),
            crate::styles::simple::build(),
            crate::styles::apple::build(),
            crate::styles::windows::build(),
        ]
    });
    match id {
        StyleId::Admin => &all[0],
        StyleId::Simple => &all[1],
        StyleId::Apple => &all[2],
        StyleId::Windows => &all[3],
    }
}
