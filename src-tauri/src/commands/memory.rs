use crate::local_memory::LocalMemoryClient;
use crate::memory_agent::MemoryAgent;
use crate::models::{MemorySummary, StashFact, StashGoal, StashMemory};
use crate::AppState;
use tauri::State;
use std::path::PathBuf;

fn open_local_memory(db_path: &PathBuf) -> Result<LocalMemoryClient, String> {
    LocalMemoryClient::new(db_path.clone())
}

#[tauri::command]
pub fn memory_init(
    app_state: State<'_, AppState>,
) -> Result<MemorySummary, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    agent.init_default_namespaces()?;
    agent.get_summary()
}

#[tauri::command]
pub fn memory_remember(
    namespace: String,
    content: String,
    metadata: Option<serde_json::Value>,
    app_state: State<'_, AppState>,
) -> Result<i64, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    agent.remember(&namespace, &content, metadata.as_ref())
}

#[tauri::command]
pub fn memory_recall(
    query: String,
    namespace: Option<String>,
    limit: Option<usize>,
    app_state: State<'_, AppState>,
) -> Result<Vec<StashMemory>, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    match namespace {
        Some(ns) => agent.recall_from(&ns, &query, limit),
        None => agent.recall_all(&query, limit),
    }
}

#[tauri::command]
pub fn memory_query_facts(
    namespace: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<Vec<StashFact>, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    match namespace {
        Some(ns) => agent.query_facts_ns(&ns),
        None => agent.get_facts(),
    }
}

#[tauri::command]
pub fn memory_list_goals(
    namespace: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<Vec<StashGoal>, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    match namespace {
        Some(ns) => agent.list_goals_ns(&ns),
        None => agent.get_goals(),
    }
}

#[tauri::command]
pub fn memory_create_goal(
    namespace: Option<String>,
    description: String,
    app_state: State<'_, AppState>,
) -> Result<i64, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    let ns = namespace.unwrap_or_else(|| "nakama:goals".to_string());
    agent.create_goal(&ns, &description)
}

#[tauri::command]
pub fn memory_complete_goal(
    namespace: Option<String>,
    goal_id: i64,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    let ns = namespace.unwrap_or_else(|| "nakama:goals".to_string());
    agent.complete_goal(&ns, goal_id)
}

#[tauri::command]
pub fn memory_record_failure(
    namespace: Option<String>,
    what: String,
    why: String,
    lesson: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    let ns = namespace.unwrap_or_else(|| "nakama:failures".to_string());
    agent.record_failure(&ns, &what, &why, &lesson)
}

#[tauri::command]
pub fn memory_set_context(
    task: String,
    active_goals: Vec<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    agent.set_working_context(&task, &active_goals)
}

#[tauri::command]
pub fn memory_summary(
    app_state: State<'_, AppState>,
) -> Result<MemorySummary, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    client.get_summary()
}

#[tauri::command]
pub fn memory_consolidate(
    namespace: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    match namespace {
        Some(ns) => agent.consolidate_ns(&ns),
        None => {
            for ns in &["nakama:conversations", "nakama:actions", "nakama:system"] {
                let _ = agent.consolidate_ns(ns);
            }
            Ok(())
        }
    }
}

#[tauri::command]
pub fn memory_build_prompt(
    user_prompt: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    let agent = MemoryAgent::new(client);
    agent.build_memory_prompt(&user_prompt)
}

#[tauri::command]
pub fn autonomy_status(
    app_state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = open_local_memory(&app_state.memory_db)?;
    match client.get_summary() {
        Ok(summary) => Ok(serde_json::json!({
            "connected": true,
            "total_episodes": summary.total_episodes,
            "total_facts": summary.total_facts,
            "total_goals": summary.total_goals,
            "namespaces": summary.namespaces,
        })),
        Err(e) => Ok(serde_json::json!({
            "connected": false,
            "error": e,
        })),
    }
}
