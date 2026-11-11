//! Fuzz target: `decode_tile` on any bytes. The rule of the crate (see `DESIGN.md`): no tile takes more memory than
//! its budget, `min(MAX_TILE_MEMORY, BUDGET_BASE + BUDGET_PER_BYTE * bytes)` (the copies of the input are in it). A counting
//! allocator measures the peak of every run; going over it is a finding (a panic here), as is any panic of the decoder.
//!
//! Pattern: none (a fuzz target). Run it only with a memory limit, see `DESIGN.md`.
#![no_main]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use fs_components_map::{BUDGET_BASE, BUDGET_PER_BYTE, MAX_TILE_MEMORY, decode_tile};
use libfuzzer_sys::fuzz_target;

struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call is passed on to the system allocator with the arguments it was given; only counters are added.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let now = LIVE.fetch_add(layout.size(), SeqCst) + layout.size();
            PEAK.fetch_max(now, SeqCst);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), SeqCst);
        unsafe { System.dealloc(p, layout) }
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, new: usize) -> *mut u8 {
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

fuzz_target!(|data: &[u8]| {
    let before = LIVE.load(SeqCst);
    PEAK.store(before, SeqCst);
    let _ = decode_tile(data);
    let peak = PEAK.load(SeqCst).saturating_sub(before);
    let budget = MAX_TILE_MEMORY.min(BUDGET_BASE + BUDGET_PER_BYTE * data.len() as u64);
    // the copies of the input are in the estimate: no extra
    let allowed = usize::try_from(budget).unwrap_or(usize::MAX);
    assert!(
        peak <= allowed,
        "{} bytes of input took {peak} bytes, the budget is {allowed}",
        data.len()
    );
});
