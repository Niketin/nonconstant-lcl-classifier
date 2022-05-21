use super::GraphCacheParams;
use crate::caches::Cache;
use crate::BiregularGraph;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct GraphSqliteCache {
    db: Connection,
}

impl Cache<GraphCacheParams, BiregularGraph> for GraphSqliteCache {
    fn read(
        &self,
        params: GraphCacheParams,
    ) -> Result<Vec<BiregularGraph>, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.db.query_row(
            "SELECT data FROM multigraph_class WHERE nodes=?1 AND degree_a=?2 AND degree_p=?3",
            params![params.n, params.degree_a, params.degree_p],
            |row| row.get(0),
        )?;

        let graphs: Vec<BiregularGraph> = bincode::deserialize(&data).unwrap();

        Ok(graphs)
    }

    fn write(
        &mut self,
        params: GraphCacheParams,
        graphs: &[BiregularGraph],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(graphs)?;
        self.db.execute(
            "INSERT INTO multigraph_class (nodes, degree_a, degree_p, data) VALUES (?1, ?2, ?3, ?4)",
            params![params.n, params.degree_a, params.degree_p, data],
        )?;
        Ok(())
    }
}

impl GraphSqliteCache {
    pub fn new(path: &Path) -> Self {
        let connection = Self::open_connection(path).unwrap_or_else(|_|
            panic!(
                "Failed to connect to SQLite database. Is there a database at path {:?} ?",
                &path.to_str()
            )
        );
        Self { db: connection }
    }
    fn open_connection(path: &Path) -> Result<Connection> {
        Connection::open(path)
    }
}
