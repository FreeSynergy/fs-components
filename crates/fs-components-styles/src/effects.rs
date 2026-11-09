//! Surfaces, shadows and the focus ring of a style.
//!
//! Pattern: none (plain data). What an effect asks of the GUI engine is told by
//! [`crate::Style::demands`], so a measurement can check each one.

use crate::color::Rgba;

/// How a surface is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Surface {
    /// A flat colour from the palette.
    Solid,
    /// Frosted glass: the picture behind is blurred and tinted.
    Glass {
        /// Blur radius in pixels.
        blur: f32,
        /// Tint laid over the blurred picture (alpha gives the strength).
        tint: Rgba,
        /// Saturation of the picture behind (1.0 unchanged).
        saturation: f32,
    },
    /// A material that takes colour from the desktop wallpaper, flat and slowly changing (Mica
    /// in Fluent). Where the engine cannot read the wallpaper it is [`Surface::Solid`].
    Backdrop {
        /// Tint laid over the wallpaper colour.
        tint: Rgba,
    },
}

/// A shadow below a surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// Horizontal offset.
    pub dx: f32,
    /// Vertical offset.
    pub dy: f32,
    /// Blur radius.
    pub blur: f32,
    /// Growth before blurring.
    pub spread: f32,
    /// Colour with alpha.
    pub color: Rgba,
}

/// The ring drawn around the control that has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRing {
    /// Line width.
    pub width: f32,
    /// Gap between control and ring.
    pub offset: f32,
}

/// All effects of a style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effects {
    /// Sidebars, toolbars, window background.
    pub chrome: Surface,
    /// Popovers, menus, dialogs.
    pub overlay: Surface,
    /// Shadow of a card (none for flat styles).
    pub card_shadow: Option<Shadow>,
    /// Shadow of a popover or dialog.
    pub overlay_shadow: Option<Shadow>,
    /// Focus ring.
    pub focus: FocusRing,
}
