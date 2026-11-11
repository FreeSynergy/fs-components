//! What a tile may cost: limits checked before `mvt-reader` decodes, and one extent per layer.
//!
//! Pattern: none (tests). The tiles are the bombs of `bombs/mod.rs`; each must be refused by the guard before any
//! decoding, so these tests take no memory themselves (run them under `limited.sh -m 1G`, see DESIGN.md).

mod bombs;

use std::time::{Duration, Instant};

use bombs::*;
use fs_components_map::{
    MAX_FEATURES, MAX_LAYERS, MAX_STRINGS, MAX_TILE_BYTES, MAX_TILE_MEMORY, decode_tile,
};

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

#[test]
fn a_feature_type_with_high_bits_is_read_as_the_decoder_reads_it() {
    // for the decoder `(1 << 32) | 2` is a line: every `MoveTo` makes a buffer of the whole geometry. 3000 of them
    // are 9 KB of input; the type 2 itself is refused, so is this (it took 216 MB when the guard did not cut the type)
    let plain = {
        let mut g = Vec::new();
        for _ in 0..3000 {
            g.extend_from_slice(&[9, 0, 0]);
        }
        tile(&[layer("roads", 4096, &[feature(2, &g)])])
    };
    assert!(decode_tile(&plain).is_err(), "type 2");
    let error = decode_tile(&line_type_with_high_bit(3000)).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
}

#[test]
fn an_extent_with_high_bits_is_read_as_the_decoder_reads_it() {
    // `(1 << 32) | 512` is 512 for the decoder: the tile is drawn at that extent, not at the default
    let tile = tile(&[layer(
        "roads",
        (1u64 << 32) | 512,
        &[line(0, 256, 512, 256)],
    )]);
    let drawn = decode_tile(&tile).expect("decodes");
    assert!(
        (drawn.extent - 512.0).abs() < f32::EPSILON,
        "extent {}",
        drawn.extent
    );
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

#[test]
fn a_command_head_with_high_bits_is_read_as_the_decoder_reads_it() {
    let bomb = close_path_bomb_with_high_bits(250_000);
    assert!(bomb.len() < MAX_TILE_BYTES, "{} bytes", bomb.len());
    let error = decode_tile(&bomb).expect_err("refused");
    assert!(error.0.contains("refused"), "{error}");
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

#[test]
fn a_multipoint_with_a_big_name_is_decoded_with_one_shared_name_and_a_huge_one_is_refused() {
    // 100 points and a 1 MiB name was 108 MB, 2000 points 2.05 GB, 20 000 points 21 GB (the name copied for every
    // point). The name is shared now, and a point is counted as a struct of its own: all are judged by the budget.
    for points in [100, 2000, 20_000] {
        let bomb = multipoint_bomb(points, 1 << 20);
        assert!(bomb.len() < 1_200_000, "{} bytes", bomb.len());
        let verdict = decode_tile(&bomb);
        // the name once (copied by the decoder, by the model) is 3 MiB: inside the budget of a tile of this size; the
        // points are 64 bytes each: 20 000 of them 1.3 MB
        assert!(verdict.is_ok(), "{points}: {verdict:?}");
        assert_eq!(
            verdict.unwrap().labels.len() as u64,
            points,
            "one label a point, one shared name"
        );
    }
    // the budget stops what is over it: many points, each of its own struct, from a tile too short to pay for them
    let many = multipoint_bomb(1_000_000, 4);
    let error = decode_tile(&many).expect_err("refused");
    assert!(
        error.0.contains("refused") || error.0.contains("not a vector"),
        "{error}"
    );
}

#[test]
fn empty_layers_keys_and_values_are_limited_and_cost_something() {
    let layers = empty_layers(MAX_LAYERS + 1);
    assert!(
        decode_tile(&layers)
            .expect_err("refused")
            .0
            .contains("too many layers")
    );
    let values = empty_values(MAX_STRINGS + 1);
    assert!(
        decode_tile(&values)
            .expect_err("refused")
            .0
            .contains("too many keys")
    );
    let keys = empty_keys(MAX_STRINGS + 1);
    assert!(
        decode_tile(&keys)
            .expect_err("refused")
            .0
            .contains("too many keys")
    );
    // the 1.1 MB tiles of the review: 550 000 empty layers, 550 000 empty values
    assert!(decode_tile(&empty_layers(550_000)).is_err());
    assert!(decode_tile(&empty_values(550_000)).is_err());
    // under the limits and paid for by its bytes (64 bytes of memory for each of its 2): accepted
    assert!(decode_tile(&empty_values(MAX_STRINGS - 10)).is_ok());
}

/// Writes the starting corpus of the fuzz target (`fuzz/corpus/mvt_decode`): the bombs of the reviews in a size that
/// still is far over the budget, and small tiles that decode. Run with `-- --ignored write_the_fuzz_corpus`.
#[test]
#[ignore = "writes files: run on purpose"]
fn write_the_fuzz_corpus() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/mvt_decode");
    std::fs::create_dir_all(&dir).unwrap();
    let mut move_to = Vec::new();
    for _ in 0..3000 {
        move_to.extend_from_slice(&[9, 0, 0]);
    }
    let files: Vec<(&str, Vec<u8>)> = vec![
        ("close-path-bomb", close_path_bomb(3000)),
        (
            "move-to-bomb",
            tile(&[layer("roads", 4096, &[feature(2, &move_to)])]),
        ),
        ("high-bits-head", close_path_bomb_with_high_bits(3000)),
        ("high-bits-type", line_type_with_high_bit(3000)),
        ("value-bomb", tag_bomb(100_000, 700)),
        ("multipoint-name-bomb", multipoint_bomb(1000, 50_000)),
        ("multipoint-many-points", multipoint_bomb(20_000, 4)),
        ("empty-layers", empty_layers(2000)),
        ("empty-values", empty_values(100_050)),
        ("ordinary", ordinary()),
        (
            "two-extents",
            tile(&[
                layer("roads", 4096, &[line(0, 2048, 4096, 2048)]),
                layer("roads", 512, &[line(0, 256, 512, 256)]),
            ]),
        ),
        ("one-point", multipoint_bomb(1, 4)),
        ("empty", Vec::new()),
    ];
    for (name, bytes) in files {
        std::fs::write(dir.join(name), bytes).unwrap();
    }
}
