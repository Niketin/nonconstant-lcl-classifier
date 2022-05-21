pub mod graph;
pub mod lcl_problem;

pub use graph::multigraph_sqlite_cache::GraphSqliteCache;
pub use graph::GraphCacheParams;
pub use lcl_problem::lcl_problem_sqlite_cache::LclProblemSqliteCache;
pub use lcl_problem::LclProblemCacheParams;
use rusqlite::DatabaseName::Main;

pub trait Cache<P, T> {
    fn read(&self, params: P) -> Result<Vec<T>, Box<dyn std::error::Error>>;
    fn write(&mut self, params: P, data: &[T]) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn create_sqlite_cache(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = rusqlite::Connection::open_in_memory()?;
    db.execute(
        "CREATE TABLE multigraph_class (
                nodes           INTEGER NOT NULL,
                degree_a        INTEGER NOT NULL,
                degree_p        INTEGER NOT NULL,
                data            BLOB,
                CONSTRAINT multigraph_class_pk PRIMARY KEY (nodes, degree_a, degree_p)
            );",
        [],
    )?;
    db.execute(
        "CREATE TABLE problem_class (
                degree_a        INTEGER NOT NULL,
                degree_p        INTEGER NOT NULL,
                label_count     INTEGER NOT NULL,
                data            BLOB,
                CONSTRAINT problem_class_pk PRIMARY KEY (degree_a, degree_p, label_count)
            );",
        [],
    )?;
    db.backup(Main, path, None)?;
    Ok(())
}
