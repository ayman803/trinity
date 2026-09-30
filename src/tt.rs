//! Transposition table: a big hash table remembering results of earlier
//! searches. It is shared by all search threads without locks: each entry
//! stores `key ^ data` next to `data`, so a torn (half-written) entry
//! simply fails the key check instead of returning garbage.

use crate::types::Move;
use std::sync::atomic::{AtomicU64, Ordering};

pub const BOUND_UPPER: u8 = 1; // score is at most this (fail low)
pub const BOUND_LOWER: u8 = 2; // score is at least this (fail high)
pub const BOUND_EXACT: u8 = 3;

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub mv: Move,
    pub score: i16,
    pub eval: i16,
    pub depth: u8,
    pub bound: u8,
    pub age: u8,
}

impl Entry {
    fn pack(&self) -> u64 {
        u64::from(self.mv.0)
            | u64::from(self.score as u16) << 16
            | u64::from(self.eval as u16) << 32
            | u64::from(self.depth) << 48
            | u64::from(self.bound & 3) << 56
            | u64::from(self.age & 63) << 58
    }

    fn unpack(d: u64) -> Entry {
        Entry {
            mv: Move(d as u16),
            score: (d >> 16) as u16 as i16,
            eval: (d >> 32) as u16 as i16,
            depth: (d >> 48) as u8,
            bound: ((d >> 56) & 3) as u8,
            age: (d >> 58) as u8,
        }
    }
}

#[derive(Default)]
struct Slot {
    key: AtomicU64,
    data: AtomicU64,
}

pub struct TranspositionTable {
    slots: Vec<Slot>,
    age: AtomicU64,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let n = (mb.max(1) * 1024 * 1024 / std::mem::size_of::<Slot>()).max(1024);
        let mut slots = Vec::with_capacity(n);
        slots.resize_with(n, Slot::default);
        TranspositionTable { slots, age: AtomicU64::new(0) }
    }

    pub fn clear(&self) {
        for s in &self.slots {
            s.key.store(0, Ordering::Relaxed);
            s.data.store(0, Ordering::Relaxed);
        }
        self.age.store(0, Ordering::Relaxed);
    }

    /// Called at the start of every search so old entries can be replaced.
    pub fn new_search(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    fn age(&self) -> u8 {
        (self.age.load(Ordering::Relaxed) & 63) as u8
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        // Map the key onto [0, len) with a multiply instead of a modulo.
        ((u128::from(key) * self.slots.len() as u128) >> 64) as usize
    }

    #[inline(always)]
    pub fn prefetch(&self, key: u64) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};
            let ptr = self.slots.as_ptr().add(self.index(key)) as *const i8;
            _mm_prefetch::<_MM_HINT_T0>(ptr);
        }
        #[cfg(not(target_arch = "x86_64"))]
        let _ = key;
    }

    pub fn probe(&self, key: u64) -> Option<Entry> {
        let slot = &self.slots[self.index(key)];
        let data = slot.data.load(Ordering::Relaxed);
        if slot.key.load(Ordering::Relaxed) ^ data == key && data != 0 { Some(Entry::unpack(data)) } else { None }
    }

    pub fn store(&self, key: u64, mv: Move, score: i32, eval: i32, depth: i32, bound: u8) {
        let slot = &self.slots[self.index(key)];
        let old_data = slot.data.load(Ordering::Relaxed);
        let same_key = slot.key.load(Ordering::Relaxed) ^ old_data == key;
        let old = Entry::unpack(old_data);
        let age = self.age();

        // Keep deeper results from the current search unless this one is exact.
        let replace = !same_key || old.age != age || bound == BOUND_EXACT || depth + 4 > i32::from(old.depth);
        if !replace {
            return;
        }
        // Don't lose a known best move when storing a result without one.
        let mv = if mv.is_none() && same_key { old.mv } else { mv };
        let e = Entry { mv, score: score as i16, eval: eval as i16, depth: depth.clamp(0, 255) as u8, bound, age };
        let data = e.pack();
        slot.key.store(key ^ data, Ordering::Relaxed);
        slot.data.store(data, Ordering::Relaxed);
    }

    /// Permille of the first 1000 slots used in the current search (UCI `hashfull`).
    pub fn hashfull(&self) -> usize {
        let age = self.age();
        self.slots
            .iter()
            .take(1000)
            .filter(|s| {
                let d = s.data.load(Ordering::Relaxed);
                d != 0 && Entry::unpack(d).age == age
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_probe() {
        let tt = TranspositionTable::new(1);
        let m = Move::new(12, 28, 1);
        tt.store(0xDEAD_BEEF_1234_5678, m, -345, 20, 7, BOUND_LOWER);
        let e = tt.probe(0xDEAD_BEEF_1234_5678).unwrap();
        assert_eq!((e.mv, e.score, e.eval, e.depth, e.bound), (m, -345, 20, 7, BOUND_LOWER));
        assert!(tt.probe(0x1111).is_none());
    }
}
