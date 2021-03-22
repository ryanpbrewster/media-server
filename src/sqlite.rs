use async_trait::async_trait;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use rusqlite::{Connection, ToSql, NO_PARAMS};

use crate::{error::Error, StorageClient};

pub struct SqliteClient {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteClient {
    pub fn new(path: PathBuf) -> Result<SqliteClient, Error> {
        let mut connection = rusqlite::Connection::open(path)?;
        initialize_sqlite(&mut connection)?;
        Ok(SqliteClient {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}
#[async_trait]
impl StorageClient for SqliteClient {
    async fn fetch_object(&self, name: &str) -> Result<Bytes, Error> {
        let raw: Vec<u8> = self.connection.lock().unwrap().query_row(
            "SELECT raw FROM objects WHERE id = ?",
            &[name],
            |row| Ok(row.get_unwrap("raw")),
        )?;
        Ok(Bytes::from(raw))
    }
    async fn create_object(&self, name: &str, bytes: Bytes) -> Result<(), Error> {
        let raw = bytes.to_vec();
        let params: &[&dyn ToSql] = &[&name, &raw];
        self.connection
            .lock()
            .unwrap()
            .execute("INSERT INTO objects (id, raw) VALUES (?, ?)", params)?;
        Ok(())
    }
}

fn initialize_sqlite(connection: &mut Connection) -> Result<(), Error> {
    connection.execute(
        r#"
        CREATE TABLE IF NOT EXISTS objects (
            id TEXT NOT NULL,
            raw BLOB NOT NULL,

            PRIMARY KEY (id)
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
