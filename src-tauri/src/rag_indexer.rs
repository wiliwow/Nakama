use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexedDocumentMetadata {
    pub id: String,
    pub source_path: String,
    pub indexed_at: u64,
    pub file_hash: String,
    pub chunk_index: usize,
    pub chunk_total: usize,
    pub size_bytes: u64,
}

pub struct RAGIndex {
    index_dir: PathBuf,
    metadata: Arc<RwLock<HashMap<String, IndexedDocumentMetadata>>>,
}

impl RAGIndex {
    pub fn new(index_dir: PathBuf) -> Result<Self, String> {
        let _ = fs::create_dir_all(&index_dir);
        let idx = Self { index_dir, metadata: Arc::new(RwLock::new(HashMap::new())) };
        idx.load_from_disk()?;
        Ok(idx)
    }

    pub fn new_in_memory() -> Self {
        Self { index_dir: PathBuf::from("."), metadata: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn manifest_path(&self) -> PathBuf {
        self.index_dir.join("rag_manifest.json")
    }

    fn load_from_disk(&self) -> Result<(), String> {
        let manifest = self.manifest_path();
        if !manifest.exists() { return Ok(()); }
        let content = fs::read_to_string(&manifest)
            .map_err(|e| format!("failed to read rag manifest: {}", e))?;
        let data: HashMap<String, IndexedDocumentMetadata> = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse rag manifest: {}", e))?;
        Ok(())
    }

    async fn save_to_disk(&self) -> Result<(), String> {
        let manifest = self.manifest_path();
        // ensure parent dir
        let _ = fs::create_dir_all(manifest.parent().unwrap_or(&PathBuf::from(".")));
        let metadata = self.metadata.read().await;
        let content = serde_json::to_string_pretty(&*metadata)
            .map_err(|e| format!("failed to serialize rag manifest: {}", e))?;
        fs::write(&manifest, content)
            .map_err(|e| format!("failed to write rag manifest: {}", e))?;
        Ok(())
    }

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
            indexed_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            file_hash,
            chunk_index,
            chunk_total,
            size_bytes,
        };
        self.metadata.write().await.insert(doc_id, metadata);
        self.save_to_disk().await
    }

    pub async fn get(&self, doc_id: &str) -> Result<Option<IndexedDocumentMetadata>, String> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(doc_id).cloned())
    }

    pub async fn list_all(&self) -> Result<Vec<IndexedDocumentMetadata>, String> {
        let metadata = self.metadata.read().await;
        Ok(metadata.values().cloned().collect())
    }

    pub async fn clear(&self) -> Result<(), String> {
        self.metadata.write().await.clear();
        self.save_to_disk().await
    }
}

pub fn compute_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_indexed_bytes: u64,
    pub indexed_documents: usize,
    pub indexed_chunks: usize,
    pub last_indexed_at: Option<u64>,
    pub estimated_index_size_mb: u64,
}

impl Default for IndexStats {
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

pub async fn compute_stats(store: &RAGIndex) -> Result<IndexStats, String> {
    let all = store.list_all().await?;
    let mut stats = IndexStats::default();
    stats.indexed_documents = all.len();
    stats.indexed_chunks = all.iter().map(|m| m.chunk_total).sum();
    stats.total_indexed_bytes = all.iter().map(|m| m.size_bytes).sum();
    stats.estimated_index_size_mb = (stats.total_indexed_bytes / (1024 * 1024)).max(1);
    stats.last_indexed_at = all.iter().map(|m| m.indexed_at).max();
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rag_index_record() {
        let dir = std::env::temp_dir().join("test_rag");
        let _ = fs::create_dir_all(&dir);
        let idx = RAGIndex::new(dir.clone()).unwrap();
        let hash = compute_hash("hello world");
        let _ = idx.record_indexed("doc1".into(), "/path/to/doc.txt", hash, 1, 1, 11).await;
        let meta = idx.get("doc1").await.unwrap();
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().source_path, "/path/to/doc.txt");
        let _ = fs::remove_dir_all(dir);
    }
}
