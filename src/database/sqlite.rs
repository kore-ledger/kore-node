// Copyright 2024 Kore Ledger
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # SQLite database backend.
//!
//! This module contains the SQLite database backend implementation.
//!

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OpenFlags, Result as SQLiteResult};

use kore_base::{DatabaseCollection, DatabaseManager, DbError as Error};

use crate::error::NodeError;

/// SQLite database manager.
pub struct SqliteManager {
    path: String,
}

impl SqliteManager {
    /// Create a new SQLite database manager.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
        }
    }
}

impl DatabaseManager<SqliteCollection> for SqliteManager {
    fn default() -> Self {
        Self::new(":memory:")
    }

    fn create_collection(&self, identifier: &str) -> SqliteCollection {
        let conn = open(&self.path).expect("fail SQLite open connection");
        let stmt = format!(
            "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, value BLOB NOT NULL)",
            identifier
        );
        conn.execute(stmt.as_str(), ())
            .expect("Cannot create table"); // empty list of parameters.
                                            //let conn = open(&self.path).expect("fail SQLite open connection");
        SqliteCollection::new(conn, identifier)
    }
}

/// SQLite collection
pub struct SqliteCollection {
    conn: Arc<Mutex<Connection>>,
    table: String,
}

impl SqliteCollection {
    /// Create a new SQLite collection.
    pub fn new(conn: Connection, table: &str) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            table: table.to_owned(),
        }
    }

    /// Create a new iterartor filtering by prefix.
    fn make_iter<'a>(
        &'a self,
        reverse: bool,
        prefix: &str,
    ) -> SQLiteResult<Box<dyn Iterator<Item = (String, Vec<u8>)> + 'a>> {
        let order = if reverse { "DESC" } else { "ASC" };
        let conn = self.conn.lock().expect("open connection");
        let query = format!("SELECT id, value FROM {} ORDER BY id {}", self.table, order);
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query([])?;
        let mut position_to_cut;
        let mut values = Vec::new();
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            if !key.starts_with(prefix) {
                continue;
            }
            position_to_cut = key.rfind(char::MAX).unwrap_or(0);
            values.push((key[position_to_cut..key.len()].to_string(), row.get(1)?));
        }
        Ok(Box::new(values.into_iter()))
    }
}

impl DatabaseCollection for SqliteCollection {
    fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::CustomError("open connection".to_owned()))?;
        let query = format!("SELECT value FROM {} WHERE id = ?1", &self.table);
        let row: Vec<u8> = conn
            .query_row(&query, params![key], |row| row.get(0))
            .map_err(|_| Error::EntryNotFound)?;

        Ok(row)
    }
    fn put(&self, key: &str, data: &[u8]) -> Result<(), Error> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::CustomError("open connection".to_owned()))?;
        let stmt = format!(
            "INSERT OR REPLACE INTO {} (id, value) VALUES (?1, ?2)",
            &self.table
        );
        conn.execute(&stmt, params![key, data])
            .map_err(|_| Error::CustomError("insert error".to_owned()))?;
        Ok(())
    }

    fn del(&self, key: &str) -> Result<(), Error> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| Error::CustomError("open connection".to_owned()))?;
        let stmt = format!("DELETE FROM {} WHERE id = ?1", &self.table);
        conn.execute(&stmt, params![key])
            .map_err(|_| Error::CustomError("delete error".to_owned()))?;
        Ok(())
    }

    fn iter<'a>(
        &'a self,
        reverse: bool,
        prefix: &str,
    ) -> Box<dyn Iterator<Item = (String, Vec<u8>)> + 'a> {
        match self.make_iter(reverse, prefix) {
            Ok(iter) => {
                let iterator = SQLiteIterator { iter };
                Box::new(iterator)
            }
            Err(_) => Box::new(std::iter::empty()),
        }
    }
}

pub struct SQLiteIterator<'a> {
    pub iter: Box<dyn Iterator<Item = (String, Vec<u8>)> + 'a>,
}

impl Iterator for SQLiteIterator<'_> {
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Open a SQLite database connection.
pub fn open<P: AsRef<Path>>(path: P) -> Result<Connection, NodeError> {
    let path = path.as_ref();
    let mut flags = OpenFlags::default();
    flags.insert(OpenFlags::SQLITE_OPEN_READ_WRITE);
    flags.insert(OpenFlags::SQLITE_OPEN_CREATE);
    let conn = Connection::open_with_flags(path, flags)
        .map_err(|_| NodeError::Database("SQLite fail open connection".to_owned()))?;
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;
        ",
    )
    .map_err(|_| NodeError::Database("SQListe fail execute batch".to_owned()))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {

    use super::*;
    use kore_base::{test_database_manager_trait, DbError as Error};

    test_database_manager_trait! {
        unit_test_sqlite_manager:SqliteManager:SqliteCollection
    }

    #[test]
    fn test_sqlite() {
        let db = SqliteManager::default();
        let first_collection = db.create_collection("first_example");

        let mut iter = first_collection.iter(false, "first_example");
        assert!(iter.next().is_none());
        build_state(&first_collection);
        // ITER TEST
        //let mut iter = first_collection.iter(false, "first".to_string());
        let mut iter = first_collection.iter(false, "");
        let (keys, data) = build_initial_data();
        for i in 0..3 {
            let (key, val) = iter.next().unwrap();
            assert_eq!(keys[i], key);
            assert_eq!(data[i], val);
        }
        assert!(iter.next().is_none());
        let mut iter = first_collection.iter(false, "a");
        for i in 0..2 {
            let (key, val) = iter.next().unwrap();
            assert_eq!(keys[i], key);
            assert_eq!(data[i], val);
        }
        assert!(iter.next().is_none());
    }

    fn build_state(collection: &SqliteCollection) {
        let data = get_data().unwrap();
        let result = collection.put("aa", &data[0]);
        assert!(result.is_ok());
        let result = collection.put("ab", &data[1]);
        assert!(result.is_ok());
        let result = collection.put("bc", &data[2]);
        assert!(result.is_ok());
    }

    #[allow(unused_imports)]
    use super::*;
    use borsh::{to_vec, BorshDeserialize, BorshSerialize};

    #[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq, Debug)]
    struct Data {
        id: usize,
        value: String,
    }

    #[allow(dead_code)]
    fn get_data() -> Result<Vec<Vec<u8>>, Error> {
        let data1 = Data {
            id: 1,
            value: "aa".into(),
        };
        let data2 = Data {
            id: 2,
            value: "ab".into(),
        };
        let data3 = Data {
            id: 3,
            value: "bc".into(),
        };
        #[rustfmt::skip] // let-else not supported yet
        let Ok(data1) = to_vec(&data1) else {
            return Err(Error::SerializeError);
        };
        #[rustfmt::skip] // let-else not supported yet
        let Ok(data2) = to_vec(&data2) else {
            return Err(Error::SerializeError);
        };
        #[rustfmt::skip] // let-else not supported yet
        let Ok(data3) = to_vec(&data3) else {
            return Err(Error::SerializeError);
        };
        Ok(vec![data1, data2, data3])
    }

    #[allow(dead_code)]
    fn build_initial_data() -> (Vec<&'static str>, Vec<Vec<u8>>) {
        let keys = vec!["aa", "ab", "bc"];
        let data = get_data().unwrap();
        let values = vec![data[0].to_owned(), data[1].to_owned(), data[2].to_owned()];
        (keys, values)
    }
}
