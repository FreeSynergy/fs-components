//! A colour as four bytes.
//!
//! Pattern: none (plain data, constants, pure functions). No Blinc type here: a style is data, the
//! views convert.

/// Red, green, blue, alpha, each 0..=255 (sRGB, alpha not premultiplied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha, 255 is opaque.
    pub a: u8,
}

impl Rgba {
    /// Opaque colour from `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Self {
            r: (rgb >> 16) as u8,
            g: (rgb >> 8) as u8,
            b: rgb as u8,
            a: 255,
        }
    }

    /// The same colour with another alpha (0..=255).
    #[must_use]
    pub const fn alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Fully transparent.
    pub const CLEAR: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// `#rrggbb`, or `#rrggbbaa` when not opaque (CSS Color 4 notation).
    pub fn css(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Relative luminance, WCAG 2.x (0.0 black .. 1.0 white), for contrast checks.
    pub fn luminance(self) -> f32 {
        fn lin(c: u8) -> f32 {
            let c = f32::from(c) / 255.0;
            if c <= 0.039_28 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * lin(self.r) + 0.7152 * lin(self.g) + 0.0722 * lin(self.b)
    }

    /// WCAG contrast ratio between two colours (1.0 .. 21.0); alpha is ignored.
    pub fn contrast(self, other: Self) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_and_css_round_trip() {
        assert_eq!(Rgba::hex(0x0070F2).css(), "#0070f2");
        assert_eq!(Rgba::hex(0xFFFFFF).alpha(128).css(), "#ffffff80");
    }

    #[test]
    fn contrast_black_on_white_is_21() {
        let c = Rgba::hex(0).contrast(Rgba::hex(0xFFFFFF));
        assert!((c - 21.0).abs() < 0.01, "{c}");
    }
}
