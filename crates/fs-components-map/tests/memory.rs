//! The rule, measured: no byte of input makes more than `BUDGET_PER_BYTE` bytes of memory (and never more than
//! `MAX_TILE_MEMORY`), whatever the tile is. A counting allocator watches `decode_tile` on the bombs of the reviews
//! (in the size of the reviews: about a megabyte each) and on ordinary tiles.
//!
//! Pattern: none (a test with a counting allocator). It is one test, so no other thread allocates while it measures.
//! Run it under `limited.sh -m 1G`: nothing here needs more than a few megabytes, because the bombs are refused before
//! they are decoded; if a guard fails, the limit of the process is what stops the test, not the machine.

mod bombs;
mod counting;

use bombs::*;
use counting::{Counting, allowed, peak_of};
use fs_components_map::decode_tile;

#[global_allocator]
static COUNTING: Counting = Counting;

// One test on purpose (the counting allocator sees every thread): it runs the cases one after the other.
#[allow(clippy::too_many_lines)]
#[test]
fn no_tile_takes_more_memory_than_its_budget_and_the_bombs_take_next_to_none() {
    let bombs: Vec<(&str, Vec<u8>)> = vec![
        ("ClosePath x 250 000", close_path_bomb(250_000)),
        ("MoveTo x 250 000", {
            let mut g = Vec::new();
            for _ in 0..250_000 {
                g.extend_from_slice(&[9, 0, 0]);
            }
            tile(&[layer("roads", 4096, &[feature(2, &g)])])
        }),
        ("head (1<<32)|9", close_path_bomb_with_high_bits(250_000)),
        ("type (1<<32)|2, 3000 MoveTo", line_type_with_high_bit(3000)),
        ("1 MB value, 400 features", tag_bomb(1_000_000, 400)),
        ("2000 points, 1 MiB name", multipoint_bomb(2000, 1 << 20)),
        (
            "20 000 points, 1 MiB name",
            multipoint_bomb(20_000, 1 << 20),
        ),
        ("550 000 empty layers", empty_layers(550_000)),
        ("550 000 empty values", empty_values(550_000)),
        ("550 000 empty keys", empty_keys(550_000)),
    ];
    for (name, tile) in &bombs {
        let (peak, ok) = peak_of(tile);
        assert!(
            peak <= allowed(tile.len()),
            "{name}: {} bytes of input took {peak} bytes (ok: {ok})",
            tile.len()
        );
        println!(
            "{name}: {} bytes in, peak {peak} bytes, decoded: {ok}",
            tile.len()
        );
    }
    // the starting corpus of the fuzz target, run the way the target runs it (the same rule, the same allocator)
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/mvt_decode");
    let mut replayed = 0;
    for entry in std::fs::read_dir(&corpus).expect("the corpus is in the repository") {
        let path = entry.unwrap().path();
        let tile = std::fs::read(&path).unwrap();
        let (peak, _) = peak_of(&tile);
        assert!(
            peak <= allowed(tile.len()),
            "{}: {} bytes in, peak {peak}",
            path.display(),
            tile.len()
        );
        replayed += 1;
    }
    // the probe of the review (w54), alone and by name: 1.1 M points made by `LineTo` in a feature of points, each with
    // the name of the feature as its label. Before the guard counted them, 109 465 031 bytes were taken (budget 67 108 864).
    let probe = std::fs::read(corpus.join("labels-lineto")).expect("the probe is in the corpus");
    let (peak, ok) = peak_of(&probe);
    println!(
        "labels-lineto: {} bytes in, peak {peak} bytes, decoded: {ok}",
        probe.len()
    );
    assert!(
        !ok && peak <= allowed(probe.len()),
        "peak {peak} for {} bytes",
        probe.len()
    );
    assert!(replayed >= 10, "{replayed} files in the corpus");
    // tiles that are ACCEPTED and near the budget: the estimate must be an upper bound for them, not a guess. The mix
    // of the review (probe H) in sizes around the budget: every one that the guard lets through must stay inside it.
    let mut accepted_near = 0;
    for (with_tags, with_text) in [
        (3000, 100),
        (3500, 150),
        (4000, 150),
        (4400, 100),
        (4800, 60),
        (5200, 20),
        (7000, 430),
        (8000, 380),
    ] {
        let tile = tag_mix(with_tags, with_text);
        let (peak, ok) = peak_of(&tile);
        assert!(
            peak <= allowed(tile.len()),
            "H mix {with_tags}/{with_text}: {} bytes in, peak {peak}, budget {} (decoded: {ok})",
            tile.len(),
            allowed(tile.len())
        );
        // near: more than 40% of the budget really taken (the estimate is about twice what is taken for this mix)
        if ok && peak * 10 > allowed(tile.len()) * 4 {
            accepted_near += 1;
        }
        println!(
            "H mix {with_tags}/{with_text}: {} bytes in, peak {peak} bytes, decoded: {ok}",
            tile.len()
        );
    }
    assert!(
        accepted_near >= 1,
        "no mix was accepted near the budget: the test would say nothing"
    );
    // a real polygon with 300 holes of nine points has to decode (it is 6 KB; the guard estimates 15 MB, the measured peak is 183 KB)
    let polygon = polygon_with_holes(300);
    let (peak, ok) = peak_of(&polygon);
    assert!(
        ok,
        "a polygon with 300 holes is a tile: {:?}",
        decode_tile(&polygon).err()
    );
    assert!(
        peak <= allowed(polygon.len()),
        "polygon: {} bytes in, peak {peak}",
        polygon.len()
    );
    let areas = decode_tile(&polygon).unwrap().areas;
    println!(
        "polygon 300 holes: {} bytes in, peak {peak} bytes, {} area(s), {} ring(s)",
        polygon.len(),
        areas.len(),
        areas.iter().map(|a| a.rings.len()).sum::<usize>()
    );
    for (name, tile) in [
        ("an ordinary tile", ordinary()),
        ("100 points, 1 MiB name", multipoint_bomb(100, 1 << 20)),
    ] {
        let (peak, ok) = peak_of(&tile);
        assert!(
            ok,
            "{name} is a tile that decodes: {:?}",
            decode_tile(&tile).err()
        );
        assert!(
            peak <= allowed(tile.len()),
            "{name}: {} bytes in, peak {peak}",
            tile.len()
        );
        println!("{name}: {} bytes in, peak {peak} bytes", tile.len());
    }
}
