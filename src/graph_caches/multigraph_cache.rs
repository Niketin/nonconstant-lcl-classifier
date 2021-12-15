use super::GraphCache;
use crate::BiregularGraph;
use rusqlite::{params, Connection, DatabaseName::Main, Result};
use std::path::PathBuf;

pub struct SqliteCacheHandler {
    db: Connection,
}

impl GraphCache for SqliteCacheHandler {
    fn read_graphs(
        &self,
        n: usize,
        degree_a: usize,
        degree_p: usize,
    ) -> Result<Vec<BiregularGraph>, Box<dyn std::error::Error>> {
        let data: Vec<u8> = self.db.query_row(
            "SELECT data FROM class WHERE nodes=?1 AND deg_a=?2 AND deg_p=?3",
            params![n, degree_a, degree_p],
            |row| row.get(0),
        )?;

        let graphs: Vec<BiregularGraph> = bincode::deserialize(&data).unwrap();

        Ok(graphs)
    }

    fn write_graphs(
        &mut self,
        nodes: usize,
        degree_a: usize,
        degree_p: usize,
        graphs: &Vec<BiregularGraph>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(graphs)?;
        self.db.execute(
            "INSERT INTO class (nodes, deg_a, deg_p, data) VALUES (?1, ?2, ?3, ?4)",
            params![nodes, degree_a, degree_p, data],
        )?;
        Ok(())
    }

    fn get_path(&self) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        todo!()
    }

    fn has_path(&self) -> bool {
        false
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

pub fn create_database(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = rusqlite::Connection::open_in_memory()?;
    db.execute(
        "CREATE TABLE class (
                nodes           INTEGER NOT NULL,
                deg_a           INTEGER NOT NULL,
                deg_p           INTEGER NOT NULL,
                data            BLOB,
                CONSTRAINT class_pk PRIMARY KEY (nodes, deg_a, deg_p)
            );",
        [],
    )?;
    db.backup(Main, path, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_connection_in_memory() -> Result<(), Box<dyn std::error::Error>> {
        // let db_path = PathBuf::from_str("db/cache.db")?;
        // let db_handler = SqliteCacheHandler::new(db_path)?;
        let db = rusqlite::Connection::open_in_memory()?;
        db.execute(
            "CREATE TABLE class (
                    nodes           INTEGER NOT NULL,
                    deg_a           INTEGER NOT NULL,
                    deg_p           INTEGER NOT NULL,
                    data            BLOB,
                    CONSTRAINT class_pk PRIMARY KEY (nodes, deg_a, deg_p)
                );",
            [],
        )?;
        let n = 10;
        let active_degree = 3;
        let passive_degree = 3;
        let graphs = BiregularGraph::get_or_generate::<SqliteCacheHandler>(
            n,
            active_degree,
            passive_degree,
            None,
        );
        let graphs_len = graphs.len();
        let data = bincode::serialize(&graphs).unwrap();

        let result_insert = db.execute(
            "INSERT INTO class (nodes, deg_a, deg_p, data) VALUES (?1, ?2, ?3, ?4)",
            params![n, active_degree, passive_degree, data],
        )?;

        dbg!(result_insert);

        let data2: Result<Vec<u8>> = db.query_row(
            "SELECT data FROM class WHERE nodes=?1 AND deg_a=?2 AND deg_p=?3",
            params![n, active_degree, passive_degree],
            |row| row.get(0),
        );

        assert!(data2.is_ok());
        let graphs2: Vec<BiregularGraph> = bincode::deserialize(&data2.unwrap()).unwrap();
        assert_eq!(graphs2.len(), graphs_len);

        println!("{:?}", &graphs2.len());

        Ok(())
    }
}
