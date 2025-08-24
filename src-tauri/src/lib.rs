mod commands;
mod input_controller;
mod recorder;
mod voice_listener;

use commands::mouse::{mouse_move, mouse_click, type_text, press_key};
use input_controller::InputController;
use std::sync::Mutex;

struct AppState {
    input: Mutex<InputController>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Start the screen recording and voice listening in separate threads
    recorder::start_screen_recording().expect("Failed to start screen recording");
    voice_listener::start_voice_listening().expect("Failed to start voice listening");

    tauri::Builder::default()
        .manage(AppState {
            input: Mutex::new(InputController::new()),
        })
        .invoke_handler(tauri::generate_handler![
            mouse_move,
            mouse_click,
            type_text,
            press_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
