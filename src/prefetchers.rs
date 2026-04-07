
pub enum Prefetcher {
    Null,
    Adjacent,
    Sequential,
    Custom
}
impl Prefetcher {
    pub fn null() -> Self { Self::Null }
    pub fn adjacent() -> Self { Self::Adjacent }
    pub fn sequential() -> Self { Self::Sequential }
    pub fn custom() -> Self { Self::Custom }

    /// Takes in the address being accessed. \
    /// Prefetches relevant cache lines based on the strategy. \
    /// Returns the number of cache lines that were prefetched.
    pub fn handle_mem_access(
        &self,
        address: u64,
    ) -> u64 {
        match self {
            Prefetcher::Null => 0,
            Prefetcher::Adjacent => todo!(),
            Prefetcher::Sequential => todo!(),
            Prefetcher::Custom => todo!(),
        }
    }
}