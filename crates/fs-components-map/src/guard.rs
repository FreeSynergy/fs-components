//! What a tile may cost, looked at before `mvt-reader` decodes it.
//!
//! Pattern: Guard (a check before the work). `mvt-reader` 2.5.0 (`parse_geometry`, lib.rs) reserves a buffer of the
//! size of the WHOLE geometry of a feature (`G` numbers, 8 bytes a point) at the start, again at every `ClosePath`
//! command, and again at every `MoveTo` of a line. A feature with `C` such commands therefore asks for `(C + 1) * G`
//! points (quadratic in its size), and what a layer decodes stays in memory until the layer is done: the cost of a
//! tile is the SUM over all its features. Running out of memory is no panic that `catch_unwind` catches (on Windows,
//! where the system must really promise the memory, it ends the process). The tile is therefore scanned first, in its
//! raw protobuf form, with almost no allocation (the sizes of the keys and values of a layer): its size, the number of
//! its features, and the estimated memory of the whole tile, geometry and tags together; the extent of each layer is
//! read too. Numbers are read as `u32` like the decoder reads them (it cuts a longer number to its low 32 bits).
//!
//! Everything else the decoder makes is counted too: layers, keys, values and features have limits and a cost each,
//! and a point of a feature costs its own copy of the feature's texts (`add_point` clones the name for every point of
//! a `MultiPoint`: 20 000 points and a 1 MiB name are 20 GB from a tile of 1 MB).
//!
//! Tags: `parse_tags` (lib.rs:305) copies the key and the value of every tag pair for every feature, so a value
//! string of 1 MB named by 400 features is 400 MB (4 MiB of tile and 100 000 features would be 300 GB). A tile over a limit is refused, never decoded.
//!
//! Evidence (no memory is used by the tests): a tile of 1 MB made of `LineTo`+`ClosePath` repeated 250 000 times is
//! one feature with 250 000 `ClosePath` and 1 000 000 numbers, so `mvt-reader` reserves 250 000 buffers of 1 000 000
//! points of 8 bytes: 2 TB. The estimate here is `(C + 1) * G * 8` = 2e12 bytes; the limit is 64 MiB.

use crate::mvt::MvtError;
use crate::wire::{Cursor, Value, low32, too_much};

/// The most bytes a tile may have (a vector tile is some tens of kilobytes; busy ones a few hundred).
pub const MAX_TILE_BYTES: usize = 4 * 1024 * 1024;
/// The most features of all layers of one tile.
pub const MAX_FEATURES: usize = 100_000;
/// The most memory (bytes, estimated) `mvt-reader` may take for all features of one tile together: the geometry
/// buffers (8 bytes a point) and the copies of the tags (every value string once for every feature that names it).
pub const MAX_TILE_MEMORY: u64 = 64 * 1024 * 1024;
/// The rule: no byte of input makes more than this many bytes of memory (`BUDGET_BASE` aside), and never more than
/// [`MAX_TILE_MEMORY`] in all. The number is high because `mvt-reader` reserves a geometry-long buffer for every ring
/// (every `ClosePath`): a polygon of 300 holes of nine points is 6.3 KB of input; the guard estimates 15 MB for it (2 400
/// bytes a byte), the measured peak is 183 KB (the estimate is a safe upper bound, not what is taken), and such polygons exist (a forest with clearings); small tiles of many lines reach about 3 300. The budget
/// of a tile is `min(MAX_TILE_MEMORY, BUDGET_BASE + BUDGET_PER_BYTE * bytes)`. The estimate is an upper bound as far
/// as measured (`tests/memory.rs` on tiles near the budget, and the fuzz target), not a proof.
pub const BUDGET_PER_BYTE: u64 = 4096;
/// What any tile may take, however short.
pub const BUDGET_BASE: u64 = 64 * 1024;
/// The most layers in one tile (real tiles have a dozen or two).
pub const MAX_LAYERS: usize = 256;
/// The most keys and values of all layers of one tile together.
pub const MAX_STRINGS: usize = 100_000;
/// A point of a geometry buffer.
const POINT_BYTES: u64 = 8;
/// What the decoder keeps besides the text for one layer, one key or value, one feature (the structs of `prost` and of
/// `mvt-reader`; an empty layer measured 175 bytes, an empty value 118), rounded up.
const LAYER_BYTES: u64 = 512;
const STRING_BYTES: u64 = 128;
const FEATURE_BYTES: u64 = 256;
/// What one point of a feature costs in the model of this crate (a dot or a label, without its text).
const POINT_OUT_BYTES: u64 = 64;
/// What one tag costs besides its two strings: the map entry (56 bytes), the room the map keeps free and the copy it
/// makes when it grows measured 115 to 130 bytes a tag; rounded up with a reserve.
const TAG_BYTES: u64 = 192;
/// What every number of a geometry or of the tags costs besides the buffers counted by `cost`: `prost` keeps them in a
/// `Vec<u32>` (up to 12 bytes a number while it grows) and the output model copies the coordinates (8 bytes).
const NUMBER_BYTES: u64 = 20;
/// The tags are numbers too (`prost` `Vec<u32>`).
const TAG_NUMBER_BYTES: u64 = 12;

/// What the scan found: the extent of each layer, in the order of the layers (`None` = the layer gives none: 4096).
pub(crate) struct Scanned {
    pub extents: Vec<Option<u32>>,
}

/// Geometry commands of one feature: the numbers, and the buffers `mvt-reader` makes besides the first one.
#[derive(Default)]
struct Geometry {
    numbers: u64,
    /// `ClosePath` commands: `mvt-reader` makes a buffer for each, in every kind of feature.
    closes: u64,
    /// `MoveTo` commands: a buffer for each, but only in a feature of lines (not in polygons and not for points).
    moves: u64,
    skip: u64,
    /// The numbers that follow a `MoveTo` or a `LineTo` (the parameters, two for a coordinate): the decoder turns EVERY
    /// pair into a coordinate, whatever the command, and in a feature of points every coordinate is a point.
    param_numbers: u64,
}

impl Geometry {
    fn feed(&mut self, value: u64) {
        // The decoder reads these as `uint32` and cuts a longer number to its low 32 bits: so does this.
        let value = low32(value);
        self.numbers += 1;
        if self.skip > 0 {
            self.skip -= 1;
            self.param_numbers += 1;
            return;
        }
        let count = value >> 3;
        match value & 7 {
            1 => {
                self.moves += 1;
                self.skip = count.saturating_mul(2);
            }
            2 => self.skip = count.saturating_mul(2),
            7 => self.closes += 1,
            _ => {}
        }
    }

    /// The coordinates of the feature, as the decoder makes them: one for every pair of parameters of a `MoveTo` or a
    /// `LineTo`. In a feature of points (type 1) each is a point of its own, drawn with its own copy of the text of
    /// the feature (`add_point` clones the name for every point); in the other types they are the points of a line
    /// or a ring, which the number cost already pays for.
    fn points(&self) -> u64 {
        self.param_numbers / 2
    }

    /// The bytes taken: the first buffer and one per `ClosePath` (and, in a feature of lines, per `MoveTo`), each as
    /// long as the geometry, and the numbers themselves (`NUMBER_BYTES` each). `lines` is whether the feature is a
    /// line (type 2); while its type is not known (it may come after the geometry) pass `false`: a lower bound.
    fn cost(&self, lines: bool) -> u64 {
        let buffers = 1 + self.closes + if lines { self.moves } else { 0 };
        buffers
            .saturating_mul(self.numbers)
            .saturating_mul(POINT_BYTES)
            .saturating_add(self.numbers.saturating_mul(NUMBER_BYTES))
    }
}

/// The sizes of the keys and of the values of a layer (a value: the bytes of its message, an upper bound of its text).
#[derive(Default)]
struct Strings {
    keys: Vec<u64>,
    values: Vec<u64>,
}

impl Strings {
    /// The sizes of one layer. `count` is the number of keys and values of the tile so far, `memory` what they cost so
    /// far: both are limited here, before a list of them is made.
    fn of(layer: &[u8], count: &mut usize, memory: &mut u64) -> Result<Self, MvtError> {
        let mut strings = Self::default();
        let mut cursor = Cursor(layer);
        while let Some((number, value)) = cursor.field()? {
            let (list, len) = match (number, value) {
                (3, Value::Bytes(key)) => (&mut strings.keys, key.len() as u64),
                (4, Value::Bytes(v)) => (&mut strings.values, v.len() as u64),
                _ => continue,
            };
            *count += 1;
            if *count > MAX_STRINGS {
                return Err(too_much("too many keys and values"));
            }
            *memory = memory.saturating_add(STRING_BYTES + len);
            list.push(len);
        }
        Ok(strings)
    }

    fn value_len(&self, value: u64) -> u64 {
        usize::try_from(value)
            .ok()
            .and_then(|i| self.values.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// What `parse_tags` copies for one pair: the key and the value, for this feature.
    fn pair(&self, key: u64, value: u64) -> u64 {
        let at = |list: &[u64], i: u64| usize::try_from(i).ok().and_then(|i| list.get(i)).copied();
        TAG_BYTES
            .saturating_add(at(&self.keys, key).unwrap_or(0))
            .saturating_add(at(&self.values, value).unwrap_or(0))
    }
}

/// The estimated memory of one feature: its geometry and the copies of its tags.
fn feature(bytes: &[u8], strings: &Strings, budget: u64) -> Result<u64, MvtError> {
    let mut geometry = Geometry::default();
    let mut tags = 0u64;
    let mut texts = 0u64; // the lengths of the values the feature names: the name is copied once for the feature
    let mut half: Option<u64> = None;
    let mut kind = 0u64; // the geometry type of the feature: 1 point, 2 line, 3 polygon
    let mut cursor = Cursor(bytes);
    let mut tag = |v: u64, tags: &mut u64| {
        // `uint32` here too; the pairs are (key, value)
        let v = low32(v);
        *tags = tags.saturating_add(TAG_NUMBER_BYTES);
        match half.take() {
            None => half = Some(v),
            Some(key) => {
                *tags = tags.saturating_add(strings.pair(key, v));
                texts = texts.saturating_add(strings.value_len(v));
            }
        }
    };
    while let Some((number, value)) = cursor.field()? {
        match (number, value) {
            (2, Value::Number(v)) => tag(v, &mut tags),
            (2, Value::Bytes(packed)) => {
                let mut numbers = Cursor(packed);
                while !numbers.0.is_empty() {
                    tag(numbers.varint()?, &mut tags);
                    if tags > budget {
                        return Err(too_much("a feature would need too much memory to decode"));
                    }
                }
            }
            (3, Value::Number(v)) => kind = low32(v), // `GeomType` is read as a 32-bit number too
            (4, Value::Number(v)) => geometry.feed(v),
            (4, Value::Bytes(packed)) => {
                let mut numbers = Cursor(packed);
                while !numbers.0.is_empty() {
                    geometry.feed(numbers.varint()?);
                    if geometry.cost(false) > budget {
                        return Err(too_much("a feature would need too much memory to decode"));
                    }
                }
            }
            _ => {}
        }
    }
    // a point is a struct of its own (the text is shared by the points of the feature, `Arc<str>`: counted once, as
    // `texts`, for the copy that `mvt.rs` makes of the name)
    let points = if kind == 1 {
        geometry.points().saturating_mul(POINT_OUT_BYTES)
    } else {
        0
    };
    Ok(geometry
        .cost(kind == 2)
        .saturating_add(tags)
        .saturating_add(points)
        .saturating_add(texts)
        .saturating_add(FEATURE_BYTES))
}

/// Look at a tile before it is decoded.
///
/// # Errors
/// [`MvtError`] when the tile is over a limit or its protobuf is cut off.
pub(crate) fn scan(bytes: &[u8]) -> Result<Scanned, MvtError> {
    if bytes.len() > MAX_TILE_BYTES {
        return Err(too_much("too many bytes"));
    }
    let budget = MAX_TILE_MEMORY
        .min(BUDGET_BASE.saturating_add(BUDGET_PER_BYTE.saturating_mul(bytes.len() as u64)));
    // the decoder holds a copy of the input (`Reader::new(bytes.to_vec())`) and what `prost` makes of it: its strings
    // and bytes are copies again (the lengths of the keys and values and the third copy of a layer name are counted
    // below; this is for the rest)
    let (mut features, mut memory, mut extents) = (0usize, 2 * bytes.len() as u64, Vec::new());
    let (mut layers, mut strings_count) = (0usize, 0usize);
    let mut tile = Cursor(bytes);
    while let Some((number, value)) = tile.field()? {
        let (3, Value::Bytes(layer)) = (number, value) else {
            continue;
        };
        layers += 1;
        if layers > MAX_LAYERS {
            return Err(too_much("too many layers"));
        }
        memory = memory.saturating_add(LAYER_BYTES);
        let strings = Strings::of(layer, &mut strings_count, &mut memory)?;
        if memory > budget {
            return Err(too_much("the tile would need too much memory to decode"));
        }
        let mut extent = None;
        let mut cursor = Cursor(layer);
        while let Some((number, value)) = cursor.field()? {
            match (number, value) {
                (2, Value::Bytes(f)) => {
                    features += 1;
                    if features > MAX_FEATURES {
                        return Err(too_much("too many features"));
                    }
                    memory = memory.saturating_add(feature(f, &strings, budget)?);
                    if memory > budget {
                        return Err(too_much("the tile would need too much memory to decode"));
                    }
                }
                (5, Value::Number(e)) => extent = u32::try_from(low32(e)).ok(), // `uint32`: the low 32 bits
                // the name of the layer: `prost` has it and `get_layer_metadata` makes a copy of it again (a third one,
                // the two copies of the whole input that are counted above are the first and the second)
                (1, Value::Bytes(name)) => memory = memory.saturating_add(name.len() as u64),
                _ => {}
            }
        }
        if memory > budget {
            return Err(too_much("the tile would need too much memory to decode"));
        }
        extents.push(extent);
    }
    Ok(Scanned { extents })
}
