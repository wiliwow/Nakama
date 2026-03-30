mod ai_companion;
mod command;
mod commands;
mod config;
mod conversation;
mod input_controller;
mod rag_indexer;
mod recorder;

use tauri::Manager;
use commands::fetch_files::fetch_files;
use ai_companion::AICompanion;
use command::get_message;
use commands::ai::{ask_ai, ask_ai_stream, ask_ai_with_conversation, ask_ai_stream_with_conversation};
use commands::mouse::{mouse_click, mouse_drag, mouse_move, mouse_down, mouse_up, mouse_scroll, press_key, type_text};
use commands::screen::{capture_all_screens, capture_primary_screen};
use commands::recording::{
    get_screen_recording_status, get_voice_recording_status, start_screen_recording,
    start_voice_recording, stop_screen_recording, stop_voice_recording,
};
#[cfg(feature = "swiftide_integration")]
use commands::rag::{rag_index, rag_retrieve, rag_add_file, rag_health_check, rag_clear_index, rag_index_stats};
use commands::conversation::{conversation_create, conversation_load, conversation_save, conversation_list, conversation_delete, conversation_add_message};
use input_controller::InputController;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use rag_indexer::IndexMetadataStore;
#[cfg(feature = "swiftide_integration")]
use config::ConfigManager;
use conversation::ConversationStore;

struct AppState {
    #[allow(dead_code)]
    input: Mutex<InputController>,
    #[cfg(feature = "swiftide_integration")]
    #[allow(dead_code)]
    metadata_store: Arc<TokioMutex<IndexMetadataStore>>,
    #[cfg(feature = "swiftide_integration")]
    #[allow(dead_code)]
    config_manager: Arc<Mutex<ConfigManager>>,
    #[allow(dead_code)]
    conversation_store: Arc<Mutex<ConversationStore>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // In release builds start recording and voice listening. Skip these heavy
    // background tasks during development to avoid interfering with the dev
    // workflow (FFmpeg or audio devices can cause crashes or exit the process).
    #[cfg(not(debug_assertions))]
    {
        if let Err(e) = recorder::start_screen_recording() {
            eprintln!("Screen recording not started: {}", e);
        }
    }

    // Initialize AI companion using Ollama model `deepseek:latest` by default
    let ai_companion = Arc::new(TokioMutex::new(AICompanion::new(
        "deepseek-r1:1.5b".to_string(),
    )));

    // Initialize metadata store for RAG indexing
    #[cfg(feature = "swiftide_integration")]
    let metadata_store = {
        let config_dir = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".nakama_config"))
            .unwrap_or_else(|_| std::path::PathBuf::from(".nakama_config"));

        Arc::new(TokioMutex::new(
            IndexMetadataStore::new_sync(config_dir)
                .expect("Failed to initialize metadata store")
        ))
    };

    // Initialize config manager
    #[cfg(feature = "swiftide_integration")]
    let config_manager = {
        let config_dir = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".nakama_config"))
            .unwrap_or_else(|_| std::path::PathBuf::from(".nakama_config"));

        Arc::new(Mutex::new(ConfigManager::new(config_dir)))
    };

    // Initialize conversation store
    let conversation_store = {
        let config_dir = std::env::var("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".nakama_config"))
            .unwrap_or_else(|_| std::path::PathBuf::from(".nakama_config"));

        Arc::new(Mutex::new(
            ConversationStore::new(config_dir)
                .expect("Failed to initialize conversation store")
        ))
    };

    // build the tauri application but do not run it yet - running consumes the builder
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            input: Mutex::new(InputController::new()),
            #[cfg(feature = "swiftide_integration")]
            metadata_store,
            #[cfg(feature = "swiftide_integration")]
            config_manager: config_manager.clone(),
            conversation_store,
        })
        .manage(ai_companion);

    #[cfg(feature = "swiftide_integration")]
    {
        builder = builder.manage(config_manager.clone());
    }

    builder = builder.invoke_handler(tauri::generate_handler![
            mouse_move,
            mouse_click,
            type_text,
            press_key,
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
            mouse_move,
            mouse_click,
            mouse_drag,
            mouse_down,
            mouse_up,
            mouse_scroll,
            press_key,
            type_text,
            fetch_files,
            conversation_create,
            conversation_load,
            conversation_save,
            conversation_list,
            conversation_delete,
            conversation_add_message,
            #[cfg(feature = "swiftide_integration")]
            rag_index,
            #[cfg(feature = "swiftide_integration")]
            rag_retrieve,
            #[cfg(feature = "swiftide_integration")]
            rag_add_file,
            #[cfg(feature = "swiftide_integration")]
            rag_health_check,
            #[cfg(feature = "swiftide_integration")]
            rag_clear_index,
            #[cfg(feature = "swiftide_integration")]
            rag_index_stats,
        ]);

    // In development builds, open the webview devtools automatically to inspect UI errors.
    #[cfg(debug_assertions)]
    {
        builder = builder.setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.open_devtools();
            }
            Ok(())
        });
    }

    // now run the application consuming the builder
    builder
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
