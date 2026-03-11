use crate::replacement_policies::ReplacementPolicy;

const ADDRESS_BITS: u32 = 32;

#[derive(Debug, Default)]
pub struct Stats {
    pub accesses: u64,
    pub hits: u64,
    pub misses: u64,
    pub dirty_evictions: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Invalid,
    Exclusive,
    Modified,
}


#[derive(Debug, Clone)]
pub struct CacheLine {
    pub tag: u64,
    pub status: CacheStatus,
}

impl Default for CacheLine {
    fn default() -> Self {
        Self { tag: 0, status: CacheStatus::Invalid }
    }
}

struct Geometry {
    offset_bits: u32,
    index_bits: u32,
    tag_bits: u32,
    offset_mask: u64,
    index_mask: u64,
}

impl Geometry {
    fn new(line_size: u64, num_sets: u64) -> Self {
        let offset_bits = line_size.trailing_zeros();
        let index_bits = num_sets.trailing_zeros();
        let tag_bits = ADDRESS_BITS - offset_bits - index_bits;
        Self {
            offset_bits,
            index_bits,
            tag_bits,
            offset_mask: (1u64 << offset_bits) - 1,
            index_mask: (1u64 << index_bits) - 1,
        }
    }

    fn decode(&self, addr: u64) -> (u64, u64, u64) {
        let offset = addr & self.offset_mask;
        let set_idx = (addr >> self.offset_bits) & self.index_mask;
        let tag = addr >> (self.offset_bits + self.index_bits);
        (tag, set_idx, offset)
    }
}

pub struct CacheSystem {
    pub stats: Stats,
    associativity: u64,
    _num_sets: u64,
    geometry: Geometry,
    lines: Vec<CacheLine>,
    policy: ReplacementPolicy,
}

impl CacheSystem {
    pub fn new(
        cache_size: u64,
        num_lines: u64,
        associativity: u64,
        policy_name: &str,
    ) -> Self {
        let line_size = cache_size / num_lines;
        let num_sets = num_lines / associativity;
        let geometry = Geometry::new(line_size, num_sets);

        println!("Parameter Info");
        println!("==============");
        println!("Replacement Policy: {policy_name}");
        println!("Cache Size: {cache_size}");
        println!("Cache Lines: {num_lines}");
        println!("Associativity: {associativity}");
        println!("Line Size: {line_size}B");
        println!("Number of Sets: {num_sets}");

        println!("\nCache System Geometry:");
        println!("Index bits: {}", geometry.index_bits);
        println!("Offset bits: {}", geometry.offset_bits);
        println!("Tag bits: {}", geometry.tag_bits);
        println!("Offset mask: 0x{:x}", geometry.offset_mask);
        println!(
            "Set index mask: 0x{:x}\n",
            u64::MAX >> geometry.tag_bits
        );

        let policy = match policy_name {
            "LRU" => ReplacementPolicy::lru(num_sets, associativity),
            "RAND" => ReplacementPolicy::rand(associativity),
            "LRU_PREFER_CLEAN" => ReplacementPolicy::lru_prefer_clean(num_sets, associativity),
            other => panic!("Unknown replacement policy: {other}"),
        };

        Self {
            stats: Stats::default(),
            associativity,
            _num_sets: num_sets,
            geometry,
            lines: vec![CacheLine::default(); (num_sets * associativity) as usize],
            policy,
        }
    }

    /// Immutable view of a single set.
    fn set(&self, set_idx: u64) -> &[CacheLine] {
        let start = (set_idx * self.associativity) as usize;
        &self.lines[start..start + self.associativity as usize]
    }

    /// Mutable view of a single set.
    fn set_mut(&mut self, set_idx: u64) -> &mut [CacheLine] {
        let start = (set_idx * self.associativity) as usize;
        &mut self.lines[start..start + self.associativity as usize]
    }

    /// Find the index *within the set* of the line matching `tag`, if any valid one exists.
    fn find_in_set(&self, set_idx: u64, tag: u64) -> Option<usize> {
        self.set(set_idx)
            .iter()
            .position(|line| line.status != CacheStatus::Invalid && line.tag == tag)
    }

    /// Compute the range of indices in `self.lines` for a given set.
    fn set_range(&self, set_idx: u64) -> std::ops::Range<usize> {
        let start = (set_idx * self.associativity) as usize;
        start..start + self.associativity as usize
    }

    pub fn access(&mut self, addr: u64, rw: char) -> Result<(), String> {
        self.stats.accesses += 1;
        let (tag, set_idx, _offset) = self.geometry.decode(addr);
        let is_write = rw == 'W';

        if let Some(idx) = self.find_in_set(set_idx, tag) {
            // ---- Hit ----
            //println!("  0x{addr:x} hit: set {set_idx}, tag 0x{tag:x}, offset {offset}");
            self.stats.hits += 1;
            if is_write {
                self.set_mut(set_idx)[idx].status = CacheStatus::Modified;
            }
        } else {
            // ---- Miss ----
            //println!("  0x{addr:x} miss");
            self.stats.misses += 1;

            let slot = self.allocate_slot(set_idx)?;

            //println!("  store cache line with tag 0x{tag:x} in set {set_idx} index {slot}");
            let line = &mut self.set_mut(set_idx)[slot];
            line.tag = tag;
            line.status = if is_write {
                CacheStatus::Modified
            } else {
                CacheStatus::Exclusive
            };
        }

        // Notify the policy *after* the line is installed/updated.
        // Borrow `self.lines` and `self.policy` as disjoint fields to
        // satisfy the borrow checker.
        let r = self.set_range(set_idx);
        self.policy.notify_access(&self.lines[r], set_idx, tag);
        Ok(())
    }

    /// Return an empty slot index, or evict one and return it.
    fn allocate_slot(&mut self, set_idx: u64) -> Result<usize, String> {
        // Prefer an empty (invalid) slot.
        if let Some(i) = self
            .set(set_idx)
            .iter()
            .position(|l| l.status == CacheStatus::Invalid)
        {
            return Ok(i);
        }

        // Ask the policy to choose a victim.
        // Access `self.lines` and `self.policy` as disjoint fields.
        let r = self.set_range(set_idx);
        let victim = self.policy.pick_victim(&self.lines[r], set_idx);

        if victim >= self.associativity as usize {
            return Err(format!("Eviction index {victim} is outside of the set!"));
        }

        let status = self.set(set_idx)[victim].status;
        if status == CacheStatus::Modified {
            self.stats.dirty_evictions += 1;
        }
        //println!("  evict {status} cache line from set {set_idx} index {victim}");

        Ok(victim)
    }

    pub fn print_stats(&self) {
        let ratio = if self.stats.accesses > 0 {
            self.stats.hits as f64 / self.stats.accesses as f64
        } else {
            0.0
        };
        println!("\n\nStatistics");
        println!("==========");
        println!("OUTPUT ACCESSES {}", self.stats.accesses);
        println!("OUTPUT HITS {}", self.stats.hits);
        println!("OUTPUT MISSES {}", self.stats.misses);
        println!("OUTPUT DIRTY EVICTIONS {}", self.stats.dirty_evictions);
        println!("OUTPUT HIT RATIO {ratio:.8}");
    }
}