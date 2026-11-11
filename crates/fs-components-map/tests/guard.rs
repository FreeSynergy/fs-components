//! What a tile may cost: limits checked before `mvt-reader` decodes, and one extent per layer.
//!
//! Pattern: none (tests). The tiles are written by hand in the protobuf form, so they can be hostile.

// Test encoders: the casts are of values that fit by construction.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::time::{Duration, Instant};

use fs_components_map::{MAX_FEATURES, MAX_TILE_BYTES, MAX_TILE_MEMORY, decode_tile};

fn varint(mut v: u64, out: &mut Vec<u8>) {
    while v >= 0x80 {
        out.push((v & 0x7f) as u8 | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn bytes_field(number: u64, body: &[u8], out: &mut Vec<u8>) {
    varint(number << 3 | 2, out);
    varint(body.len() as u64, out);
    out.extend_from_slice(body);
}

fn number_field(number: u64, value: u64, out: &mut Vec<u8>) {
    varint(number << 3, out);
    varint(value, out);
}

fn feature(kind: u64, geometry: &[u64]) -> Vec<u8> {
    let mut f = Vec::new();
    number_field(3, kind, &mut f);
    let mut packed = Vec::new();
    for v in geometry {
        varint(*v, &mut packed);
    }
    bytes_field(4, &packed, &mut f);
    f
}

fn layer(name: &str, extent: u64, features: &[Vec<u8>]) -> Vec<u8> {
    let mut l = Vec::new();
    bytes_field(1, name.as_bytes(), &mut l);
    for f in features {
        bytes_field(2, f, &mut l);
    }
    number_field(5, extent, &mut l);
    number_field(15, 2, &mut l);
    l
}

fn tile(layers: &[Vec<u8>]) -> Vec<u8> {
    let mut t = Vec::new();
    for l in layers {
        bytes_field(3, l, &mut t);
    }
    t
}

/// A line from (x0, y0) to (x1, y1) in a layer's own coordinates (zigzag numbers).
fn line(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<u8> {
    let zig = |v: i64| ((v << 1) ^ (v >> 63)) as u64;
    feature(2, &[9, zig(x0), zig(y0), 10, zig(x1 - x0), zig(y1 - y0)])
}

/// The real bomb of `mvt-reader` 2.5.0: a polygon with `rings` times `LineTo` + `ClosePath`. The decoder reserves a
/// buffer of the whole geometry at every `ClosePath` (see `guard.rs`); a `MoveTo` alone reserves nothing for a polygon.
fn close_path_bomb(rings: u64) -> Vec<u8> {
    let mut geometry = vec![9, 0, 0]; // MoveTo(1) 0,0
    for _ in 0..rings {
        geometry.extend_from_slice(&[10, 2, 2, 15]); // LineTo(1) 1,1 ; ClosePath
    }
    tile(&[layer("water", 4096, &[feature(3, &geometry)])])
}

#[test]
fn the_real_bomb_of_a_megabyte_is_refused_at_once_and_unread() {
    let started = Instant::now();
    // 250 000 ClosePaths over 1 000 003 numbers: mvt-reader would reserve 250 000 x 1 000 003 points of 8 bytes, 2 TB.
    let bomb = close_path_bomb(250_000);
    assert!(
        bomb.len() <= 1_100_000,
        "about a megabyte: {} bytes",
        bomb.len()
    );
    assert!(
        bomb.len() < MAX_TILE_BYTES,
        "it passes the byte limit, so only the estimate can stop it"
    );
    let error = decode_tile(&bomb).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "refused without the work"
    );
}

#[test]
fn a_bomb_of_move_to_alone_is_counted_too_to_be_safe() {
    let mut geometry = Vec::new();
    for _ in 0..250_000 {
        geometry.extend_from_slice(&[9, 0, 0]);
    }
    let error =
        decode_tile(&tile(&[layer("roads", 4096, &[feature(2, &geometry)])])).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
}

/// A feature of `rings` rings, which costs `(rings + 1) * (3 + 4 * rings)` points on its own.
fn middling(rings: u64) -> Vec<u8> {
    let mut geometry = vec![9, 0, 0];
    for _ in 0..rings {
        geometry.extend_from_slice(&[10, 2, 2, 15]);
    }
    feature(3, &geometry)
}

#[test]
fn the_cost_is_summed_over_the_tile_not_only_checked_per_feature() {
    // one feature of 100 rings: 101 * 403 = 40 703 points, far below the limit
    assert!(decode_tile(&tile(&[layer("water", 4096, &[middling(100)])])).is_ok());
    // 400 of them in one tile: each is fine alone, together 16.3 million points, over the limit of 8 million
    let features: Vec<Vec<u8>> = (0..400).map(|_| middling(100)).collect();
    const { assert!(40_703 * 8 * 400 > MAX_TILE_MEMORY) };
    let big = tile(&[layer("water", 4096, &features)]);
    assert!(big.len() < 200_000, "small in bytes: {}", big.len());
    let error = decode_tile(&big).expect_err("refused");
    assert!(
        error.0.contains("whole") || error.0.contains("refused"),
        "{error}"
    );
    // and the sum runs over layers: two layers of 200
    let half: Vec<Vec<u8>> = (0..200).map(|_| middling(100)).collect();
    let two = tile(&[layer("water", 4096, &half), layer("landuse", 4096, &half)]);
    assert!(decode_tile(&two).is_err(), "the layers are summed");
}

#[test]
fn a_tile_over_the_byte_limit_is_refused_before_anything_is_read() {
    let big = vec![0u8; MAX_TILE_BYTES + 1];
    let error = decode_tile(&big).expect_err("refused");
    assert!(error.0.contains("too many bytes"), "{error}");
}

#[test]
fn a_tile_over_the_feature_limit_is_refused() {
    let features: Vec<Vec<u8>> = (0..=MAX_FEATURES).map(|_| feature(1, &[9, 0, 0])).collect();
    let error = decode_tile(&tile(&[layer("places", 4096, &features)])).expect_err("refused");
    assert!(error.0.contains("too many features"), "{error}");
}

#[test]
fn a_cut_off_tile_is_an_error_not_a_panic() {
    let whole = tile(&[layer("roads", 4096, &[line(0, 0, 100, 100)])]);
    for cut in 1..whole.len() {
        let _ = decode_tile(&whole[..cut]); // any answer, no panic, no hang
    }
}

#[test]
fn every_layer_has_its_own_extent_and_lies_where_it_belongs() {
    // The same road across the middle, once in a layer of 4096 units and once in a layer of 512 units.
    let t = tile(&[
        layer("roads", 4096, &[line(0, 2048, 4096, 2048)]),
        layer("roads", 512, &[line(0, 256, 512, 256)]),
    ]);
    let g = decode_tile(&t).expect("decodes");
    assert!((g.extent - 4096.0).abs() < f32::EPSILON, "the widest layer");
    assert_eq!(g.lines.len(), 2);
    for l in &g.lines {
        assert_eq!(l.points.first(), Some(&[0.0, 2048.0]), "{:?}", l.points);
        assert_eq!(l.points.last(), Some(&[4096.0, 2048.0]), "{:?}", l.points);
    }
}

/// The same bomb, every command written with the 32nd bit set: protobuf `uint32` keeps the low 32 bits, so for the
/// decoder `(1 << 32) | 15` is the command 15 (a `ClosePath`), and the guard must read it the same way.
fn close_path_bomb_with_high_bits(rings: u64) -> Vec<u8> {
    let high = 1u64 << 32;
    let mut geometry = vec![high | 0x09, 0, 0];
    for _ in 0..rings {
        geometry.extend_from_slice(&[high | 0x0a, 2, 2, high | 0x0f]);
    }
    tile(&[layer("water", 4096, &[feature(3, &geometry)])])
}

#[test]
fn a_command_head_with_high_bits_is_read_as_the_decoder_reads_it() {
    let bomb = close_path_bomb_with_high_bits(250_000);
    assert!(bomb.len() < MAX_TILE_BYTES, "{} bytes", bomb.len());
    let error = decode_tile(&bomb).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
}

/// A layer with one big value string (`size` bytes) and `features` point features that each name it.
fn tag_bomb(size: usize, features: usize) -> Vec<u8> {
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

#[test]
fn the_copies_of_the_tags_are_counted_one_big_value_named_by_many_features_is_refused() {
    // mvt-reader 2.5.0 copies the value for every feature that names it: 400 x 1 MB = 400 MB from a 1 MB tile.
    let bomb = tag_bomb(1_000_000, 400);
    assert!(bomb.len() < 1_100_000, "a megabyte: {} bytes", bomb.len());
    let error = decode_tile(&bomb).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
    // the same value named by one feature is a normal tile
    assert!(decode_tile(&tag_bomb(1_000_000, 1)).is_ok());
    // and many small ones are fine
    assert!(decode_tile(&tag_bomb(20, 5_000)).is_ok());
}
