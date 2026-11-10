//! What a tile may cost, looked at before `mvt-reader` decodes it.
//!
//! Pattern: Guard (a check before the work). `mvt-reader` 2.5.0 reserves, for every ring of a feature, room for the
//! whole geometry of that feature, so a feature of `G` numbers with `R` rings asks for `R * G` points (quadratic), and
//! running out of memory is no panic that `catch_unwind` catches. The tile is therefore scanned first, in its raw
//! protobuf form, with no allocation: its size, the number of its features, and for each feature `R * G`, and the
//! extent of each layer is read here too. A tile over a limit is refused, never decoded.

use crate::mvt::MvtError;

/// The most bytes a tile may have (a vector tile is some tens of kilobytes; busy ones a few hundred).
pub const MAX_TILE_BYTES: usize = 4 * 1024 * 1024;
/// The most features of all layers of one tile.
pub const MAX_FEATURES: usize = 100_000;
/// The most points `mvt-reader` may reserve for one feature (`rings * numbers of its geometry`, 8 bytes each).
pub const MAX_RING_CELLS: u64 = 4_000_000;

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

/// Geometry commands of one feature: the numbers and the rings (the `MoveTo`s) so far.
#[derive(Default)]
struct Geometry {
    numbers: u64,
    rings: u64,
    skip: u64,
}

impl Geometry {
    fn feed(&mut self, value: u64) {
        self.numbers += 1;
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        let count = value >> 3;
        match value & 7 {
            1 => {
                self.rings = self.rings.saturating_add(count);
                self.skip = count.saturating_mul(2);
            }
            2 => self.skip = count.saturating_mul(2),
            _ => {}
        }
    }

    fn cost(&self) -> u64 {
        self.rings.saturating_mul(self.numbers)
    }
}

fn feature(bytes: &[u8]) -> Result<(), MvtError> {
    let mut geometry = Geometry::default();
    let mut cursor = Cursor(bytes);
    while let Some((number, value)) = cursor.field()? {
        if number != 4 {
            continue;
        }
        match value {
            Value::Number(v) => geometry.feed(v),
            Value::Bytes(packed) => {
                let mut numbers = Cursor(packed);
                while !numbers.0.is_empty() {
                    geometry.feed(numbers.varint()?);
                }
            }
            Value::Skipped => {}
        }
        if geometry.cost() > MAX_RING_CELLS {
            return Err(too_much("a feature would need too much memory to decode"));
        }
    }
    Ok(())
}

/// Look at a tile before it is decoded.
///
/// # Errors
/// [`MvtError`] when the tile is over a limit or its protobuf is cut off.
pub(crate) fn scan(bytes: &[u8]) -> Result<Scanned, MvtError> {
    if bytes.len() > MAX_TILE_BYTES {
        return Err(too_much("too many bytes"));
    }
    let (mut features, mut extents) = (0usize, Vec::new());
    let mut tile = Cursor(bytes);
    while let Some((number, value)) = tile.field()? {
        let (3, Value::Bytes(layer)) = (number, value) else {
            continue;
        };
        let mut extent = None;
        let mut cursor = Cursor(layer);
        while let Some((number, value)) = cursor.field()? {
            match (number, value) {
                (2, Value::Bytes(f)) => {
                    features += 1;
                    if features > MAX_FEATURES {
                        return Err(too_much("too many features"));
                    }
                    feature(f)?;
                }
                (5, Value::Number(e)) => extent = u32::try_from(e).ok(),
                _ => {}
            }
        }
        extents.push(extent);
    }
    Ok(Scanned { extents })
}
