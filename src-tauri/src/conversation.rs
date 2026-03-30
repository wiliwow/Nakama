use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Option<i64>,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<i64>,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ConversationMessage>,
}

pub struct ConversationStore {
    conn: Connection,
}

impl ConversationStore {
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&config_dir)?;
        let db_path = config_dir.join("conversations.db");
        let conn = Connection::open(db_path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(ConversationStore { conn })
    }

    pub fn create_conversation(&self, title: String) -> Result<Conversation> {
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO conversations (title, created_at, updated_at) VALUES (?, ?, ?)",
            [&title, &now.to_rfc3339(), &now.to_rfc3339()],
        )?;

        let id = self.conn.last_insert_rowid();

        Ok(Conversation {
            id: Some(id),
            title,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        })
    }

    pub fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        // Update conversation metadata
        if let Some(id) = conversation.id {
            tx.execute(
                "UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?",
                [
                    &conversation.title,
                    &conversation.updated_at.to_rfc3339(),
                    &id.to_string(),
                ],
            )?;

            // Delete existing messages
            tx.execute("DELETE FROM messages WHERE conversation_id = ?", [&id])?;

            // Insert new messages
            for message in &conversation.messages {
                tx.execute(
                    "INSERT INTO messages (conversation_id, role, content, timestamp, metadata) VALUES (?, ?, ?, ?, ?)",
                    [
                        &id.to_string(),
                        &message.role,
                        &message.content,
                        &message.timestamp.to_rfc3339(),
                        &message.metadata.as_ref().map(|m| m.to_string()).unwrap_or_default(),
                    ],
                )?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_conversation(&self, id: i64) -> Result<Option<Conversation>> {
        // Load conversation metadata
        let mut stmt = self.conn.prepare(
            "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?"
        )?;
        let mut rows = stmt.query([id])?;

        let mut conversation = if let Some(row) = rows.next()? {
            let id_val: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let created_at_str: String = row.get(2)?;
            let updated_at_str: String = row.get(3)?;

            Conversation {
                id: Some(id_val),
                title,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .into(),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .into(),
                messages: Vec::new(),
            }
        } else {
            return Ok(None);
        };

        // Load messages
        let mut stmt = self.conn.prepare(
            "SELECT id, role, content, timestamp, metadata FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC"
        )?;
        let mut message_rows = stmt.query([id])?;

        while let Some(row) = message_rows.next()? {
            let timestamp_str: String = row.get(3)?;
            let metadata_str: Option<String> = row.get(4)?;

            let message = ConversationMessage {
                id: Some(row.get(0)?),
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .into(),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            };
            conversation.messages.push(message);
        }

        Ok(Some(conversation))
    }

    pub fn list_conversations(&self) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
        )?;
        let mut rows = stmt.query([])?;

        let mut conversations = Vec::new();
        while let Some(row) = rows.next()? {
            let id_val: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let created_at_str: String = row.get(2)?;
            let updated_at_str: String = row.get(3)?;

            let conversation = Conversation {
                id: Some(id_val),
                title,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .into(),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    .into(),
                messages: Vec::new(), // Don't load messages for list view
            };
            conversations.push(conversation);
        }

        Ok(conversations)
    }

    pub fn delete_conversation(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM conversations WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn add_message(&self, conversation_id: i64, message: ConversationMessage) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO messages (conversation_id, role, content, timestamp, metadata) VALUES (?, ?, ?, ?, ?)",
            [
                &conversation_id.to_string(),
                &message.role,
                &message.content,
                &message.timestamp.to_rfc3339(),
                &message.metadata.as_ref().map(|m| m.to_string()).unwrap_or_default(),
            ],
        )?;

        // Update conversation updated_at
        self.conn.execute(
            "UPDATE conversations SET updated_at = ? WHERE id = ?",
            [Utc::now().to_rfc3339(), conversation_id.to_string()],
        )?;

        Ok(self.conn.last_insert_rowid())
    }
}