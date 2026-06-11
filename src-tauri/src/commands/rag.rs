use serde::{Serialize, Deserialize};
use crate::AppState;
use crate::rag_indexer::IndexStats;
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RetrievedPassage {
    pub id: String,
    pub score: f32,
    pub content: String,
    pub source: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthCheckResult {
    pub embeddings_ok: bool,
    pub embeddings_error: Option<String>,
    pub llm_ok: bool,
    pub llm_error: Option<String>,
}

#[tauri::command]
pub async fn rag_index(
    app_state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let _ = path;
    let _ = app_state;
    Err("RAG indexing requires external embedding provider. Not available in self-contained mode.".into())
}

#[tauri::command]
pub async fn rag_retrieve(
    query_text: String,
    top_k: Option<usize>,
) -> Result<Vec<RetrievedPassage>, String> {
    let _ = (query_text, top_k);
    Err("RAG retrieval requires external embedding provider. Not available in self-contained mode.".into())
}

#[tauri::command]
pub async fn rag_add_file(
    filename: String,
    content: String,
) -> Result<String, String> {
    let _ = (filename, content);
    Err("RAG indexing requires external embedding provider. Not available in self-contained mode.".into())
}

#[tauri::command]
pub async fn rag_health_check(
    app_state: State<'_, AppState>,
) -> Result<HealthCheckResult, String> {
    let config = {
        let mgr = app_state.config_manager.lock().unwrap();
        mgr.load().map_err(|e| format!("failed to load config: {}", e))?
    };
    let endpoint = config.llm_endpoint;
    let model = config.llm_model;
    let llm_ok = !endpoint.is_empty() && !model.is_empty();
    Ok(HealthCheckResult {
        embeddings_ok: false,
        embeddings_error: Some("Embeddings not available in self-contained mode".into()),
        llm_ok,
        llm_error: if llm_ok { None } else { Some("LLM endpoint not configured. Set LLM_ENDPOINT and LLM_MODEL.".into()) },
    })
}

#[tauri::command]
pub async fn rag_clear_index(
    app_state: State<'_, AppState>,
) -> Result<u32, String> {
    let mut store = app_state.metadata_store.lock().await;
    store.clear().await.map_err(|e| format!("Failed to clear index: {}", e))?;
    Ok(0)
}

#[tauri::command]
pub async fn rag_index_stats(
    app_state: State<'_, AppState>,
) -> Result<IndexStats, String> {
    let store = app_state.metadata_store.lock().await;
    crate::rag_indexer::compute_stats(&*store).await
}
