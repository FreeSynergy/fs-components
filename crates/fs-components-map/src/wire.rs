//! Reading the protobuf wire format without allocating, for the guard.
//!
//! Pattern: Iterator-like cursor (a position in a slice, fields one after another). Split from `guard.rs` for the file
//! limit. Nothing here makes memory: a field is a number or a slice of the input.

use crate::mvt::MvtError;

/// The low 32 bits of a number: the decoder reads every number of a tile (commands, coordinates, tags, the type of a
/// feature, the extent) as a 32-bit number and cuts a longer one, so the guard reads them the same way.
pub(crate) fn low32(value: u64) -> u64 {
    value & 0xFFFF_FFFF
}

/// A position in the bytes of a message.
pub(crate) struct Cursor<'a>(pub(crate) &'a [u8]);

impl<'a> Cursor<'a> {
    /// A varint: up to ten bytes.
    pub(crate) fn varint(&mut self) -> Result<u64, MvtError> {
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
    pub(crate) fn field(&mut self) -> Result<Option<(u64, Value<'a>)>, MvtError> {
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

/// What a field holds.
pub(crate) enum Value<'a> {
    Number(u64),
    Bytes(&'a [u8]),
    Skipped,
}

pub(crate) fn bad(why: &str) -> MvtError {
    MvtError(format!("not a vector tile: {why}"))
}

pub(crate) fn too_much(what: &str) -> MvtError {
    MvtError(format!("tile refused: {what}"))
}
