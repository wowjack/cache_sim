use rand::{Rng, RngExt};

use crate::memory_system::CacheLine;


pub enum Prefetcher {
    Null,
    Adjacent,
    Sequential(u64),
    Custom {
        min: Option<u64>,
        max: Option<u64>,
        locality: u64
    }
}
impl Prefetcher {
    pub fn null() -> Self { Self::Null }
    pub fn adjacent() -> Self { Self::Adjacent }
    pub fn sequential(num: u64) -> Self { Self::Sequential(num) }
    pub fn custom(locality: u64) -> Self { Self::Custom { min: None, max: None, locality } }

    /// Takes in the address being accessed, and the number of offset bits. \
    /// We need to know the number of offset bits so we can be certain to fetch the next cache line. \
    /// Returns addresses that need to be prefetched.
    /// This is a bit of a hack to get around borrow checker problems caused by the CacheSystem type architecture.
    /// Prefetcher is a field of CacheSystem, so cannot directly invoke the mutable CacheSystem method.
    /// Oh well, I don't care enough the redesign the type structure.
    /// The CacheSystem will call the access method itself on the addresses returned here. \
    /// Also take in the content of the cache line being accessed and the hashset of all accesses for use in the custom strategy
    pub fn handle_mem_access(
        &mut self,
        address: u64,
        offset_bits: u32,
        _line_contents: CacheLine,
    ) -> Vec<u64> {
        let line_size = 1 << offset_bits;
        match self {
            Prefetcher::Null => vec![],
            Prefetcher::Adjacent => {
                // Prefetch the next cache line
                let next_line = (address & !(line_size - 1)) + line_size;
                return vec![next_line]
            },
            Prefetcher::Sequential(n) => {
                let get_nth = |n: u64| (address & !(line_size - 1)) + n*line_size;
                (1..=(*n as u64)).map(get_nth).collect()
            },
            Prefetcher::Custom { min, max, locality} => {
                let line_size = 2_u64.pow(offset_bits);

                if min.is_none() || min.unwrap() < address {
                    *min = Some(address);
                }
                if max.is_none() || max.unwrap() > address {
                    *max = Some(address)
                }

                let min = min.unwrap_or(0);
                let max = max.unwrap_or(u64::MAX);

                let rng = rand::rng();
                let fake_line: Vec<u8> = rng.random_iter().take(line_size as usize).collect();
                let mut prefetches = Vec::new();

                for bytes in fake_line.windows(4) {
                    let addr = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                    if addr < (max.saturating_add(line_size * *locality)) as u32
                        && addr > (min.saturating_sub(line_size * *locality)) as u32
                    {
                        prefetches.push(addr as u64);
                    }
                }
                
                return prefetches
            },
        }
    }
}