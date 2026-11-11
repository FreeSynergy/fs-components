//! Tiles written by hand in the protobuf form, so they can be hostile: the bombs of the reviews, in any size.
//!
//! Pattern: Builder (test support). Used by `guard.rs`, `memory.rs` and the writer of the fuzz corpus.
#![allow(dead_code, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

pub fn varint(mut v: u64, out: &mut Vec<u8>) {
    while v >= 0x80 {
        out.push((v & 0x7f) as u8 | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

pub fn bytes_field(number: u64, body: &[u8], out: &mut Vec<u8>) {
    varint(number << 3 | 2, out);
    varint(body.len() as u64, out);
    out.extend_from_slice(body);
}

pub fn number_field(number: u64, value: u64, out: &mut Vec<u8>) {
    varint(number << 3, out);
    varint(value, out);
}

pub fn feature(kind: u64, geometry: &[u64]) -> Vec<u8> {
    let mut f = Vec::new();
    number_field(3, kind, &mut f);
    let mut packed = Vec::new();
    for v in geometry {
        varint(*v, &mut packed);
    }
    bytes_field(4, &packed, &mut f);
    f
}

pub fn layer(name: &str, extent: u64, features: &[Vec<u8>]) -> Vec<u8> {
    let mut l = Vec::new();
    bytes_field(1, name.as_bytes(), &mut l);
    for f in features {
        bytes_field(2, f, &mut l);
    }
    number_field(5, extent, &mut l);
    number_field(15, 2, &mut l);
    l
}

pub fn tile(layers: &[Vec<u8>]) -> Vec<u8> {
    let mut t = Vec::new();
    for l in layers {
        bytes_field(3, l, &mut t);
    }
    t
}

/// A line from (x0, y0) to (x1, y1) in a layer's own coordinates (zigzag numbers).
pub fn line(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<u8> {
    let zig = |v: i64| ((v << 1) ^ (v >> 63)) as u64;
    feature(2, &[9, zig(x0), zig(y0), 10, zig(x1 - x0), zig(y1 - y0)])
}

/// The real bomb of `mvt-reader` 2.5.0: a polygon with `rings` times `LineTo` + `ClosePath`. The decoder reserves a
/// buffer of the whole geometry at every `ClosePath` (see `guard.rs`); a `MoveTo` alone reserves nothing for a polygon.
pub fn close_path_bomb(rings: u64) -> Vec<u8> {
    let mut geometry = vec![9, 0, 0]; // MoveTo(1) 0,0
    for _ in 0..rings {
        geometry.extend_from_slice(&[10, 2, 2, 15]); // LineTo(1) 1,1 ; ClosePath
    }
    tile(&[layer("water", 4096, &[feature(3, &geometry)])])
}

/// A feature of `rings` rings, which costs `(rings + 1) * (3 + 4 * rings)` points on its own.
pub fn middling(rings: u64) -> Vec<u8> {
    let mut geometry = vec![9, 0, 0];
    for _ in 0..rings {
        geometry.extend_from_slice(&[10, 2, 2, 15]);
    }
    feature(3, &geometry)
}

/// The same bomb, every command written with the 32nd bit set: protobuf `uint32` keeps the low 32 bits, so for the
/// decoder `(1 << 32) | 15` is the command 15 (a `ClosePath`), and the guard must read it the same way.
pub fn close_path_bomb_with_high_bits(rings: u64) -> Vec<u8> {
    let high = 1u64 << 32;
    let mut geometry = vec![high | 0x09, 0, 0];
    for _ in 0..rings {
        geometry.extend_from_slice(&[high | 0x0a, 2, 2, high | 0x0f]);
    }
    tile(&[layer("water", 4096, &[feature(3, &geometry)])])
}

/// A layer with one big value string (`size` bytes) and `features` point features that each name it.
pub fn tag_bomb(size: usize, features: usize) -> Vec<u8> {
    let mut value = Vec::new();
    bytes_field(1, &vec![b'x'; size], &mut value); // Value.string_value
    let mut layer = Vec::new();
    bytes_field(1, b"places", &mut layer);
    for _ in 0..features {
        let mut f = Vec::new();
        let mut pair = Vec::new();
        varint(0, &mut pair);
        varint(0, &mut pair);
        bytes_field(2, &pair, &mut f); // tags: key 0, value 0
        number_field(3, 1, &mut f); // type: point
        let mut g = Vec::new();
        for v in [9u64, 0, 0] {
            varint(v, &mut g);
        }
        bytes_field(4, &g, &mut f);
        bytes_field(2, &f, &mut layer);
    }
    bytes_field(3, b"name", &mut layer); // keys
    bytes_field(4, &value, &mut layer); // values
    number_field(5, 4096, &mut layer);
    tile(&[layer])
}

/// A layer named `places` with one feature of type point that has `points` points (one `MoveTo` with that count) and
/// the tag `name` with a value of `name_size` bytes: `add_point` copied that name for every point.
pub fn places_layer(points: u64, name_size: usize) -> Vec<u8> {
    let mut value = Vec::new();
    bytes_field(1, &vec![b'x'; name_size], &mut value);
    let mut f = Vec::new();
    let mut pair = Vec::new();
    varint(0, &mut pair);
    varint(0, &mut pair);
    bytes_field(2, &pair, &mut f);
    number_field(3, 1, &mut f);
    let mut g = Vec::new();
    varint(points << 3 | 1, &mut g);
    for _ in 0..points * 2 {
        varint(0, &mut g);
    }
    bytes_field(4, &g, &mut f);
    let mut layer = Vec::new();
    bytes_field(1, b"places", &mut layer);
    bytes_field(2, &f, &mut layer);
    bytes_field(3, b"name", &mut layer);
    bytes_field(4, &value, &mut layer);
    number_field(5, 4096, &mut layer);
    layer
}

/// The same as a tile of its own.
pub fn multipoint_bomb(points: u64, name_size: usize) -> Vec<u8> {
    tile(&[places_layer(points, name_size)])
}

/// `n` layers with nothing in them (`1a 00` repeated): the decoder made 175 bytes for each.
pub fn empty_layers(n: usize) -> Vec<u8> {
    let mut t = Vec::new();
    for _ in 0..n {
        bytes_field(3, &[], &mut t);
    }
    t
}

/// One layer with `n` empty values (`22 00` repeated): 118 bytes each in the decoder.
pub fn empty_values(n: usize) -> Vec<u8> {
    let mut layer = Vec::new();
    bytes_field(1, b"places", &mut layer);
    for _ in 0..n {
        bytes_field(4, &[], &mut layer);
    }
    tile(&[layer])
}

/// One layer with `n` empty keys.
pub fn empty_keys(n: usize) -> Vec<u8> {
    let mut layer = Vec::new();
    bytes_field(1, b"places", &mut layer);
    for _ in 0..n {
        bytes_field(3, &[], &mut layer);
    }
    tile(&[layer])
}

/// A tile as a map has it: a few layers, a few hundred features with short names.
pub fn ordinary() -> Vec<u8> {
    let road = |i: i64| line(i, 0, i + 500, 900 + i);
    let roads: Vec<Vec<u8>> = (0..300).map(road).collect();
    let water = feature(3, &[9, 200, 200, 26, 800, 0, 0, 800, 799, 0, 15]);
    tile(&[
        layer("roads", 4096, &roads),
        layer("water", 4096, &[water]),
        places_layer(3, 5),
    ])
}
