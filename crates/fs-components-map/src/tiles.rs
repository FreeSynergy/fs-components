//! Which tiles a view needs, and where each lies on the screen.
//!
//! Pattern: Pure functions — from a position, a size and the zoom range of the caller's source to a
//! list of tiles with their pixel rectangles. Tiles outside the world's rows are left out, columns
//! wrap (the same tile appears again where the world repeats), and the list is capped so that no
//! window size or zoom can ask for an endless number of tiles.

use super::mercator::{TILE_SIZE, clamp_zoom, finite_or, project};
use super::position::MapPosition;

/// The most tiles one view asks for.
pub const MOST_TILES: usize = 64;

/// A tile of the usual `z/x/y` scheme (y downwards).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileId {
    /// Zoom.
    pub z: u8,
    /// Column, west to east.
    pub x: u32,
    /// Row, north to south.
    pub y: u32,
}

/// A tile a view needs and the rectangle it covers on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeededTile {
    /// Which tile.
    pub id: TileId,
    /// Left edge on the screen, in pixels (may be negative: partly out of view).
    pub left: f64,
    /// Top edge on the screen.
    pub top: f64,
    /// Side on the screen; tiles are square.
    pub size: f64,
}

/// The tiles a view needs.
#[derive(Clone, Debug, PartialEq)]
pub struct TileSet {
    /// The tiles, the ones nearest the centre first.
    pub tiles: Vec<NeededTile>,
    /// More were needed than [`MOST_TILES`]; the farthest from the centre were left out.
    pub truncated: bool,
}

/// The zoom of the tiles to draw at `zoom`: the whole zoom below it, inside what the source has.
#[must_use]
pub fn tile_zoom(zoom: f64, min_zoom: u8, max_zoom: u8) -> u8 {
    let (low, high) = (min_zoom.min(max_zoom), max_zoom.max(min_zoom));
    let whole = clamp_zoom(zoom).floor();
    // `whole` is in 0..=22.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = whole as u8;
    whole.clamp(low, high)
}

/// The tiles that cover a view `width` x `height` pixels at `position`, from a source with tiles
/// between `min_zoom` and `max_zoom`. A view of no size needs none.
#[must_use]
pub fn needed_tiles(
    position: MapPosition,
    width: f64,
    height: f64,
    min_zoom: u8,
    max_zoom: u8,
) -> TileSet {
    let (width, height) = (finite_or(width, 0.0), finite_or(height, 0.0));
    let empty = TileSet {
        tiles: Vec::new(),
        truncated: false,
    };
    if width < 1.0 || height < 1.0 {
        return empty;
    }
    let z = tile_zoom(position.zoom(), min_zoom, max_zoom);
    let size = TILE_SIZE * 2f64.powf(position.zoom() - f64::from(z));
    let (cx, cy) = project(position.lon(), position.lat(), position.zoom());
    let count = 2f64.powi(i32::from(z));
    let (left, right) = ((cx - width / 2.0) / size, (cx + width / 2.0) / size);
    let (top, bottom) = ((cy - height / 2.0) / size, (cy + height / 2.0) / size);
    let (first_col, last_col) = (left.floor(), right.floor());
    let first_row = top.floor().max(0.0);
    let last_row = bottom.floor().min(count - 1.0);
    if last_row < first_row {
        return empty;
    }
    // Counted before anything is made: a huge view must not build a huge list.
    let columns = (last_col - first_col + 1.0).min(1.0e6);
    let rows = last_row - first_row + 1.0;
    let mut tiles = Vec::new();
    let keep = MOST_TILES as f64;
    let wanted = columns * rows;
    let mut col = first_col;
    // Walk outwards from the centre so that, when capped, the middle stays.
    let (mid_col, mid_row) = (
        (cx / size).floor(),
        (cy / size).floor().clamp(first_row, last_row),
    );
    let mut push = |col: f64, row: f64| {
        let wrapped = col.rem_euclid(count);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        tiles.push(NeededTile {
            id: TileId {
                z,
                x: wrapped as u32,
                y: row as u32,
            },
            left: width / 2.0 + col * size - cx,
            top: height / 2.0 + row * size - cy,
            size,
        });
    };
    if wanted <= keep * 4.0 {
        while col <= last_col {
            let mut row = first_row;
            while row <= last_row {
                push(col, row);
                row += 1.0;
            }
            col += 1.0;
        }
    } else {
        // Far more than allowed: only a square around the centre is walked.
        let reach = (keep.sqrt() / 2.0).ceil();
        let mut col = (mid_col - reach).max(first_col);
        while col <= (mid_col + reach).min(last_col) {
            let mut row = (mid_row - reach).max(first_row);
            while row <= (mid_row + reach).min(last_row) {
                push(col, row);
                row += 1.0;
            }
            col += 1.0;
        }
    }
    let centre = |t: &NeededTile| {
        let (dx, dy) = (
            t.left + t.size / 2.0 - width / 2.0,
            t.top + t.size / 2.0 - height / 2.0,
        );
        dx * dx + dy * dy
    };
    tiles.sort_by(|a, b| centre(a).total_cmp(&centre(b)));
    let truncated = tiles.len() > MOST_TILES || wanted > tiles.len() as f64;
    tiles.truncate(MOST_TILES);
    TileSet { tiles, truncated }
}
