pub mod multigraph_sqlite_cache;

#[derive(Clone, Copy)]
pub struct GraphCacheParams {
    pub n: usize,
    pub degree_a: usize,
    pub degree_p: usize,
}
