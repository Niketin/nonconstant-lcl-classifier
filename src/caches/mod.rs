pub mod graph;
pub mod lcl_problem;

use rusqlite::DatabaseName::Main;

pub fn create_sqlite_cache(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = rusqlite::Connection::open_in_memory()?;
    db.execute(
        "CREATE TABLE multigraph_class (
                nodes           INTEGER NOT NULL,
                deg_a           INTEGER NOT NULL,
                deg_p           INTEGER NOT NULL,
                data            BLOB,
                CONSTRAINT multigraph_class_pk PRIMARY KEY (nodes, deg_a, deg_p)
            );",
        [],
    )?;
    db.execute(
        "CREATE TABLE problem_class (
                deg_a           INTEGER NOT NULL,
                deg_p           INTEGER NOT NULL,
                label_count     INTEGER NOT NULL,
                data            BLOB,
                CONSTRAINT problem_class_pk PRIMARY KEY (deg_a, deg_p, label_count)
            );",
        [],
    )?;
    db.backup(Main, path, None)?;
    Ok(())
}
