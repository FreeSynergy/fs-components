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
//! Tags: `parse_tags` (lib.rs:305) copies the key and the value of every tag pair for every feature, so a value
//! string of 1 MB named by 400 features is 400 MB (4 MiB of tile and 100 000 features would be 300 GB). A tile over a limit is refused, never decoded.
//!
//! Evidence (no memory is used by the tests): a tile of 1 MB made of `LineTo`+`ClosePath` repeated 250 000 times is
//! one feature with 250 000 `ClosePath` and 1 000 000 numbers, so `mvt-reader` reserves 250 000 buffers of 1 000 000
//! points of 8 bytes: 2 TB. The estimate here is `(C + 1) * G * 8` = 2e12 bytes; the limit is 64 MiB.

use crate::mvt::MvtError;

/// The most bytes a tile may have (a vector tile is some tens of kilobytes; busy ones a few hundred).
pub const MAX_TILE_BYTES: usize = 4 * 1024 * 1024;
/// The most features of all layers of one tile.
pub const MAX_FEATURES: usize = 100_000;
/// The most memory (bytes, estimated) `mvt-reader` may take for all features of one tile together: the geometry
/// buffers (8 bytes a point) and the copies of the tags (every value string once for every feature that names it).
pub const MAX_TILE_MEMORY: u64 = 64 * 1024 * 1024;
/// A point of a geometry buffer.
const POINT_BYTES: u64 = 8;
/// What one tag costs besides its two strings (the map entry, the string headers).
const TAG_BYTES: u64 = 64;

/// What the scan found: the extent of each layer, in the order of the layers (`None` = the layer gives none: 4096).
pub(crate) struct Scanned {
    pub extents: Vec<Option<u32>>,
}

struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    fn varint(&mut self) -> Result<u64, MvtError> {
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let (&byte, rest) = self.0.split_first().ok_or_else(|| bad("cut off number"))?;
            self.0 = rest;
            if shift < 64 {
                value |= u64::from(byte & 0x7f) << shift;
            }
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(bad("number too long"))
    }

    /// The next field: its number, and what it holds (a number, or the bytes of a nested message or packed list).
    fn field(&mut self) -> Result<Option<(u64, Value<'a>)>, MvtError> {
        if self.0.is_empty() {
            return Ok(None);
        }
        let key = self.varint()?;
        let value = match key & 7 {
            0 => Value::Number(self.varint()?),
            1 => {
                self.take(8)?;
                Value::Skipped
            }
            2 => {
                let len = usize::try_from(self.varint()?).map_err(|_| bad("length"))?;
                Value::Bytes(self.take(len)?)
            }
            5 => {
                self.take(4)?;
                Value::Skipped
            }
            _ => return Err(bad("unknown wire type")),
        };
        Ok(Some((key >> 3, value)))
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], MvtError> {
        if len > self.0.len() {
            return Err(bad("cut off field"));
        }
        let (head, rest) = self.0.split_at(len);
        self.0 = rest;
        Ok(head)
    }
}

enum Value<'a> {
    Number(u64),
    Bytes(&'a [u8]),
    Skipped,
}

fn bad(why: &str) -> MvtError {
    MvtError(format!("not a vector tile: {why}"))
}

fn too_much(what: &str) -> MvtError {
    MvtError(format!("tile refused: {what}"))
}

/// Geometry commands of one feature: the numbers, and the buffers `mvt-reader` makes besides the first one.
#[derive(Default)]
struct Geometry {
    numbers: u64,
    extra: u64,
    skip: u64,
}

impl Geometry {
    fn feed(&mut self, value: u64) {
        // The decoder reads these as `uint32` and cuts a longer number to its low 32 bits: so does this.
        let value = u64::from(value as u32);
        self.numbers += 1;
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        let count = value >> 3;
        match value & 7 {
            // a `MoveTo` makes a buffer for a line, a `ClosePath` for every ring: counted for both, to be safe
            1 => {
                self.extra += 1;
                self.skip = count.saturating_mul(2);
            }
            2 => self.skip = count.saturating_mul(2),
            7 => self.extra += 1,
            _ => {}
        }
    }

    /// The bytes reserved: the first buffer and one per command, each as long as the geometry.
    fn cost(&self) -> u64 {
        self.extra
            .saturating_add(1)
            .saturating_mul(self.numbers)
            .saturating_mul(POINT_BYTES)
    }
}

/// The sizes of the keys and of the values of a layer (a value: the bytes of its message, an upper bound of its text).
#[derive(Default)]
struct Strings {
    keys: Vec<u64>,
    values: Vec<u64>,
}

impl Strings {
    fn of(layer: &[u8]) -> Result<Self, MvtError> {
        let mut strings = Self::default();
        let mut cursor = Cursor(layer);
        while let Some((number, value)) = cursor.field()? {
            match (number, value) {
                (3, Value::Bytes(key)) => strings.keys.push(key.len() as u64),
                (4, Value::Bytes(v)) => strings.values.push(v.len() as u64),
                _ => {}
            }
        }
        Ok(strings)
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
fn feature(bytes: &[u8], strings: &Strings) -> Result<u64, MvtError> {
    let mut geometry = Geometry::default();
    let mut tags = 0u64;
    let mut half: Option<u64> = None;
    let mut cursor = Cursor(bytes);
    let mut tag = |v: u64, tags: &mut u64| {
        // `uint32` here too; the pairs are (key, value)
        let v = u64::from(v as u32);
        match half.take() {
            None => half = Some(v),
            Some(key) => *tags = tags.saturating_add(strings.pair(key, v)),
        }
    };
    while let Some((number, value)) = cursor.field()? {
        match (number, value) {
            (2, Value::Number(v)) => tag(v, &mut tags),
            (2, Value::Bytes(packed)) => {
                let mut numbers = Cursor(packed);
                while !numbers.0.is_empty() {
                    tag(numbers.varint()?, &mut tags);
                    if tags > MAX_TILE_MEMORY {
                        return Err(too_much("a feature would need too much memory to decode"));
                    }
                }
            }
            (4, Value::Number(v)) => geometry.feed(v),
            (4, Value::Bytes(packed)) => {
                let mut numbers = Cursor(packed);
                while !numbers.0.is_empty() {
                    geometry.feed(numbers.varint()?);
                    if geometry.cost() > MAX_TILE_MEMORY {
                        return Err(too_much("a feature would need too much memory to decode"));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(geometry.cost().saturating_add(tags))
}

/// Look at a tile before it is decoded.
///
/// # Errors
/// [`MvtError`] when the tile is over a limit or its protobuf is cut off.
pub(crate) fn scan(bytes: &[u8]) -> Result<Scanned, MvtError> {
    if bytes.len() > MAX_TILE_BYTES {
        return Err(too_much("too many bytes"));
    }
    let (mut features, mut memory, mut extents) = (0usize, 0u64, Vec::new());
    let mut tile = Cursor(bytes);
    while let Some((number, value)) = tile.field()? {
        let (3, Value::Bytes(layer)) = (number, value) else {
            continue;
        };
        let strings = Strings::of(layer)?;
        let mut extent = None;
        let mut cursor = Cursor(layer);
        while let Some((number, value)) = cursor.field()? {
            match (number, value) {
                (2, Value::Bytes(f)) => {
                    features += 1;
                    if features > MAX_FEATURES {
                        return Err(too_much("too many features"));
                    }
                    memory = memory.saturating_add(feature(f, &strings)?);
                    if memory > MAX_TILE_MEMORY {
                        return Err(too_much("the tile would need too much memory to decode"));
                    }
                }
                (5, Value::Number(e)) => extent = u32::try_from(e).ok(),
                _ => {}
            }
        }
        extents.push(extent);
    }
    Ok(Scanned { extents })
}
