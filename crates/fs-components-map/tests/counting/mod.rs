//! A counting allocator for the tests that measure what `decode_tile` takes. The test file that uses it declares
//! `#[global_allocator] static COUNTING: counting::Counting = counting::Counting;` (it must stand at the root of the
//! test crate) and runs its cases one after the other (the allocator sees every thread).
//!
//! Pattern: none (test support).
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use fs_components_map::{BUDGET_BASE, BUDGET_PER_BYTE, MAX_TILE_MEMORY, decode_tile};

pub struct Counting;
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

/// The most memory `decode_tile` took above what was held before, for this tile.
pub fn peak_of(tile: &[u8]) -> (usize, bool) {
    let before = LIVE.load(SeqCst);
    PEAK.store(before, SeqCst);
    let ok = decode_tile(tile).is_ok();
    (PEAK.load(SeqCst).saturating_sub(before), ok)
}

pub fn allowed(len: usize) -> usize {
    // the budget as the guard counts it: the copy of the input is in it (no extra for it here)
    let budget = MAX_TILE_MEMORY.min(BUDGET_BASE + BUDGET_PER_BYTE * len as u64);
    usize::try_from(budget).unwrap()
}
