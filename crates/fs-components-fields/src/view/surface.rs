//! A surface as the style draws it: flat, glass, or tinted backdrop.
//!
//! Pattern: Adapter — `fs_components_styles::Surface` data in, Blinc background or material out.
//! Backdrop (Mica) is Unsupported: Blinc cannot read the desktop wallpaper, so it is drawn as the
//! flat tint over the palette's panel colour, and `Style::demands()` still reports `Backdrop`.

use blinc_layout::div::Div;
use blinc_layout::element::GlassMaterial;
use fs_components_styles::Surface;

use super::Look;

impl Look {
    /// `ground` drawn as the chrome (window, sidebar, toolbar) or the overlay (popover, dialog)
    /// surface of the style: the panel colour for a flat one, glass for a glass one.
    pub fn surface(&self, ground: Div, overlay: bool) -> Div {
        let effects = self.style.effects(self.dark);
        let surface = if overlay {
            effects.overlay
        } else {
            effects.chrome
        };
        let panel = self.palette().panel;
        match surface {
            Surface::Solid => ground.bg(Self::color(panel)),
            Surface::Glass {
                blur,
                tint,
                saturation,
            } => ground.effect(
                GlassMaterial::new()
                    .blur(blur)
                    .tint(Self::color(tint))
                    .saturation(saturation),
            ),
            Surface::Backdrop { tint } => ground.bg(Self::color(over(tint, panel))),
        }
    }
}

/// `top` laid over the opaque `base` (straight alpha), opaque.
fn over(
    top: fs_components_styles::Rgba,
    base: fs_components_styles::Rgba,
) -> fs_components_styles::Rgba {
    let a = u32::from(top.a);
    let mix = |t: u8, b: u8| ((u32::from(t) * a + u32::from(b) * (255 - a)) / 255) as u8;
    fs_components_styles::Rgba {
        r: mix(top.r, base.r),
        g: mix(top.g, base.g),
        b: mix(top.b, base.b),
        a: 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_components_styles::Rgba;

    #[test]
    fn over_mixes_by_alpha() {
        let got = over(Rgba::hex(0xFFFFFF).alpha(128), Rgba::hex(0));
        assert_eq!((got.r, got.a), (128, 255));
        assert_eq!(
            over(Rgba::hex(0x123456), Rgba::hex(0xFFFFFF)),
            Rgba::hex(0x123456)
        );
    }
}
