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

/// `moves` times `MoveTo(1) 0 0` in a feature whose type is written with the 32nd bit set: `(1 << 32) | 2`. The decoder
/// reads the type as a 32-bit number, so for it this is a line, and a line costs a buffer of the whole geometry at
/// every `MoveTo` (3000 of them: 216 MB measured, 2.6 TB for a tile of 1 MB). The guard must read the type the same way.
pub fn line_type_with_high_bit(moves: u64) -> Vec<u8> {
    let mut geometry = Vec::new();
    for _ in 0..moves {
        geometry.extend_from_slice(&[9, 0, 0]);
    }
    tile(&[layer(
        "roads",
        4096,
        &[feature((1u64 << 32) | 2, &geometry)],
    )])
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

/// The mix of the review (probe H): one layer of point features, `with_tags` of them naming 57 tags each, and
/// `with_text` naming one 40 KB value: the estimate is filled by an exact part (the big string) and a part that is
/// hash-map heavy.
pub fn tag_mix(with_tags: usize, with_text: usize) -> Vec<u8> {
    let mut layer = Vec::new();
    bytes_field(1, b"roads", &mut layer);
    let mut pairs = Vec::new();
    for k in 0..57u64 {
        varint(k, &mut pairs);
        varint(0, &mut pairs);
    }
    for i in 0..with_tags + with_text {
        let mut f = Vec::new();
        if i < with_tags {
            bytes_field(2, &pairs, &mut f);
        } else {
            bytes_field(2, &[57, 1], &mut f);
        }
        number_field(3, 1, &mut f);
        bytes_field(4, &[9, 0, 0], &mut f);
        bytes_field(2, &f, &mut layer);
    }
    for k in 0..58 {
        bytes_field(3, format!("{k:03}").as_bytes(), &mut layer);
    }
    let mut v0 = Vec::new();
    number_field(4, 7, &mut v0);
    bytes_field(4, &v0, &mut layer);
    let mut v1 = Vec::new();
    bytes_field(1, &vec![b'x'; 40_000], &mut v1);
    bytes_field(4, &v1, &mut layer);
    number_field(15, 2, &mut layer);
    tile(&[layer])
}

/// A polygon with `holes` holes of nine points each (the case behind the factor of the budget: a forest with
/// clearings): the outer ring and the holes, each `MoveTo`, `LineTo` x 8, `ClosePath`.
pub fn polygon_with_holes(holes: usize) -> Vec<u8> {
    let mut g = vec![9u64, 0, 0, (3 << 3) | 2, 8000, 0, 0, 8000, 7999, 0, 15];
    for _ in 0..holes {
        g.extend_from_slice(&[
            9,
            20,
            20,
            (8 << 3) | 2,
            2,
            0,
            0,
            2,
            1,
            0,
            0,
            1,
            2,
            0,
            0,
            2,
            1,
            0,
            0,
            1,
            15,
        ]);
    }
    tile(&[layer("water", 4096, &[feature(3, &g)])])
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
