//! Colour spaces: sRGB, linear light, CIE Lab, and WCAG contrast.
//!
//! Pattern: Value objects and pure conversions (standard formulas: IEC 61966-2-1 for sRGB, CIE
//! 1976 for Lab with the D65 white point, WCAG 2.x for relative luminance and contrast).

/// A colour in sRGB, each channel 0..=1.
pub type Rgb = [f32; 3];

fn to_linear(c: f32) -> f64 {
    let c = f64::from(c.clamp(0.0, 1.0));
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(c: f64) -> f32 {
    let c = c.clamp(0.0, 1.0);
    (if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }) as f32
}

/// Linear-light components.
pub fn linear(c: Rgb) -> [f64; 3] {
    [to_linear(c[0]), to_linear(c[1]), to_linear(c[2])]
}

/// Back to sRGB, clamped to the gamut.
pub fn from_linear_rgb(l: [f64; 3]) -> Rgb {
    [from_linear(l[0]), from_linear(l[1]), from_linear(l[2])]
}

const WHITE: [f64; 3] = [0.950_47, 1.0, 1.088_83];

fn f(t: f64) -> f64 {
    if t > 216.0 / 24_389.0 {
        t.cbrt()
    } else {
        (24_389.0 / 27.0 * t + 16.0) / 116.0
    }
}

fn f_inv(t: f64) -> f64 {
    if t * t * t > 216.0 / 24_389.0 {
        t * t * t
    } else {
        (116.0 * t - 16.0) / (24_389.0 / 27.0)
    }
}

/// CIE L*a*b* of linear light.
pub fn lab_of_linear(l: [f64; 3]) -> [f64; 3] {
    let x = 0.412_456_4 * l[0] + 0.357_576_1 * l[1] + 0.180_437_5 * l[2];
    let y = 0.212_672_9 * l[0] + 0.715_152_2 * l[1] + 0.072_175_0 * l[2];
    let z = 0.019_333_9 * l[0] + 0.119_192_0 * l[1] + 0.950_304_1 * l[2];
    let (fx, fy, fz) = (f(x / WHITE[0]), f(y / WHITE[1]), f(z / WHITE[2]));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIE L*a*b* of an sRGB colour.
pub fn lab(c: Rgb) -> [f64; 3] {
    lab_of_linear(linear(c))
}

/// sRGB of a Lab colour (clamped to the gamut).
pub fn rgb_of_lab(lab: [f64; 3]) -> Rgb {
    let fy = (lab[0] + 16.0) / 116.0;
    let (fx, fz) = (fy + lab[1] / 500.0, fy - lab[2] / 200.0);
    let (x, y, z) = (
        f_inv(fx) * WHITE[0],
        f_inv(fy) * WHITE[1],
        f_inv(fz) * WHITE[2],
    );
    from_linear_rgb([
        3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z,
        -0.969_266_0 * x + 1.876_010_8 * y + 0.041_556_0 * z,
        0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z,
    ])
}

/// Colour difference, CIE 1976 (Euclidean in Lab).
pub fn delta_e(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// WCAG relative luminance.
pub fn luminance(c: Rgb) -> f64 {
    let l = linear(c);
    0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2]
}

/// WCAG contrast ratio, 1..=21.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}
