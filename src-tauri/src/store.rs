use crate::error::AppError;
use rusqlite::{Connection, Result as RusqliteResult, Row};
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Manager};

pub trait Storable: Sized {
    fn from_row(row: &Row) -> RusqliteResult<Self>;
}

pub struct Store {
    db: Mutex<Connection>,
}

impl Store {
    pub fn new(app_handle: &AppHandle, db_filename: &str) -> Result<Self, AppError> {
        let data_dir = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|_| AppError::DirectoryNotFound)?;
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }
        let db_path = data_dir.join(db_filename);
        let db = Connection::open(db_path)?;

        Ok(Self { db: Mutex::new(db) })
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, AppError> {
        let db = Connection::open_in_memory()?;
        Ok(Self { db: Mutex::new(db) })
    }

    pub fn init_table(&self, schema: &str) -> Result<(), AppError> {
        self.conn().execute(schema, [])?;
        Ok(())
    }

    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.db.lock().expect("store db mutex poisoned")
    }

    pub fn query<T: Storable, P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, AppError> {
        let db = self.conn();
        let mut stmt = db.prepare(sql)?;
        let iter = stmt.query_map(params, T::from_row)?;
        iter.collect::<RusqliteResult<Vec<_>>>()
            .map_err(|e| e.into())
    }

    pub fn query_row<T: Storable, P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<T>, AppError> {
        let db = self.conn();
        let mut stmt = db.prepare(sql)?;
        let mut iter = stmt.query_map(params, T::from_row)?;

        if let Some(row) = iter.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn execute<P: rusqlite::Params>(&self, sql: &str, params: P) -> Result<usize, AppError> {
        self.conn().execute(sql, params).map_err(|e| e.into())
    }

    pub fn last_insert_rowid(&self) -> i64 {
        self.conn().last_insert_rowid()
    }
}
