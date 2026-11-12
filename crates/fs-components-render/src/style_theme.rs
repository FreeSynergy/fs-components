//! A [`Style`] as a Blinc theme: the colours, corners, sizes and fonts of the style become the tokens the components read, so
//! the buttons, fields and tooltips of the component library look like the style (square corners in Craft, for one).
//!
//! Pattern: Adapter (`Style` to `blinc_theme::Theme`, one struct for every style). The shapes and colours are data of the style;
//! nothing about one style is written here, except which style the process uses ([`use_style`]).

use std::sync::{PoisonError, RwLock};

use blinc_core::Color;
use blinc_theme::{
    AnimationTokens, ColorScheme, ColorTokens, FontFamily, RadiusTokens, ShadowTokens,
    SpacingTokens, Theme, ThemeBundle, TypographyTokens,
};
use fs_components_styles::styles::craft;
use fs_components_styles::{Rgba, Style, StyleId, style};

static CHOSEN: RwLock<Option<StyleId>> = RwLock::new(None);

/// Chooses the style this process draws with, for the window and for pictures alike; `None` is the default theme of the
/// platform. Call it once at the start, before the window or a picture is made.
pub fn use_style(id: Option<StyleId>) {
    *CHOSEN.write().unwrap_or_else(PoisonError::into_inner) = id;
}

/// The style chosen with [`use_style`].
pub fn chosen_style() -> Option<StyleId> {
    *CHOSEN.read().unwrap_or_else(PoisonError::into_inner)
}

fn col(c: Rgba) -> Color {
    Color::rgba(
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
        f32::from(c.a) / 255.0,
    )
}

/// The theme of one appearance of a style.
#[derive(Debug, Clone)]
pub struct StyleTheme {
    name: String,
    scheme: ColorScheme,
    colors: ColorTokens,
    typography: TypographyTokens,
    spacing: SpacingTokens,
    radii: RadiusTokens,
    shadows: ShadowTokens,
    animations: AnimationTokens,
}

fn colors_of(style: &Style, dark: bool) -> ColorTokens {
    let p = style.palette(dark);
    let x = craft::extras(dark);
    let accent = col(p.accent);
    let soft = |c: Rgba, a: f32| col(c).with_alpha(a);
    ColorTokens {
        primary: accent,
        primary_hover: accent.with_alpha(0.85),
        primary_active: accent.with_alpha(0.7),
        secondary: col(p.border),
        secondary_hover: col(p.hover),
        secondary_active: col(p.selected),
        success: col(p.success),
        success_bg: soft(p.success, 0.12),
        warning: col(x.badge_update),
        warning_bg: soft(x.badge_update, 0.12),
        error: col(p.error),
        error_bg: soft(p.error, 0.14),
        info: col(x.badge_offer),
        info_bg: soft(x.badge_offer, 0.12),
        background: col(p.ground),
        surface: col(p.panel),
        surface_elevated: col(p.panel),
        surface_overlay: col(p.panel),
        text_primary: col(p.text),
        text_secondary: col(p.text_weak),
        text_tertiary: col(x.text_dim),
        text_inverse: col(p.on_accent),
        text_link: col(x.badge_offer),
        border: col(p.border),
        border_secondary: col(x.line_strong),
        border_hover: col(x.line_strong),
        border_focus: col(p.focus),
        border_error: col(p.error),
        input_bg: col(p.field),
        input_bg_hover: col(p.field),
        input_bg_focus: col(p.field),
        input_bg_disabled: col(p.hover),
        selection: soft(p.focus, 0.35),
        selection_text: col(p.text),
        accent,
        accent_subtle: soft(p.text, 0.06),
        tooltip_bg: col(x.deep),
        tooltip_text: col(p.text),
    }
}

fn typography_of(style: &Style) -> TypographyTokens {
    let sans = |fonts: &[&str]| {
        let mut rest: Vec<&str> = fonts[1..].to_vec();
        rest.push("sans-serif");
        FontFamily::new(fonts[0], rest)
    };
    TypographyTokens {
        font_sans: sans(style.fonts),
        font_mono: FontFamily::new(craft::MONO[0], craft::MONO[1..].to_vec()),
        text_xs: style.text.caption,
        text_sm: style.text.body,
        text_base: style.text.body,
        text_lg: style.text.title,
        text_xl: style.text.title + 2.0,
        text_2xl: style.text.headline,
        ..TypographyTokens::default()
    }
}

impl StyleTheme {
    /// The theme of `style` in the light or the dark appearance.
    pub fn new(style: &Style, dark: bool) -> Self {
        let r = style.radii;
        let defaults = RadiusTokens::default();
        Self {
            name: format!("fs-{}", style.id.name()),
            scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            colors: colors_of(style, dark),
            typography: typography_of(style),
            spacing: SpacingTokens::with_base(style.spacing.xs),
            radii: RadiusTokens {
                radius_none: 0.0,
                radius_sm: r.control,
                radius_default: r.control,
                radius_md: r.card,
                radius_lg: r.card,
                radius_xl: r.overlay,
                radius_2xl: r.overlay,
                radius_3xl: r.overlay,
                // a round thing (a switch, a dot) stays round in every style that has corners at all
                radius_full: if r.control == 0.0 && r.card == 0.0 {
                    0.0
                } else {
                    defaults.radius_full
                },
            },
            shadows: ShadowTokens {
                shadow_sm: Vec::new(),
                shadow_default: Vec::new(),
                shadow_md: Vec::new(),
                shadow_lg: Vec::new(),
                shadow_xl: Vec::new(),
                shadow_2xl: Vec::new(),
                shadow_inner: Vec::new(),
                shadow_none: Vec::new(),
            },
            animations: AnimationTokens::default(),
        }
    }
}

impl Theme for StyleTheme {
    fn name(&self) -> &str {
        &self.name
    }
    fn color_scheme(&self) -> ColorScheme {
        self.scheme
    }
    fn colors(&self) -> &ColorTokens {
        &self.colors
    }
    fn typography(&self) -> &TypographyTokens {
        &self.typography
    }
    fn spacing(&self) -> &SpacingTokens {
        &self.spacing
    }
    fn radii(&self) -> &RadiusTokens {
        &self.radii
    }
    fn shadows(&self) -> &ShadowTokens {
        &self.shadows
    }
    fn animations(&self) -> &AnimationTokens {
        &self.animations
    }
}

/// The bundle of a style: its light and dark theme, the component library's sheet and `css`. A style that is dark by nature
/// (Craft) has the same look in both appearances, so a system in light mode does not turn it light.
pub fn style_bundle(id: StyleId, css: &str) -> ThemeBundle {
    let s = style(id);
    let always_dark = id == StyleId::Craft;
    let light = StyleTheme::new(s, always_dark);
    let dark = StyleTheme::new(s, true);
    let mut bundle = ThemeBundle::new(format!("fs-{}", id.name()), light, dark);
    bundle
        .css_sources
        .push(blinc_cn::cn_styles::CN_STYLES.to_owned());
    if id == StyleId::Craft {
        bundle
            .css_sources
            .push(fs_components_styles::styles::craft_sheet::SHEET.to_owned());
    }
    bundle.css_sources.push(css.to_owned());
    bundle
}
