mod ai_companion;
mod command;
mod commands;
mod config;
mod conversation;
mod input_controller;
mod local_memory;
mod memory_agent;
mod memory;
mod models;
mod rag_indexer;
mod recorder;

use std::fs;
use tauri::Manager;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use commands::fetch_files::fetch_files;
use ai_companion::AICompanion;
use command::get_message;
use commands::ai::{ask_ai, ask_ai_stream, ask_ai_with_conversation, ask_ai_stream_with_conversation};
use commands::mouse::{mouse_click, mouse_drag, mouse_move, mouse_down, mouse_up, mouse_scroll, press_key, type_text};
use commands::screen::{capture_all_screens, capture_primary_screen, get_display_info};
use commands::recording::{
    get_screen_recording_status, get_voice_recording_status, start_screen_recording,
    start_voice_recording, stop_screen_recording, stop_voice_recording,
};
use commands::memory::{
    memory_init, memory_remember, memory_recall, memory_query_facts,
    memory_list_goals, memory_create_goal, memory_complete_goal,
    memory_record_failure, memory_set_context, memory_summary,
    memory_consolidate, memory_build_prompt, autonomy_status,
};
use commands::rag::{rag_index, rag_retrieve, rag_add_file, rag_health_check, rag_clear_index, rag_index_stats};
use commands::conversation::{conversation_create, conversation_load, conversation_save, conversation_list, conversation_delete, conversation_add_message};
use commands::execute_ai_with_actions::execute_ai_with_actions;
use commands::computer_use::{execute_computer_tool, execute_text_editor_tool, execute_bash_tool};
use input_controller::InputController;
use rag_indexer::RAGIndex;
use config::ConfigManager;
use conversation::ConversationStore;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;

pub struct AppState {
    #[allow(dead_code)]
    input: Mutex<InputController>,
    #[allow(dead_code)]
    metadata_store: Arc<TokioMutex<RAGIndex>>,
    #[allow(dead_code)]
    config_manager: Arc<Mutex<ConfigManager>>,
    #[allow(dead_code)]
    memory_db: std::path::PathBuf,
    #[allow(dead_code)]
    conversation_store: Arc<Mutex<ConversationStore>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = dotenv::dotenv();
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let log_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(|h| std::path::PathBuf::from(h).join(".nakama_logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".nakama_logs"));

    let file_appender = tracing_appender::rolling::daily(&log_dir, "nakama.log");

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .json()
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .with_current_span(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .pretty()
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "Nakama logger initialised");

    #[cfg(not(debug_assertions))]
    {
        if let Err(e) = recorder::start_screen_recording() {
            eprintln!("Screen recording not started: {}", e);
        }
    }

    let ai_companion = Arc::new(TokioMutex::new(AICompanion::new(
        "".to_string(),
    )));

    let config_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(|h| std::path::PathBuf::from(h).join(".nakama_config"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".nakama_config"));

    let metadata_store = {
        let _ = fs::create_dir_all(&config_dir);
        Arc::new(TokioMutex::new(
            RAGIndex::new(config_dir.join("rag_index"))
                .expect("Failed to initialize RAG index")
        ))
    };

    let config_manager = Arc::new(Mutex::new(ConfigManager::new(config_dir.clone())));

    let memory_db = config_dir.join("memory.db");
    let _ = fs::create_dir_all(memory_db.parent().unwrap_or(&config_dir));

    let conversation_store = {
        let store = ConversationStore::new(config_dir)
            .expect("Failed to initialize conversation store");
        Arc::new(Mutex::new(store))
    };

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_mic_recorder::init())
        .manage(AppState {
            input: Mutex::new(InputController::new()),
            metadata_store,
            config_manager: config_manager.clone(),
            memory_db,
            conversation_store,
        })
        .manage(ai_companion);

    builder = builder.invoke_handler(tauri::generate_handler![
            mouse_move,
            mouse_click,
            mouse_drag,
            mouse_down,
            mouse_up,
            mouse_scroll,
            press_key,
            type_text,
            get_message,
            start_screen_recording,
            stop_screen_recording,
            get_screen_recording_status,
            start_voice_recording,
            stop_voice_recording,
            get_voice_recording_status,
            ask_ai,
            ask_ai_stream,
            ask_ai_with_conversation,
            ask_ai_stream_with_conversation,
            capture_primary_screen,
            capture_all_screens,
            get_display_info,
            fetch_files,
            conversation_create,
            conversation_load,
            conversation_save,
            conversation_list,
            conversation_delete,
            conversation_add_message,
            rag_index,
            rag_retrieve,
            rag_add_file,
            rag_health_check,
            rag_clear_index,
            rag_index_stats,
            execute_ai_with_actions,
            execute_computer_tool,
            execute_text_editor_tool,
            execute_bash_tool,
            memory_init,
            memory_remember,
            memory_recall,
            memory_query_facts,
            memory_list_goals,
            memory_create_goal,
            memory_complete_goal,
            memory_record_failure,
            memory_set_context,
            memory_summary,
            memory_consolidate,
            memory_build_prompt,
            autonomy_status,
        ]);

    #[cfg(debug_assertions)]
    {
        builder = builder.setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.open_devtools();
            }
            Ok(())
        });
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
