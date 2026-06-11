use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalMemoryItem {
    pub id: Option<i64>,
    pub namespace: String,
    pub content: String,
    pub score: Option<f64>,
    pub created_at: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

impl From<LocalMemoryItem> for crate::models::StashMemory {
    fn from(m: LocalMemoryItem) -> Self {
        Self {
            id: m.id.map(|i| i.to_string()),
            content: m.content,
            score: m.score,
            namespace: Some(m.namespace),
            created_at: m.created_at.map(|ts| ts.to_string()),
            metadata: m.metadata,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalFact {
    pub id: Option<i64>,
    pub namespace: String,
    pub fact: String,
    pub confidence: Option<f64>,
}

impl From<LocalFact> for crate::models::StashFact {
    fn from(f: LocalFact) -> Self {
        Self {
            id: f.id.map(|i| i.to_string()),
            fact: f.fact,
            confidence: f.confidence,
            namespace: Some(f.namespace),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalGoal {
    pub id: Option<i64>,
    pub namespace: String,
    pub description: String,
    pub status: String,
    pub progress: Option<f64>,
}

impl From<LocalGoal> for crate::models::StashGoal {
    fn from(g: LocalGoal) -> Self {
        Self {
            id: g.id.map(|i| i.to_string()),
            description: g.description,
            status: g.status,
            progress: g.progress,
            namespace: Some(g.namespace),
        }
    }
}

pub struct LocalMemoryClient {
    conn: Connection,
    fts: Arc<RwLock<HashMap<String, Vec<i64>>>>,
}

impl LocalMemoryClient {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let _ = std::fs::create_dir_all(db_path.parent().unwrap_or(&PathBuf::from(".")));
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open memory db: {}", e))?;
        let client = Self { conn, fts: Arc::new(RwLock::new(HashMap::new())) };
        client.init_schema()?;
        Ok(client)
    }

    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory db: {}", e))?;
        let client = Self { conn, fts: Arc::new(RwLock::new(HashMap::new())) };
        client.init_schema()?;
        Ok(client)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS facts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace TEXT NOT NULL,
                fact TEXT NOT NULL,
                confidence REAL,
                created_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL,
                created_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS failures (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                namespace TEXT NOT NULL,
                what TEXT NOT NULL,
                why TEXT NOT NULL,
                lesson TEXT NOT NULL,
                created_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS working_context (
                namespace TEXT PRIMARY KEY,
                context TEXT,
                updated_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_episodes_ns ON episodes(namespace);
            CREATE INDEX IF NOT EXISTS idx_facts_ns ON facts(namespace);
            CREATE INDEX IF NOT EXISTS idx_goals_ns ON goals(namespace);"
        ).map_err(|e| format!("Schema init failed: {}", e))?;
        Ok(())
    }

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    pub fn remember(&self, namespace: &str, content: &str, metadata: Option<serde_json::Value>) -> Result<i64, String> {
        let now = Self::now();
        self.conn.execute(
            "INSERT INTO episodes (namespace, content, metadata, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![namespace, content, metadata.map(|v| v.to_string()), now],
        ).map_err(|e| format!("remember insert failed: {}", e))?;
        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    pub fn recall(&self, namespace: &str, query: &str, limit: Option<usize>) -> Result<Vec<crate::models::StashMemory>, String> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        let limit = limit.unwrap_or(5);
        let query_lower = query.to_lowercase();
        let pattern1 = format!("%{}%", keywords.first().map(|s| *s).unwrap_or(""));
        let pattern2 = keywords.get(1).map(|s| format!("%{}%", s)).unwrap_or_else(|| pattern1.clone());
        let pattern3 = keywords.get(2).map(|s| format!("%{}%", s)).unwrap_or_else(|| pattern1.clone());
        let mut stmt = self.conn.prepare(
            "SELECT id, content, created_at FROM episodes WHERE namespace = ?1 AND (LOWER(content) LIKE ?2 OR LOWER(content) LIKE ?3 OR LOWER(content) LIKE ?4) ORDER BY created_at DESC LIMIT ?5"
        ).map_err(|e| format!("recall prepare failed: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![namespace, pattern1, pattern2, pattern3, limit], |row| {
            Ok(LocalMemoryItem {
                id: Some(row.get(0)?),
                namespace: namespace.to_string(),
                content: row.get(1)?,
                score: Some(0.5),
                created_at: row.get(2)?,
                metadata: None,
            })
        }).map_err(|e| format!("recall query failed: {}", e))?;
        let mut results = Vec::new();
        for row in rows { results.push(crate::models::StashMemory::from(row.map_err(|e| e.to_string())?)); }
        Ok(results)
    }

    pub fn set_context(&self, namespace: &str, context: &str) -> Result<(), String> {
        let now = Self::now();
        self.conn.execute(
            "INSERT OR REPLACE INTO working_context (namespace, context, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![namespace, context, now],
        ).map_err(|e| format!("set_context failed: {}", e))?;
        Ok(())
    }

    pub fn get_context(&self, namespace: &str) -> Result<Option<String>, String> {
        let mut stmt = self.conn.prepare("SELECT context FROM working_context WHERE namespace = ?1")
            .map_err(|e| format!("get_context prepare failed: {}", e))?;
        let result = stmt.query_row(rusqlite::params![namespace], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn clear_context(&self, namespace: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM working_context WHERE namespace = ?1", rusqlite::params![namespace])
            .map_err(|e| format!("clear_context failed: {}", e))?;
        Ok(())
    }

    pub fn list_namespaces(&self) -> Result<Vec<String>, String> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT namespace FROM episodes")
            .map_err(|e| format!("list_namespaces failed: {}", e))?;
        let rows = stmt.query_map([], |row| row.get(0))
            .map_err(|e| format!("list_namespaces query failed: {}", e))?;
        let mut namespaces = Vec::new();
        for row in rows { namespaces.push(row.unwrap_or_default()); }
        Ok(namespaces)
    }

    pub fn create_namespace(&self, namespace: &str) -> Result<(), String> {
        self.conn.execute(
            "INSERT OR IGNORE INTO episodes (namespace, content, created_at) VALUES (?1, '', ?2)",
            rusqlite::params![namespace, Self::now()],
        ).map_err(|e| format!("create_namespace failed: {}", e))?;
        Ok(())
    }

    pub fn consolidate(&self, namespace: &str) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content FROM episodes WHERE namespace = ?1 ORDER BY created_at DESC LIMIT 20"
        ).map_err(|e| format!("consolidate prepare failed: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![namespace], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| format!("consolidate query failed: {}", e))?;
        for row in rows {
            let (id, content) = row.map_err(|e| e.to_string())?;
            let fact_text = format!("Consolidated from episode {}: {}", id, content.chars().take(100).collect::<String>());
            let now = Self::now();
            let _ = self.conn.execute(
                "INSERT OR IGNORE INTO facts (namespace, fact, confidence, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![namespace, fact_text, 0.8, now],
            );
        }
        Ok(())
    }

    pub fn query_facts(&self, namespace: &str) -> Result<Vec<crate::models::StashFact>, String> {
        let mut stmt = self.conn.prepare("SELECT id, fact, confidence FROM facts WHERE namespace = ?1")
            .map_err(|e| format!("query_facts prepare failed: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![namespace], |row| {
            Ok(LocalFact {
                id: Some(row.get(0)?),
                namespace: namespace.to_string(),
                fact: row.get(1)?,
                confidence: row.get(2)?,
            })
        }).map_err(|e| format!("query_facts query failed: {}", e))?;
        let mut results = Vec::new();
        for row in rows { results.push(crate::models::StashFact::from(row.map_err(|e| e.to_string())?)); }
        Ok(results)
    }

    pub fn list_goals(&self, namespace: &str) -> Result<Vec<crate::models::StashGoal>, String> {
        let mut stmt = self.conn.prepare("SELECT id, description, status, progress FROM goals WHERE namespace = ?1")
            .map_err(|e| format!("list_goals prepare failed: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![namespace], |row| {
            Ok(LocalGoal {
                id: Some(row.get(0)?),
                namespace: namespace.to_string(),
                description: row.get(1)?,
                status: row.get(2)?,
                progress: row.get(3)?,
            })
        }).map_err(|e| format!("list_goals query failed: {}", e))?;
        let mut results = Vec::new();
        for row in rows { results.push(crate::models::StashGoal::from(row.map_err(|e| e.to_string())?)); }
        Ok(results)
    }

    pub fn create_goal(&self, namespace: &str, description: &str) -> Result<i64, String> {
        let now = Self::now();
        self.conn.execute(
            "INSERT INTO goals (namespace, description, status, progress, created_at) VALUES (?1, ?2, 'in_progress', 0.0, ?3)",
            rusqlite::params![namespace, description, now],
        ).map_err(|e| format!("create_goal failed: {}", e))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn complete_goal(&self, namespace: &str, goal_id: i64) -> Result<(), String> {
        self.conn.execute("UPDATE goals SET status = 'completed' WHERE id = ?1 AND namespace = ?2", rusqlite::params![goal_id, namespace])
            .map_err(|e| format!("complete_goal failed: {}", e))?;
        Ok(())
    }

    pub fn create_failure(&self, namespace: &str, what: &str, why: &str, lesson: &str) -> Result<(), String> {
        let now = Self::now();
        self.conn.execute(
            "INSERT INTO failures (namespace, what, why, lesson, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![namespace, what, why, lesson, now],
        ).map_err(|e| format!("create_failure failed: {}", e))?;
        Ok(())
    }

    pub fn get_summary(&self) -> Result<crate::models::MemorySummary, String> {
        let ns = self.list_namespaces()?;
        let mut total_facts = 0usize;
        let mut total_goals = 0usize;
        let mut total_failures = 0usize;
        let mut total_episodes = 0usize;
        for ns_name in &ns {
            total_facts += self.query_facts(ns_name).map(|v| v.len()).unwrap_or(0);
            total_goals += self.list_goals(ns_name).map(|v| v.len()).unwrap_or(0);
            total_failures += self.count_failures(ns_name).unwrap_or(0);
            total_episodes += self.count_episodes(ns_name).unwrap_or(0);
        }
        Ok(crate::models::MemorySummary {
            total_episodes,
            total_facts,
            total_goals,
            total_failures,
            namespaces: ns,
        })
    }

    fn count_failures(&self, namespace: &str) -> Result<usize, String> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM failures WHERE namespace = ?1", rusqlite::params![namespace], |r| r.get(0))
            .map_err(|e| format!("count_failures failed: {}", e))?;
        Ok(count as usize)
    }

    fn count_episodes(&self, namespace: &str) -> Result<usize, String> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM episodes WHERE namespace = ?1", rusqlite::params![namespace], |r| r.get(0))
            .map_err(|e| format!("count_episodes failed: {}", e))?;
        Ok(count as usize)
    }
}
