use super::PowersetCache;
use crate::Configurations;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
pub struct PowersetSqliteHandler {
    db: Connection,
}

// TODO This module is a bit DRY with multigraph cache.
// TODO Maybe there is a more general solution to caching lcl problems and multigraphs?

impl PowersetCache for PowersetSqliteHandler {
    fn get_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        panic!("This configuration powerset has no specific file")
    }

    fn has_path(&self) -> bool {
        false
    }

    fn read_powerset(
        &self,
        degree: usize,
        label_count: usize,
    ) -> Result<Vec<Configurations>, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.db.query_row(
            "SELECT data FROM configuration_powerset WHERE degree=?1 AND label_count=?2",
            params![degree, label_count],
            |row| row.get(0),
        )?;

        let powerset: Vec<Configurations> = bincode::deserialize(&data).unwrap();

        Ok(powerset)
    }

    fn write_powerset(
        &mut self,
        degree: usize,
        label_count: usize,
        powerset: &Vec<Configurations>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(powerset)?;
        self.db.execute(
            "INSERT INTO configuration_powerset (degree, label_count, data) VALUES (?1, ?2, ?3)",
            params![degree, label_count, data],
        )?;
        Ok(())
    }
}

impl PowersetSqliteHandler {
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
