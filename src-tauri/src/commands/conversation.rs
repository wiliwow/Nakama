use crate::conversation::{Conversation, ConversationMessage, ConversationStore};
use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn conversation_create(
    title: String,
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<Conversation, String> {
    let store = store.lock().await;
    store
        .create_conversation(title)
        .map_err(|e| format!("Failed to create conversation: {}", e))
}

#[tauri::command]
pub async fn conversation_load(
    id: i64,
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<Option<Conversation>, String> {
    let store = store.lock().await;
    store
        .load_conversation(id)
        .map_err(|e| format!("Failed to load conversation: {}", e))
}

#[tauri::command]
pub async fn conversation_save(
    conversation: Conversation,
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<(), String> {
    let store = store.lock().await;
    store
        .save_conversation(&conversation)
        .map_err(|e| format!("Failed to save conversation: {}", e))
}

#[tauri::command]
pub async fn conversation_list(
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<Vec<Conversation>, String> {
    let store = store.lock().await;
    store
        .list_conversations()
        .map_err(|e| format!("Failed to list conversations: {}", e))
}

#[tauri::command]
pub async fn conversation_delete(
    id: i64,
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<(), String> {
    let store = store.lock().await;
    store
        .delete_conversation(id)
        .map_err(|e| format!("Failed to delete conversation: {}", e))
}

#[tauri::command]
pub async fn conversation_add_message(
    conversation_id: i64,
    role: String,
    content: String,
    store: State<'_, Arc<Mutex<ConversationStore>>>,
) -> Result<i64, String> {
    let store = store.lock().await;
    let message = ConversationMessage {
        id: None,
        role,
        content,
        timestamp: Utc::now(),
        metadata: None,
    };

    store
        .add_message(conversation_id, message)
        .map_err(|e| format!("Failed to add message: {}", e))
}