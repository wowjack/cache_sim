use crate::memory_system::{CacheLine, CacheStatus};
use rand::RngExt;

/// Tracks cache line access order using a counter.
/// Shared by LRU and LRU_PREFER_CLEAN.
struct LruState {
    counter: u64,
    /// last access time per cache line.
    timestamps: Vec<Vec<u64>>,
}

impl LruState {
    fn new(num_sets: u64, associativity: u64) -> Self {
        Self {
            counter: 0,
            timestamps: vec![vec![0u64; associativity as usize]; num_sets as usize],
        }
    }

    fn record_access(&mut self, set: &[CacheLine], set_idx: u64, tag: u64) {
        self.counter += 1;
        if let Some(i) = set.iter().position(|line| line.tag == tag) {
            self.timestamps[set_idx as usize][i] = self.counter;
        }
    }

    fn lru_index(&self, set_idx: u64) -> usize {
        self.timestamps[set_idx as usize]
            .iter()
            .enumerate()
            .min_by_key(|(_, ts)| *ts)
            .unwrap()
            .0
    }
}

pub enum ReplacementPolicy {
    Lru(LruState),
    Rand { associativity: u64 },
    LruPreferClean(LruState),
}

impl ReplacementPolicy {
    pub fn lru(num_sets: u64, associativity: u64) -> Self {
        Self::Lru(LruState::new(num_sets, associativity))
    }

    pub fn rand(associativity: u64) -> Self {
        Self::Rand { associativity }
    }

    pub fn lru_prefer_clean(num_sets: u64, associativity: u64) -> Self {
        Self::LruPreferClean(LruState::new(num_sets, associativity))
    }

    /// Notify the policy that a line with `tag` was just accessed in `set_idx`.
    pub fn notify_access(&mut self, set: &[CacheLine], set_idx: u64, tag: u64) {
        match self {
            Self::Lru(state) | Self::LruPreferClean(state) => {
                state.record_access(set, set_idx, tag);
            }
            Self::Rand { .. } => {}
        }
    }

    /// Choose which line index (within the set) to evict.
    pub fn pick_victim(&mut self, set: &[CacheLine], set_idx: u64) -> usize {
        match self {
            Self::Lru(state) => state.lru_index(set_idx),

            Self::Rand { associativity } => {
                rand::rng().random_range(0..*associativity) as usize
            }

            Self::LruPreferClean(state) => {
                let ts = &state.timestamps[set_idx as usize];

                // Prefer the least-recently-used clean line.
                let clean_victim = set
                    .iter()
                    .enumerate()
                    .filter(|(_, line)| line.status != CacheStatus::Modified)
                    .min_by_key(|(i, _)| ts[*i])
                    .map(|(i, _)| i);

                // Fall back to overall LRU if every line is dirty.
                clean_victim.unwrap_or_else(|| state.lru_index(set_idx))
            }
        }
    }
}