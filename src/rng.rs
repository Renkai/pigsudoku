//! Tiny PRNG shared by the practice games.
//!
//! The games only need a handful of numbers, so this replaces the `rand` crate
//! (which would also need extra setup on wasm).

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};

/// xorshift64* generator, seeded from the clock.
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Value in `lo..=hi` (both ends included).
    pub fn gen_range(&mut self, lo: u8, hi: u8) -> u8 {
        debug_assert!(lo <= hi, "empty range {lo}..={hi}");
        lo + (self.next() % (hi - lo + 1) as u64) as u8
    }
}
