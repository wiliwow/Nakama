use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration for RAG system, embeddings, LLM backend, and retention policies.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RagConfig {
    /// Path to LanceDB data directory (defaults to ~/.lancedb)
    pub lancedb_path: PathBuf,

    /// Vector size used by FastEmbed (default 384)
    pub embedding_vector_size: i32,

    /// Embedding model to use (e.g., "BAAI/bge-small-en-v1.5")
    pub embedding_model: String,

    /// LanceDB table name for documents
    pub table_name: String,

    /// Chunk size range for document chunking (min..max characters)
    pub chunk_size_min: usize,
    pub chunk_size_max: usize,

    /// Batch size for embedding operations
    pub embed_batch_size: usize,

    /// Maximum index size in MB before sharding (0 = no sharding)
    pub max_shard_size_mb: u64,

    /// LLM backend URL (defaults to http://localhost:11434 for local Ollama)
    pub llm_backend_url: String,

    /// Enable retention policy (delete old entries after a certain age)
    pub retention_enabled: bool,

    /// Retention policy: delete entries older than this many days (default 90)
    pub retention_days: u32,

    /// Enable local-only enforcement (no cloud uploads)
    pub local_only: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            lancedb_path: home.join(".lancedb"),
            embedding_vector_size: 384,
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
            table_name: "nakama_documents".to_string(),
            chunk_size_min: 256,
            chunk_size_max: 2048,
            embed_batch_size: 8,
            max_shard_size_mb: 500,
            llm_backend_url: "http://localhost:11434".to_string(),
            retention_enabled: false,
            retention_days: 90,
            local_only: true,
        }
    }
}

/// Manages RAG configuration persistence (JSON file in AppData).
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new config manager. The config file lives in a platform-specific AppData dir.
    pub fn new(app_data_dir: PathBuf) -> Self {
        let config_path = app_data_dir.join("rag_config.json");
        if !app_data_dir.exists() {
            let _ = fs::create_dir_all(&app_data_dir);
        }
        ConfigManager { config_path }
    }

    /// Load config from disk, or return default if not found.
    pub fn load(&self) -> Result<RagConfig, String> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)
                .map_err(|e| format!("failed to read config: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("failed to parse config: {}", e))
        } else {
            Ok(RagConfig::default())
        }
    }

    /// Save config to disk.
    #[allow(dead_code)]
    pub fn save(&self, config: &RagConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("failed to serialize config: {}", e))?;
        fs::write(&self.config_path, content)
            .map_err(|e| format!("failed to write config: {}", e))?;
        Ok(())
    }

    /// Get the config file path (for debugging).
    #[allow(dead_code)]
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = RagConfig::default();
        assert_eq!(cfg.embedding_vector_size, 384);
        assert_eq!(cfg.chunk_size_min, 256);
        assert_eq!(cfg.chunk_size_max, 2048);
    }

    #[test]
    fn test_config_serialization() {
        let cfg = RagConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: RagConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.embedding_vector_size, cfg.embedding_vector_size);
    }
}
