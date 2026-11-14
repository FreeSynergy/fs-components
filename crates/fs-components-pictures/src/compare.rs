//! Pixel comparison of two pictures with a tolerance set per group.
//!
//! Pattern: Value objects (`Tolerance`, `Report`) and one pure function; no drawing, no files, so it runs without a GPU.

use image::{Rgba, RgbaImage};

/// What may differ between a reference and a new picture without failing a group.
///
/// A pixel counts as *different* when it matches no pixel of the other picture within `shift` pixels (each
/// channel within `channel`). `shift` is how a rounding of a scale (at most 2 px) is allowed; 0 compares in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tolerance {
    /// Largest difference of one channel that still counts as equal (0 to 255).
    pub channel: u8,
    /// How far a pixel may have moved and still count as equal (pixels).
    pub shift: u32,
    /// How many different pixels the group accepts.
    pub pixels: usize,
}

impl Tolerance {
    /// Exact: every pixel equal in place.
    pub const EXACT: Self = Self {
        channel: 0,
        shift: 0,
        pixels: 0,
    };
}

impl Tolerance {
    /// For a block with text in the system font (the chart text uses the default sans of the machine, not the bundled
    /// font): a channel step up to 64, a move of 1 px, and at most 900 of 64000 pixels (1.4 %) outside that.
    ///
    /// Measured (charts group, 320x200): the same drawing with only Noto Sans or only `DejaVu` Sans installed differs from
    /// the reference by at most 621 pixels at this setting (CI runner: 3723 and 2787 pixels at EXACT); a move of 3 px
    /// gives 1163 and more, a wrong colour or an inverted picture nearly every pixel (`tests/tolerance_counter.rs`).
    pub const SYSTEM_FONT: Self = Self {
        channel: 64,
        shift: 1,
        pixels: 900,
    };

    /// A large board with much text in the system font (kanban, 480x260 = 124800 pixels): channel 96, 1 px, at most
    /// 1600 pixels (1.3 %). Measured with one font only: at most 1076; a move of 3 px gives 2287 and more.
    pub const SYSTEM_FONT_BOARD: Self = Self {
        channel: 96,
        shift: 1,
        pixels: 1600,
    };

    /// A small picture with one line of text in the system font (search field, 240x80): channel 64, 1 px, at most 35
    /// pixels. Measured with one font only: at most 23; a move of 3 px gives 42 and more.
    pub const SYSTEM_FONT_SMALL: Self = Self {
        channel: 64,
        shift: 1,
        pixels: 35,
    };
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::EXACT
    }
}

/// The result of comparing two pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Sizes differ: nothing else was compared.
    pub size_mismatch: Option<((u32, u32), (u32, u32))>,
    /// Pixels that match nothing within the tolerance.
    pub differing: usize,
    /// Largest difference of any channel over all pixels compared in place (0 when equal).
    pub max_channel: u8,
    /// Where the first different pixel is.
    pub first: Option<(u32, u32)>,
}

impl Report {
    /// Is the picture within `tolerance`?
    pub fn within(&self, tolerance: Tolerance) -> bool {
        self.size_mismatch.is_none() && self.differing <= tolerance.pixels
    }

    /// One line for a failure message.
    pub fn describe(&self) -> String {
        if let Some((a, b)) = self.size_mismatch {
            return format!("size {}x{} against {}x{}", a.0, a.1, b.0, b.1);
        }
        format!(
            "{} different pixels, largest channel difference {}, first at {:?}",
            self.differing, self.max_channel, self.first
        )
    }
}

fn close(a: Rgba<u8>, b: Rgba<u8>, channel: u8) -> bool {
    a.0.iter().zip(b.0).all(|(x, y)| x.abs_diff(y) <= channel)
}

fn matches_near(a: &RgbaImage, b: &RgbaImage, (x, y): (u32, u32), t: Tolerance) -> bool {
    let p = *a.get_pixel(x, y);
    let (w, h) = a.dimensions();
    let s = i64::from(t.shift);
    for dy in -s..=s {
        for dx in -s..=s {
            let (nx, ny) = (i64::from(x) + dx, i64::from(y) + dy);
            if nx < 0 || ny < 0 || nx >= i64::from(w) || ny >= i64::from(h) {
                continue;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // bounds checked just above
            if close(p, *b.get_pixel(nx as u32, ny as u32), t.channel) {
                return true;
            }
        }
    }
    false
}

/// Is the pixel at `at` the same within the tolerance, looking both ways?
fn agrees(reference: &RgbaImage, new: &RgbaImage, at: (u32, u32), t: Tolerance) -> bool {
    matches_near(new, reference, at, t) && matches_near(reference, new, at, t)
}

/// Compare `new` with `reference`. A pixel differs when it is not equal in place and either the pixel of `new` has no
/// match in the reference or the pixel of the reference has none in `new` (the test is symmetric), within `tolerance`.
pub fn compare(reference: &RgbaImage, new: &RgbaImage, tolerance: Tolerance) -> Report {
    if reference.dimensions() != new.dimensions() {
        return Report {
            size_mismatch: Some((reference.dimensions(), new.dimensions())),
            differing: usize::MAX,
            max_channel: 255,
            first: None,
        };
    }
    let (w, h) = reference.dimensions();
    let (mut differing, mut max_channel, mut first) = (0, 0, None);
    for y in 0..h {
        for x in 0..w {
            let (a, b) = (reference.get_pixel(x, y), new.get_pixel(x, y));
            let d =
                a.0.iter()
                    .zip(b.0)
                    .map(|(p, q)| p.abs_diff(q))
                    .max()
                    .unwrap_or(0);
            max_channel = max_channel.max(d);
            if d > tolerance.channel && !agrees(reference, new, (x, y), tolerance) {
                differing += 1;
                first.get_or_insert((x, y));
            }
        }
    }
    Report {
        size_mismatch: None,
        differing,
        max_channel,
        first,
    }
}

/// A picture of the difference: the new picture dimmed, every different pixel in red. Same size as `new`.
pub fn diff_image(reference: &RgbaImage, new: &RgbaImage, tolerance: Tolerance) -> RgbaImage {
    let mut out = RgbaImage::new(new.width(), new.height());
    for (x, y, p) in new.enumerate_pixels() {
        let same = x < reference.width()
            && y < reference.height()
            && (close(*p, *reference.get_pixel(x, y), tolerance.channel)
                || agrees(reference, new, (x, y), tolerance));
        let dim = |c: u8| c / 3 + 120;
        out.put_pixel(
            x,
            y,
            if same {
                Rgba([dim(p.0[0]), dim(p.0[1]), dim(p.0[2]), 255])
            } else {
                Rgba([255, 0, 0, 255])
            },
        );
    }
    out
}
