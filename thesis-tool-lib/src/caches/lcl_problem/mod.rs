pub mod lcl_problem_sqlite_cache;

#[derive(Clone, Copy)]
pub struct LclProblemCacheParams {
    pub degree_a: usize,
    pub degree_p: usize,
    pub label_count: usize,
}
