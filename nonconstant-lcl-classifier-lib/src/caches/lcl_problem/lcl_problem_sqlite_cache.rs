use super::LclProblemCacheParams;
use crate::caches::Cache;
use crate::LclProblem;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct LclProblemSqliteCache {
    db: Connection,
}

impl LclProblemSqliteCache {
    pub fn new(path: &Path) -> Self {
        let connection = Self::open_connection(path).unwrap_or_else(|_| {
            panic!(
                "Failed to connect to SQLite database. Is there a database at path {:?} ?",
                &path
            )
        });
        Self { db: connection }
    }
    fn open_connection(path: &Path) -> Result<Connection> {
        Connection::open(path)
    }
}

impl Cache<LclProblemCacheParams, LclProblem> for LclProblemSqliteCache {
    fn read(
        &self,
        params: LclProblemCacheParams,
    ) -> Result<Vec<LclProblem>, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.db.query_row(
            "SELECT data FROM problem_class WHERE degree_a=?1 AND degree_p=?2 AND label_count=?3",
            params![params.degree_a, params.degree_p, params.label_count],
            |row| row.get(0),
        )?;

        let problems: Vec<LclProblem> = bincode::deserialize(&data).unwrap();

        Ok(problems)
    }

    fn write(
        &mut self,
        params: LclProblemCacheParams,
        problems: &[LclProblem],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(problems)?;
        self.db.execute(
            "INSERT INTO problem_class (degree_a, degree_p, label_count, data) VALUES (?1, ?2, ?3, ?4)",
            params![params.degree_a, params.degree_p, params.label_count, data],
        )?;
        Ok(())
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
