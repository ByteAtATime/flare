use crate::error::AppError;
use crate::snippets::types::Snippet;
use crate::store::{Storable, Store};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;
use tauri::AppHandle;

const SNIPPETS_SCHEMA: &str = "CREATE TABLE IF NOT EXISTS snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    keyword TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)";

#[derive(Clone)]
pub struct SnippetManager {
    store: Arc<Store>,
}

impl Storable for Snippet {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let created_at_ts: i64 = row.get(4)?;
        let updated_at_ts: i64 = row.get(5)?;
        let last_used_at_ts: i64 = row.get(7)?;
        Ok(Snippet {
            id: row.get(0)?,
            name: row.get(1)?,
            keyword: row.get(2)?,
            content: row.get(3)?,
            created_at: DateTime::from_timestamp_nanos(created_at_ts),
            updated_at: DateTime::from_timestamp_nanos(updated_at_ts),
            times_used: row.get(6)?,
            last_used_at: DateTime::from_timestamp_nanos(last_used_at_ts),
        })
    }
}

impl SnippetManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let store = Store::new(app_handle, "snippets.sqlite")?;
        store.init_table(SNIPPETS_SCHEMA)?;

        {
            let db = store.conn();
            let mut stmt = db.prepare("PRAGMA table_info(snippets)")?;
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get(1))?
                .collect::<Result<Vec<_>, _>>()?;

            if !columns.contains(&"times_used".to_string()) {
                db.execute(
                    "ALTER TABLE snippets ADD COLUMN times_used INTEGER NOT NULL DEFAULT 0",
                    [],
                )?;
            }
            if !columns.contains(&"last_used_at".to_string()) {
                db.execute(
                    "ALTER TABLE snippets ADD COLUMN last_used_at INTEGER NOT NULL DEFAULT 0",
                    [],
                )?;
            }

            // Add index for faster keyword lookups
            db.execute(
                "CREATE INDEX IF NOT EXISTS idx_snippets_keyword ON snippets(keyword)",
                [],
            )?;
        }

        Ok(Self {
            store: Arc::new(store),
        })
    }

    #[cfg(test)]
    pub fn new_for_test() -> Result<Self, AppError> {
        let store = Store::new_in_memory()?;
        store.init_table(SNIPPETS_SCHEMA)?;

        {
            let db = store.conn();
            db.execute(
                "ALTER TABLE snippets ADD COLUMN times_used INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
            db.execute(
                "ALTER TABLE snippets ADD COLUMN last_used_at INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }

        Ok(Self {
            store: Arc::new(store),
        })
    }

    pub fn create_snippet(
        &self,
        name: String,
        keyword: String,
        content: String,
    ) -> Result<i64, AppError> {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        self.store.execute(
            "INSERT INTO snippets (name, keyword, content, created_at, updated_at, times_used, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?4, 0, 0)",
            params![name, keyword, content, now],
        )?;
        Ok(self.store.last_insert_rowid())
    }

    pub fn list_snippets(&self, search_term: Option<String>) -> Result<Vec<Snippet>, AppError> {
        let mut query = "SELECT id, name, keyword, content, created_at, updated_at, times_used, last_used_at FROM snippets".to_string();

        if let Some(term) = search_term {
            if !term.is_empty() {
                query.push_str(" WHERE name LIKE ?1 OR keyword LIKE ?1 OR content LIKE ?1");
                query.push_str(" ORDER BY updated_at DESC");
                let search_param = format!("%{}%", term);
                return self.store.query(&query, params![search_param]);
            }
        }

        query.push_str(" ORDER BY updated_at DESC");
        self.store.query(&query, [])
    }

    pub fn update_snippet(
        &self,
        id: i64,
        name: String,
        keyword: String,
        content: String,
    ) -> Result<(), AppError> {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        self.store.execute(
            "UPDATE snippets SET name = ?1, keyword = ?2, content = ?3, updated_at = ?4 WHERE id = ?5",
            params![name, keyword, content, now, id],
        )?;
        Ok(())
    }

    pub fn delete_snippet(&self, id: i64) -> Result<(), AppError> {
        self.store
            .execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn snippet_was_used(&self, id: i64) -> Result<(), AppError> {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        self.store.execute(
            "UPDATE snippets SET times_used = times_used + 1, last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub fn find_snippet_by_keyword(&self, keyword: &str) -> Result<Option<Snippet>, AppError> {
        self.store.query_row(
            "SELECT id, name, keyword, content, created_at, updated_at, times_used, last_used_at FROM snippets WHERE keyword = ?1",
            params![keyword],
        )
    }

    pub fn find_snippet_by_name(&self, name: &str) -> Result<Option<Snippet>, AppError> {
        self.store.query_row(
            "SELECT id, name, keyword, content, created_at, updated_at, times_used, last_used_at FROM snippets WHERE name = ?1 ORDER BY updated_at DESC LIMIT 1",
            params![name],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn test_create_and_list_snippets() {
        let manager = SnippetManager::new_for_test().unwrap();
        manager
            .create_snippet(
                "Test Snippet".into(),
                "testkey".into(),
                "This is a test.".into(),
            )
            .unwrap();

        let snippets = manager.list_snippets(None).unwrap();
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "Test Snippet");
        assert_eq!(snippets[0].keyword, "testkey");
        assert_eq!(snippets[0].content, "This is a test.");
    }

    #[test]
    fn test_create_snippet_with_duplicate_keyword_fails() {
        let manager = SnippetManager::new_for_test().unwrap();
        manager
            .create_snippet("First".into(), "dupkey".into(), "content1".into())
            .unwrap();

        let result = manager.create_snippet("Second".into(), "dupkey".into(), "content2".into());

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Rusqlite(rusqlite::Error::SqliteFailure(e, _)) => {
                assert_eq!(e.code, rusqlite::ErrorCode::ConstraintViolation);
            }
            _ => panic!("Expected a database constraint violation"),
        }
    }

    #[test]
    fn test_list_snippets_with_search() {
        let manager = SnippetManager::new_for_test().unwrap();
        manager
            .create_snippet(
                "Email Signature".into(),
                "sig".into(),
                "Best regards".into(),
            )
            .unwrap();
        manager
            .create_snippet(
                "Boilerplate".into(),
                "bp".into(),
                "Some email content".into(),
            )
            .unwrap();

        assert_eq!(
            manager.list_snippets(Some("email".into())).unwrap().len(),
            2
        );
        assert_eq!(manager.list_snippets(Some("sig".into())).unwrap().len(), 1);
        assert_eq!(
            manager.list_snippets(Some("regards".into())).unwrap().len(),
            1
        );
        assert_eq!(
            manager.list_snippets(Some("nothing".into())).unwrap().len(),
            0
        );
    }

    #[test]
    fn test_update_snippet() {
        let manager = SnippetManager::new_for_test().unwrap();
        let id = manager
            .create_snippet("Original".into(), "orig".into(), "original content".into())
            .unwrap();

        manager
            .update_snippet(
                id,
                "Updated".into(),
                "updated".into(),
                "updated content".into(),
            )
            .unwrap();

        let snippet = manager.find_snippet_by_keyword("updated").unwrap().unwrap();
        assert_eq!(snippet.id, id);
        assert_eq!(snippet.name, "Updated");
        assert_eq!(snippet.content, "updated content");
    }

    #[test]
    fn test_delete_snippet() {
        let manager = SnippetManager::new_for_test().unwrap();
        let id = manager
            .create_snippet("To Delete".into(), "del".into(), "delete me".into())
            .unwrap();
        assert_eq!(manager.list_snippets(None).unwrap().len(), 1);
        manager.delete_snippet(id).unwrap();
        assert!(manager.list_snippets(None).unwrap().is_empty());
    }

    #[test]
    fn test_snippet_was_used() {
        let manager = SnippetManager::new_for_test().unwrap();
        let id = manager
            .create_snippet("Test".into(), "test".into(), "test content".into())
            .unwrap();

        let snippet1 = manager.find_snippet_by_keyword("test").unwrap().unwrap();
        assert_eq!(snippet1.times_used, 0);

        manager.snippet_was_used(id).unwrap();
        let snippet2 = manager.find_snippet_by_keyword("test").unwrap().unwrap();
        assert_eq!(snippet2.times_used, 1);
        assert!(snippet2.last_used_at.timestamp() > 0);

        manager.snippet_was_used(id).unwrap();
        let snippet3 = manager.find_snippet_by_keyword("test").unwrap().unwrap();
        assert_eq!(snippet3.times_used, 2);
    }

    #[test]
    fn test_find_snippet_by_name() {
        let manager = SnippetManager::new_for_test().unwrap();
        manager
            .create_snippet("Unique Name".into(), "key1".into(), "content1".into())
            .unwrap();
        thread::sleep(Duration::from_millis(10));
        manager
            .create_snippet("Shared Name".into(), "key2".into(), "content2".into())
            .unwrap();
        thread::sleep(Duration::from_millis(10));
        let newest_id = manager
            .create_snippet("Shared Name".into(), "key3".into(), "newest".into())
            .unwrap();

        let found = manager.find_snippet_by_name("Shared Name").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, newest_id);

        let not_found = manager.find_snippet_by_name("Non Existent").unwrap();
        assert!(not_found.is_none());
    }
}
