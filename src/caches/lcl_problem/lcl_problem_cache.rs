use super::LclProblemCache;
use crate::LclProblem;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
pub struct SqliteCacheHandler {
    db: Connection,
}

// TODO This module is a bit DRY.
// TODO Maybe there is a more general solution to caching lcl problems and multigraphs?

impl LclProblemCache for SqliteCacheHandler {
    fn get_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        panic!("This LCL problem class has no specific file")
    }

    fn has_path(&self) -> bool {
        false
    }

    fn read_problems(
        &self,
        degree_a: usize,
        degree_p: usize,
        label_count: usize,
    ) -> Result<Vec<crate::LclProblem>, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.db.query_row(
            "SELECT data FROM problem_class WHERE deg_a=?1 AND deg_p=?2 AND label_count=?3",
            params![degree_a, degree_p, label_count],
            |row| row.get(0),
        )?;

        let problems: Vec<LclProblem> = bincode::deserialize(&data).unwrap();

        Ok(problems)
    }

    fn write_problems(
        &mut self,
        degree_a: usize,
        degree_p: usize,
        label_count: usize,
        problems: &Vec<crate::LclProblem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(problems)?;
        self.db.execute(
            "INSERT INTO problem_class (deg_a, deg_p, label_count, data) VALUES (?1, ?2, ?3, ?4)",
            params![degree_a, degree_p, label_count, data],
        )?;
        Ok(())
    }
}

impl SqliteCacheHandler {
    pub fn new(path: PathBuf) -> Self {
        let connection = Self::open_connection(&path).expect(
            format!(
                "Failed to connect to SQLite database. Is there a database at path {:?} ?",
                &path.as_path().to_str()
            )
            .as_str(),
        );
        return Self { db: connection };
    }
    fn open_connection(path: &PathBuf) -> Result<Connection> {
        Connection::open(path.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing() -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
