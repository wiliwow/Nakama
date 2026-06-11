use crate::conversation::{Conversation, ConversationMessage};
use crate::AppState;
use chrono::Utc;
use tauri::State;
use tracing::error;

#[tauri::command]
pub async fn conversation_create(
    title: String,
    app_state: State<'_, AppState>,
) -> Result<Conversation, String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    store
        .create_conversation(title)
        .map_err(|e| {
            error!(error = %e, "conversation_create: database error");
            format!("Failed to create conversation: {}", e)
        })
}

#[tauri::command]
pub async fn conversation_load(
    id: i64,
    app_state: State<'_, AppState>,
) -> Result<Option<Conversation>, String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    store
        .load_conversation(id)
        .map_err(|e| {
            error!(error = %e, conversation_id = id, "conversation_load: database error");
            format!("Failed to load conversation: {}", e)
        })
}

#[tauri::command]
pub async fn conversation_save(
    conversation: Conversation,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    store
        .save_conversation(&conversation)
        .map_err(|e| {
            error!(error = %e, conversation_id = ?conversation.id, "conversation_save: database error");
            format!("Failed to save conversation: {}", e)
        })
}

#[tauri::command]
pub async fn conversation_list(
    app_state: State<'_, AppState>,
) -> Result<Vec<Conversation>, String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    store
        .list_conversations()
        .map_err(|e| {
            error!(error = %e, "conversation_list: database error");
            format!("Failed to list conversations: {}", e)
        })
}

#[tauri::command]
pub async fn conversation_delete(
    id: i64,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    store
        .delete_conversation(id)
        .map_err(|e| {
            error!(error = %e, conversation_id = id, "conversation_delete: database error");
            format!("Failed to delete conversation: {}", e)
        })
}

#[tauri::command]
pub async fn conversation_add_message(
    conversation_id: i64,
    role: String,
    content: String,
    app_state: State<'_, AppState>,
) -> Result<i64, String> {
    let store = app_state.inner().conversation_store.lock().unwrap();
    let message = ConversationMessage {
        id: None,
        role,
        content,
        timestamp: Utc::now(),
        metadata: None,
    };

    store
        .add_message(conversation_id, message)
        .map_err(|e| {
            error!(error = %e, conversation_id, "conversation_add_message: database error");
            format!("Failed to add message: {}", e)
        })
}