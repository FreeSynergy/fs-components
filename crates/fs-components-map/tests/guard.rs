//! What a tile may cost: limits checked before `mvt-reader` decodes, and one extent per layer.
//!
//! Pattern: none (tests). The tiles are written by hand in the protobuf form, so they can be hostile.

// Test encoders: the casts are of values that fit by construction.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::time::{Duration, Instant};

use fs_components_map::{MAX_FEATURES, MAX_TILE_BYTES, decode_tile};

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

/// One feature with `rings` rings: a `MoveTo` and a point each, so `rings * numbers` grows with the square.
fn ring_bomb(rings: u64) -> Vec<u8> {
    let mut geometry = Vec::new();
    for _ in 0..rings {
        geometry.extend_from_slice(&[9, 0, 0]);
    }
    tile(&[layer("water", 4096, &[feature(3, &geometry)])])
}

#[test]
fn a_tile_whose_rings_would_make_the_decoder_reserve_the_square_is_refused_at_once_and_unread() {
    let started = Instant::now();
    // 20_000 rings, 60_000 numbers: 1.2 thousand million points reserved by mvt-reader 2.5.0 (9.6 GB), from 60 KB.
    let bomb = ring_bomb(20_000);
    assert!(bomb.len() < 100_000, "a small tile: {} bytes", bomb.len());
    let error = decode_tile(&bomb).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "refused without the work"
    );
}

#[test]
fn the_same_shape_in_a_size_that_is_fine_is_not_refused_by_the_guard() {
    let tile = ring_bomb(100);
    let result = decode_tile(&tile);
    assert!(
        result
            .as_ref()
            .is_err_and(|e| e.0.contains("refused"))
            .not(),
        "{result:?}"
    );
}

trait Not {
    fn not(self) -> bool;
}
impl Not for bool {
    fn not(self) -> bool {
        !self
    }
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
