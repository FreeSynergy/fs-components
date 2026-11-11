//! The rule, measured: no byte of input makes more than `BUDGET_PER_BYTE` bytes of memory (and never more than
//! `MAX_TILE_MEMORY`), whatever the tile is. A counting allocator watches `decode_tile` on the bombs of the reviews
//! (in the size of the reviews: about a megabyte each) and on ordinary tiles.
//!
//! Pattern: none (a test with a counting allocator). It is one test, so no other thread allocates while it measures.
//! Run it under `limited.sh -m 1G`: nothing here needs more than a few megabytes, because the bombs are refused before
//! they are decoded; if a guard fails, the limit of the process is what stops the test, not the machine.

mod bombs;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use bombs::*;
use fs_components_map::{BUDGET_BASE, BUDGET_PER_BYTE, MAX_TILE_MEMORY, decode_tile};

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the same call, on the system allocator, with the layout the caller gave.
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let now = LIVE.fetch_add(layout.size(), SeqCst) + layout.size();
            PEAK.fetch_max(now, SeqCst);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), SeqCst);
        // SAFETY: `p` came from `alloc` with this layout.
        unsafe { System.dealloc(p, layout) }
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        // SAFETY: `p` came from this allocator with this layout.
        let q = unsafe { System.realloc(p, layout, new) };
        if !q.is_null() {
            LIVE.fetch_sub(layout.size(), SeqCst);
            let now = LIVE.fetch_add(new, SeqCst) + new;
            PEAK.fetch_max(now, SeqCst);
        }
        q
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// The most memory `decode_tile` took above what was held before, for this tile.
fn peak_of(tile: &[u8]) -> (usize, bool) {
    let before = LIVE.load(SeqCst);
    PEAK.store(before, SeqCst);
    let ok = decode_tile(tile).is_ok();
    (PEAK.load(SeqCst).saturating_sub(before), ok)
}

fn allowed(len: usize) -> usize {
    let budget = MAX_TILE_MEMORY.min(BUDGET_BASE + BUDGET_PER_BYTE * len as u64);
    // the decoder also holds a copy of the input (`Reader::new(bytes.to_vec())`)
    usize::try_from(budget).unwrap() + len
}

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
    assert!(replayed >= 10, "{replayed} files in the corpus");
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
