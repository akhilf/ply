// ai-generated
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistMode {
    Disk,
    Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkRow {
    pub url: String,
    pub title: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitRow {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visited_at: i64,
}

#[derive(Debug)]
pub struct StoreError;

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "store unavailable")
    }
}

impl std::error::Error for StoreError {}

pub struct Store {
    conn: Option<Connection>,
    persist_mode: PersistMode,
}

impl Store {
    pub fn open_path(path: &Path) -> Self {
        match Connection::open(path).and_then(|conn| {
            init_schema(&conn)?;
            Ok(conn)
        }) {
            Ok(conn) => Self {
                conn: Some(conn),
                persist_mode: PersistMode::Disk,
            },
            Err(_) => Self::open_memory_only(),
        }
    }

    pub fn open_memory_only() -> Self {
        Self {
            conn: None,
            persist_mode: PersistMode::Memory,
        }
    }

    pub fn persist_mode(&self) -> PersistMode {
        self.persist_mode
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        let Some(conn) = self.conn.as_ref() else {
            return false;
        };
        conn.query_row(
            "SELECT 1 FROM bookmarks WHERE url = ?1",
            params![url],
            |_| Ok(()),
        )
        .optional()
        .ok()
        .flatten()
        .is_some()
    }

    pub fn toggle_bookmark(&mut self, url: &str, title: &str) -> Result<bool, StoreError> {
        let conn = self.conn.as_ref().ok_or(StoreError)?;
        if self.is_bookmarked(url) {
            conn.execute("DELETE FROM bookmarks WHERE url = ?1", params![url])
                .map_err(|_| StoreError)?;
            Ok(false)
        } else {
            let created_at = now_secs();
            conn.execute(
                "INSERT INTO bookmarks (url, title, created_at) VALUES (?1, ?2, ?3)",
                params![url, title, created_at],
            )
            .map_err(|_| StoreError)?;
            Ok(true)
        }
    }

    pub fn append_visit(&mut self, url: &str, title: &str) -> Result<(), StoreError> {
        let conn = self.conn.as_ref().ok_or(StoreError)?;
        conn.execute(
            "INSERT INTO visits (url, title, visited_at) VALUES (?1, ?2, ?3)",
            params![url, title, now_secs()],
        )
        .map_err(|_| StoreError)?;
        Ok(())
    }

    pub fn list_bookmarks(&self) -> Vec<BookmarkRow> {
        let Some(conn) = self.conn.as_ref() else {
            return Vec::new();
        };
        let mut stmt = match conn.prepare(
            "SELECT url, title, created_at FROM bookmarks ORDER BY created_at DESC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        stmt.query_map([], |row| {
            Ok(BookmarkRow {
                url: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    pub fn list_history(&self, limit: usize) -> Vec<VisitRow> {
        let Some(conn) = self.conn.as_ref() else {
            return Vec::new();
        };
        let mut stmt = match conn.prepare(
            "SELECT id, url, title, visited_at FROM visits ORDER BY visited_at DESC, id DESC LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        stmt.query_map(params![limit as i64], |row| {
            Ok(VisitRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }

    pub fn search_history(&self, query: &str) -> Vec<VisitRow> {
        let Some(conn) = self.conn.as_ref() else {
            return Vec::new();
        };
        let mut stmt = match conn.prepare(
            "SELECT id, url, title, visited_at FROM visits
             WHERE LOWER(url) LIKE '%' || LOWER(?1) || '%'
                OR LOWER(title) LIKE '%' || LOWER(?1) || '%'
             ORDER BY visited_at DESC, id DESC",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        stmt.query_map(params![query], |row| {
            Ok(VisitRow {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default()
    }
}

fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            url TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS visits (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            visited_at INTEGER NOT NULL
        );",
    )
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark_toggle_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Store::open_path(&dir.path().join("ply.sqlite"));
        assert_eq!(s.persist_mode(), PersistMode::Disk);
        assert!(!s.is_bookmarked("https://a.test/"));
        assert_eq!(s.toggle_bookmark("https://a.test/", "A").unwrap(), true);
        assert!(s.is_bookmarked("https://a.test/"));
        assert_eq!(s.list_bookmarks()[0].title, "A");
        assert_eq!(s.toggle_bookmark("https://a.test/", "A").unwrap(), false);
        assert!(s.list_bookmarks().is_empty());
    }

    #[test]
    fn visit_and_search() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Store::open_path(&dir.path().join("ply.sqlite"));
        s.append_visit("https://a.test/x", "Alpha").unwrap();
        s.append_visit("https://b.test/y", "Beta").unwrap();
        assert_eq!(s.list_history(10)[0].title, "Beta");
        let hits = s.search_history("alpha");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].url, "https://a.test/x");
    }

    #[test]
    fn unavailable_db_is_memory_and_writes_fail() {
        let mut s = Store::open_memory_only();
        assert_eq!(s.persist_mode(), PersistMode::Memory);
        assert!(s.toggle_bookmark("https://a.test/", "A").is_err());
        assert!(s.append_visit("https://a.test/", "A").is_err());
        assert!(s.list_history(10).is_empty());
    }
}
// ai-generated end
