use serde::{Serialize, Deserialize};

/// Lightweight DTO returned to the frontend for retrieved passages
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RetrievedPassage {
    pub id: String,
    pub score: f32,
    pub content: String,
    /// Optional source identifier (file path, name, etc.) pulled from metadata
    pub source: Option<String>,
}

/// Index files under `path` using Swiftide pipelines (async). This command wires to the
/// Swiftide FastEmbed + LanceDB integration pattern. Files are discovered recursively,
/// chunked, embedded, and stored in a local LanceDB table.
#[tauri::command]
pub async fn rag_index(
    path: String,
    #[cfg(feature = "swiftide_integration")]
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
    #[cfg(feature = "swiftide_integration")]
    _metadata_store: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::rag_indexer::IndexMetadataStore>>>,
) -> Result<String, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Err("swiftide integration not enabled in this build".into());
    }

    #[cfg(feature = "swiftide_integration")]
    {
        use swiftide::{indexing, integrations};
        use swiftide::indexing::EmbeddedField;

        // Load configuration
        let config = {
            let mgr = config_manager.lock().unwrap();
            mgr.load().map_err(|e| format!("failed to load config: {}", e))?
        };

        let lancedb_uri = config.lancedb_path.to_string_lossy().to_string();

        // Initialize FastEmbed for dense embeddings
        let fastembed = match integrations::fastembed::FastEmbed::try_default() {
            Ok(c) => c,
            Err(e) => return Err(format!("failed to init fastembed: {}", e)),
        };

        // Build a LanceDB client using config values
        let lancedb = match integrations::lancedb::LanceDB::builder()
            .uri(lancedb_uri)
            .vector_size(config.embedding_vector_size as i32)
            .with_vector(EmbeddedField::Combined)
            .table_name(config.table_name.clone())
            .build()
        {
            Ok(c) => c,
            Err(e) => return Err(format!("failed to init lancedb: {}", e)),
        };

        // Run the indexing pipeline: load files -> chunk -> embed -> store
        let pipeline = indexing::Pipeline::from_loader(
            indexing::loaders::FileLoader::new(&path)
        )
        .then_chunk(indexing::transformers::ChunkMarkdown::from_chunk_range(config.chunk_size_min..config.chunk_size_max))
        .then_in_batch(
            indexing::transformers::Embed::new(fastembed.clone()).with_batch_size(config.embed_batch_size)
        )
        .then_store_with(lancedb);

        match pipeline.run().await {
            Ok(_) => Ok("indexed".into()),
            Err(e) => Err(format!("indexing failed: {}", e)),
        }
    }
}

/// Retrieve top-k passages for a query and return them to the frontend. The handler returns
/// a list of `RetrievedPassage` that can be fed to your generation step (e.g., `ask_ai`).
#[tauri::command]
pub async fn rag_retrieve(
    query_text: String,
    top_k: Option<usize>,
    #[cfg(feature = "swiftide_integration")]
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<Vec<RetrievedPassage>, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Err("swiftide integration not enabled in this build".into());
    }

    #[cfg(feature = "swiftide_integration")]
    {
        use swiftide::{query, integrations};
        use swiftide::indexing::EmbeddedField;
        use swiftide_core::querying::states;

        // Load configuration
        let config = {
            let mgr = config_manager.lock().unwrap();
            mgr.load().map_err(|e| format!("failed to load config: {}", e))?
        };

        let lancedb_uri = config.lancedb_path.to_string_lossy().to_string();
        let k = top_k.unwrap_or(5);

        // FastEmbed client for embedding the query
        let fastembed = integrations::fastembed::FastEmbed::try_default()
            .map_err(|e| format!("failed to init fastembed: {}", e))?;

        // LanceDB retrieval client using config values
        let lancedb = integrations::lancedb::LanceDB::builder()
            .uri(lancedb_uri)
            .vector_size(config.embedding_vector_size as i32)
            .with_vector(EmbeddedField::Combined)
            .table_name(config.table_name.clone())
            .build()
            .map_err(|e| format!("failed to init lancedb: {}", e))?;

        // Build a query pipeline: embed query -> retrieve from lancedb -> pass-through answer
        // (We use a pass-through answerer since we only need retrieval, not LLM generation)
        let pipeline = query::Pipeline::default()
            .then_transform_query(query::query_transformers::Embed::from_client(fastembed.clone()))
            .then_retrieve(lancedb)
            .then_answer(|query: swiftide_core::querying::Query<states::Retrieved>| {
                Ok(query.answered(""))
            });

        let result = pipeline
            .query(&query_text)
            .await
            .map_err(|e| format!("query pipeline error: {}", e))?;

        // Map retrieved documents to DTOs. Extract content and score from metadata.
        let mut out = Vec::new();
        for (idx, doc) in result.documents.iter().enumerate().take(k) {
            let id = format!("doc_{}", idx);
            
            // Try to extract score from metadata, default to 0.0
            let score = doc
                .metadata()
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            // Attempt to get path/source info from metadata
            let source = doc
                .metadata()
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    doc.metadata()
                        .get("source")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            
            let content = doc.content().to_string();

            out.push(RetrievedPassage { id, score, content, source });
        }

        Ok(out)
    }
}

/// Add a file directly to the RAG index without requiring it on disk.
/// Useful for indexing files sent/uploaded via the app UI.
/// The file content is chunked, embedded, and stored in LanceDB.
#[tauri::command]
pub async fn rag_add_file(
    filename: String,
    content: String,
    #[cfg(feature = "swiftide_integration")]
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
    #[cfg(feature = "swiftide_integration")]
    metadata_store: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::rag_indexer::IndexMetadataStore>>>,
) -> Result<String, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Err("swiftide integration not enabled in this build".into());
    }

    #[cfg(feature = "swiftide_integration")]
    {
        // Input validation
        if filename.is_empty() {
            return Err("Filename cannot be empty".into());
        }
        if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
            return Err("Invalid filename: must not contain path separators or '..'".into());
        }
        const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if content.len() > MAX_FILE_SIZE {
            return Err(format!("File too large: maximum allowed is {} bytes", MAX_FILE_SIZE));
        }

        use swiftide::{indexing, integrations};
        use swiftide::indexing::EmbeddedField;
        use crate::rag_indexer::compute_hash;

        // Load configuration
        let config = {
            let mgr = config_manager.lock().unwrap();
            mgr.load().map_err(|e| format!("failed to load config: {}", e))?
        };

        let lancedb_uri = config.lancedb_path.to_string_lossy().to_string();

        // Check for deduplication - skip if file already indexed
        let file_hash = compute_hash(&content);
        {
            let store = metadata_store.lock().await;
            if let Ok(Some(existing)) = store.get(&filename).await {
                if existing.file_hash == file_hash {
                    return Ok(format!("file '{}' already indexed (unchanged)", filename));
                }
            }
        }

        // Initialize FastEmbed for dense embeddings
        let fastembed = match integrations::fastembed::FastEmbed::try_default() {
            Ok(c) => c,
            Err(e) => return Err(format!("failed to init fastembed: {}", e)),
        };

        // Build a LanceDB client using config values
        let lancedb = match integrations::lancedb::LanceDB::builder()
            .uri(lancedb_uri)
            .vector_size(config.embedding_vector_size as i32)
            .with_vector(EmbeddedField::Combined)
            .table_name(config.table_name.clone())
            .build()
        {
            Ok(c) => c,
            Err(e) => return Err(format!("failed to init lancedb: {}", e)),
        };

        // Create a node (indexing unit) from the uploaded file content
        let node = indexing::Node::<String>::builder()
            .chunk(content.clone())
            .path(&filename)
            .metadata(swiftide_core::indexing::Metadata::from([("source", filename.clone())]))
            .build()
            .map_err(|e| format!("failed to create node: {}", e))?;

        // Create a pipeline: chunk the node, embed it, and store it
        let pipeline = indexing::Pipeline::from_stream(vec![node])
            .then_chunk(indexing::transformers::ChunkMarkdown::from_chunk_range(config.chunk_size_min..config.chunk_size_max))
            .then_in_batch(
                indexing::transformers::Embed::new(fastembed.clone()).with_batch_size(config.embed_batch_size)
            )
            .then_store_with(lancedb);

        match pipeline.run().await {
            Ok(_) => {
                // Record in metadata store
                let store = metadata_store.lock().await;
                let _ = store.record_indexed(
                    filename.clone(),
                    filename.clone(),
                    file_hash,
                    0, // chunk_index - we'll need to track this properly
                    1, // chunk_total - we'll need to track this properly
                    content.len() as u64,
                ).await;

                Ok(format!("file '{}' indexed", filename))
            },
            Err(e) => Err(format!("failed to index file: {}", e)),
        }
    }
}
/// Health check for RAG system: verify Ollama is reachable and LanceDB is accessible.
#[tauri::command]
pub async fn rag_health_check(
    #[cfg(feature = "swiftide_integration")]
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<HealthCheckResult, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Ok(HealthCheckResult {
            embeddings_ok: false,
            embeddings_error: Some("swiftide integration not enabled in this build".into()),
            llm_ok: false,
            llm_error: Some("swiftide integration not enabled in this build".into()),
        });
    }

    #[cfg(feature = "swiftide_integration")]
    {
        use swiftide::integrations;
        use swiftide::indexing::EmbeddedField;

        // Load configuration
        let config = {
            let mgr = config_manager.lock().unwrap();
            mgr.load().map_err(|e| format!("failed to load config: {}", e))?
        };

        let lancedb_uri = config.lancedb_path.to_string_lossy().to_string();

        let mut result = HealthCheckResult {
            embeddings_ok: false,
            embeddings_error: None,
            llm_ok: false,
            llm_error: None,
        };

        // Check embeddings (FastEmbed)
        match integrations::fastembed::FastEmbed::try_default() {
            Ok(_) => result.embeddings_ok = true,
            Err(e) => result.embeddings_error = Some(format!("{}", e)),
        }

        // Check LanceDB accessibility using config values
        match integrations::lancedb::LanceDB::builder()
            .uri(lancedb_uri)
            .vector_size(config.embedding_vector_size as i32)
            .with_vector(EmbeddedField::Combined)
            .table_name(config.table_name.clone())
            .build()
        {
            Ok(_) => {},
            Err(e) => {
                result.embeddings_ok = false;
                result.embeddings_error = Some(format!("LanceDB init failed: {}", e));
            }
        }

        // Check Ollama (LLM backend) using config URL
        match reqwest::Client::new()
            .get(&format!("{}/api/tags", config.llm_backend_url.trim_end_matches('/')))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => result.llm_ok = true,
            Ok(_) => result.llm_error = Some("Ollama returned non-success status".into()),
            Err(e) => result.llm_error = Some(format!("Failed to reach Ollama: {}", e)),
        }

        Ok(result)
    }
}

/// Health check result DTO.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthCheckResult {
    pub embeddings_ok: bool,
    pub embeddings_error: Option<String>,
    pub llm_ok: bool,
    pub llm_error: Option<String>,
}

/// Clear all indexed data (dangerous!). Returns count of removed documents.
#[tauri::command]
pub async fn rag_clear_index(
    #[cfg(feature = "swiftide_integration")]
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
    #[cfg(feature = "swiftide_integration")]
    metadata_store: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::rag_indexer::IndexMetadataStore>>>,
) -> Result<u32, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Err("swiftide integration not enabled in this build".into());
    }

    #[cfg(feature = "swiftide_integration")]
    {
        use std::fs;
        use crate::rag_indexer::compute_stats;

        // Load configuration
        let config = {
            let mgr = config_manager.lock().unwrap();
            mgr.load().map_err(|e| format!("failed to load config: {}", e))?
        };

        // Get the LanceDB directory path from config
        let lancedb_path = config.lancedb_path;

        // Count documents before clearing (for return value)
        let document_count = {
            let store = metadata_store.lock().await;
            match compute_stats(&*store).await {
                Ok(stats) => stats.indexed_documents as u32,
                Err(_) => 0, // If we can't get stats, proceed with clearing
            }
        };

        // For LanceDB, we need to drop the table and clear metadata
        // Since LanceDB doesn't expose a direct drop_table API in swiftide wrapper,
        // we'll delete the table directory and metadata
        let table_path = lancedb_path.join(format!("{}.lance", config.table_name));

        if table_path.exists() {
            if let Err(e) = fs::remove_dir_all(&table_path) {
                return Err(format!("Failed to remove LanceDB table directory: {}", e));
            }
        }

        // Clear metadata store
        let store = metadata_store.lock().await;
        if let Err(e) = store.clear().await {
            return Err(format!("Failed to clear metadata store: {}", e));
        }

        Ok(document_count)
    }
}

/// Get index statistics (documents indexed, estimated size, etc).
#[tauri::command]
pub async fn rag_index_stats(
    #[cfg(feature = "swiftide_integration")]
    metadata_store: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::rag_indexer::IndexMetadataStore>>>,
) -> Result<IndexStats, String> {
    #[cfg(not(feature = "swiftide_integration"))]
    {
        return Ok(IndexStats {
            indexed_documents: 0,
            indexed_chunks: 0,
            total_indexed_bytes: 0,
            last_indexed_at: None,
            estimated_index_size_mb: 0,
        });
    }

    #[cfg(feature = "swiftide_integration")]
    {
        use crate::rag_indexer::compute_stats;

        let store = metadata_store.lock().await;
        let stats = compute_stats(&*store).await?;
        Ok(IndexStats {
            indexed_documents: stats.indexed_documents,
            indexed_chunks: stats.indexed_chunks,
            total_indexed_bytes: stats.total_indexed_bytes,
            last_indexed_at: stats.last_indexed_at,
            estimated_index_size_mb: stats.estimated_index_size_mb,
        })
    }
}

/// Index statistics DTO.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexStats {
    pub indexed_documents: usize,
    pub indexed_chunks: usize,
    pub total_indexed_bytes: u64,
    pub last_indexed_at: Option<u64>,
    pub estimated_index_size_mb: u64,
}
