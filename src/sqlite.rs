use std::path::PathBuf;

use bytes::Bytes;
use rusqlite::{Connection, ToSql, NO_PARAMS};

use crate::error::Error;

pub struct SqliteClient {
    connection: Connection,
    bucket: String,
}

impl SqliteClient {
    pub fn new(path: PathBuf, bucket: String) -> Result<SqliteClient, Error> {
        let mut connection = rusqlite::Connection::open(path)?;
        initialize_sqlite(&mut connection)?;
        Ok(SqliteClient { connection, bucket })
    }
    pub async fn fetch_object(&self, name: &str) -> Result<Bytes, Error> {
        let raw: Vec<u8> = self.connection.query_row(
            "SELECT raw FROM objects WHERE bucket = ? AND id = ?",
            &[&self.bucket, name],
            |row| Ok(row.get_unwrap("raw")),
        )?;
        Ok(Bytes::from(raw))
    }
    pub async fn create_object(&self, name: &str, bytes: Bytes) -> Result<(), Error> {
        let raw = bytes.to_vec();
        let params: Vec<&dyn ToSql> = vec![&self.bucket, &name, &raw];
        self.connection.execute(
            "INSERT INTO objects (bucket, id, raw) VALUES (?, ?, ?)",
            &params,
        )?;
        Ok(())
    }
}

fn initialize_sqlite(connection: &mut Connection) -> Result<(), Error> {
    connection.execute(
        r#"
        CREATE TABLE IF NOT EXISTS objects (
            bucket TEXT NOT NULL,
            id TEXT NOT NULL,
            raw BLOB NOT NULL,
        )
    "#,
        NO_PARAMS,
    )?;
    Ok(())
}

pub fn safe_file_name(mut root: PathBuf, name: String) -> Option<PathBuf> {
    let name = PathBuf::from(name);
    if name.is_relative() {
        root.push(name);
        Some(root)
    } else {
        None
    }
}
