mod ai_companion;
mod command;
mod commands;
mod input_controller;
mod recorder;

use tauri::Manager;

use ai_companion::AICompanion;
use command::get_message;
use commands::ai::{ask_ai, ask_ai_stream};
use commands::mouse::{mouse_click, mouse_move, press_key, type_text};
use commands::recording::{
    get_screen_recording_status, get_voice_recording_status, start_screen_recording,
    start_voice_recording, stop_screen_recording, stop_voice_recording,
};
use input_controller::InputController;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;

struct AppState {
    input: Mutex<InputController>,
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

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            input: Mutex::new(InputController::new()),
        })
        .manage(ai_companion)
        .invoke_handler(tauri::generate_handler![
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

    builder
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
