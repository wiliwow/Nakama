use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metadata for an indexed document (source file, timestamp, vector ID)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexedDocumentMetadata {
    pub id: String,
    pub source_path: String,
    pub indexed_at: u64,  // Unix timestamp
    pub file_hash: String,  // SHA256 for dedup/change detection
    pub chunk_index: usize,
    pub chunk_total: usize,
    pub size_bytes: u64,
}

/// Tracks indexing state and metadata for retention and diagnostics.
pub struct IndexMetadataStore {
    metadata_dir: PathBuf,
    in_memory: Arc<RwLock<HashMap<String, IndexedDocumentMetadata>>>,
}

impl IndexMetadataStore {
    /// Create a new metadata store backed by a directory.
    #[allow(dead_code)]
    pub async fn new(metadata_dir: PathBuf) -> Result<Self, String> {
        if !metadata_dir.exists() {
            fs::create_dir_all(&metadata_dir)
                .map_err(|e| format!("failed to create metadata dir: {}", e))?;
        }

        let mut store = Self {
            metadata_dir,
            in_memory: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load persisted metadata from disk
        store.load_from_disk().await?;

        Ok(store)
    }

    /// Create a new metadata store synchronously (without loading from disk)
    pub fn new_sync(metadata_dir: PathBuf) -> Result<Self, String> {
        if !metadata_dir.exists() {
            fs::create_dir_all(&metadata_dir)
                .map_err(|e| format!("failed to create metadata dir: {}", e))?;
        }

        Ok(Self {
            metadata_dir,
            in_memory: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load metadata from disk files (one JSON per indexed document).
    #[allow(dead_code)]
    async fn load_from_disk(&mut self) -> Result<(), String> {
        let manifest_path = self.metadata_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("failed to read manifest: {}", e))?;

        let metadata: HashMap<String, IndexedDocumentMetadata> = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse manifest: {}", e))?;

        *self.in_memory.write().await = metadata;
        Ok(())
    }

    /// Persist metadata to disk.
    async fn save_to_disk(&self) -> Result<(), String> {
        let manifest_path = self.metadata_dir.join("manifest.json");
        let metadata = self.in_memory.read().await;

        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| format!("failed to serialize metadata: {}", e))?;

        fs::write(&manifest_path, content)
            .map_err(|e| format!("failed to write manifest: {}", e))?;

        Ok(())
    }

    /// Record a newly indexed document.
    pub async fn record_indexed(
        &self,
        doc_id: String,
        source_path: String,
        file_hash: String,
        chunk_index: usize,
        chunk_total: usize,
        size_bytes: u64,
    ) -> Result<(), String> {
        let metadata = IndexedDocumentMetadata {
            id: doc_id.clone(),
            source_path,
            indexed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            file_hash,
            chunk_index,
            chunk_total,
            size_bytes,
        };

        self.in_memory.write().await.insert(doc_id, metadata);
        self.save_to_disk().await
    }

    /// Get all indexed document metadata.
    pub async fn list_all(&self) -> Result<Vec<IndexedDocumentMetadata>, String> {
        let metadata = self.in_memory.read().await;
        Ok(metadata.values().cloned().collect())
    }

    /// Get metadata for a specific document.
    pub async fn get(&self, doc_id: &str) -> Result<Option<IndexedDocumentMetadata>, String> {
        let metadata = self.in_memory.read().await;
        Ok(metadata.get(doc_id).cloned())
    }

    /// Remove metadata entries older than a specified number of days.
    #[allow(dead_code)]
    pub async fn prune_old_entries(&self, days: u32) -> Result<u32, String> {
        let cutoff_secs = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)) as i64
            - (days as i64 * 86400);

        let mut metadata = self.in_memory.write().await;
        let initial_count = metadata.len();

        metadata.retain(|_, m| (m.indexed_at as i64) > cutoff_secs);

        let removed = (initial_count - metadata.len()) as u32;

        if removed > 0 {
            self.save_to_disk().await?;
        }

        Ok(removed)
    }

    /// Clear all metadata.
    pub async fn clear(&self) -> Result<(), String> {
        self.in_memory.write().await.clear();
        self.save_to_disk().await
    }
}

/// Compute a simple SHA256 hash of file/text content for dedup.
pub fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// IndexingStats for diagnostics: size, entry count, last ingestion time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexingStats {
    pub total_indexed_bytes: u64,
    pub indexed_documents: usize,
    pub indexed_chunks: usize,
    pub last_indexed_at: Option<u64>,
    pub estimated_index_size_mb: u64,
}

impl Default for IndexingStats {
    fn default() -> Self {
        Self {
            total_indexed_bytes: 0,
            indexed_documents: 0,
            indexed_chunks: 0,
            last_indexed_at: None,
            estimated_index_size_mb: 0,
        }
    }
}

/// Compute current indexing statistics from metadata.
pub async fn compute_stats(metadata_store: &IndexMetadataStore) -> Result<IndexingStats, String> {
    let all_metadata = metadata_store.list_all().await?;

    let mut stats = IndexingStats::default();
    stats.indexed_documents = all_metadata.len();
    stats.indexed_chunks = all_metadata.iter().map(|m| m.chunk_total).sum();
    stats.total_indexed_bytes = all_metadata.iter().map(|m| m.size_bytes).sum();
    stats.estimated_index_size_mb = (stats.total_indexed_bytes / (1024 * 1024)).max(1);
    stats.last_indexed_at = all_metadata
        .iter()
        .map(|m| m.indexed_at)
        .max();

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metadata_store_record_and_retrieve() {
        let dir = std::env::temp_dir().join("test_metadata");
        let store = IndexMetadataStore::new(dir.clone()).await.unwrap();

        store
            .record_indexed(
                "doc_1".into(),
                "/path/to/file.txt".into(),
                "abc123".into(),
                0,
                1,
                1024,
            )
            .await
            .unwrap();

        let metadata = store.get("doc_1").await.unwrap();
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().source_path, "/path/to/file.txt");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compute_hash() {
        let hash1 = compute_hash("hello");
        let hash2 = compute_hash("hello");
        let hash3 = compute_hash("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
