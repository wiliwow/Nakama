use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::recorder::{self, get_last_recording_info, stop_screen_recording_signal};

// Global state for recording control
pub static SCREEN_RECORDING: AtomicBool = AtomicBool::new(false);
pub static VOICE_RECORDING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
struct RecordingSummary {
    frames_dir: String,
    keylog_path: String,
    duration_seconds: u64,
}

#[tauri::command]
pub fn start_screen_recording() -> Result<String, String> {
    if SCREEN_RECORDING.load(Ordering::SeqCst) {
        return Err("Screen recording already running".into());
    }
    // start background recorder
    if let Err(e) = recorder::start_screen_recording() {
        return Err(e);
    }
    SCREEN_RECORDING.store(true, Ordering::SeqCst);
    eprintln!("Screen recording started");
    Ok("Screen recording started".into())
}

#[tauri::command]
pub fn stop_screen_recording() -> Result<String, String> {
    if !SCREEN_RECORDING.load(Ordering::SeqCst) {
        return Err("Screen recording not running".into());
    }
    // signal recorder to stop
    stop_screen_recording_signal();
    SCREEN_RECORDING.store(false, Ordering::SeqCst);
    // read last recording info
    if let Some(info) = get_last_recording_info() {
        let summary = RecordingSummary {
            frames_dir: info.frames_dir,
            keylog_path: info.keylog_path,
            duration_seconds: info.duration_seconds,
        };
        let json = serde_json::to_string(&summary).map_err(|e| e.to_string())?;
        return Ok(json);
    }
    Ok("Screen recording stopped".into())
}

#[tauri::command]
pub fn get_screen_recording_status() -> bool {
    SCREEN_RECORDING.load(Ordering::SeqCst)
}

#[tauri::command]
pub fn start_voice_recording() -> Result<String, String> {
    if VOICE_RECORDING.load(Ordering::SeqCst) {
        return Err("Voice recording already running".into());
    }
    VOICE_RECORDING.store(true, Ordering::SeqCst);
    eprintln!("Voice recording started");
    Ok("Voice recording started".into())
}

#[tauri::command]
pub fn stop_voice_recording() -> Result<String, String> {
    if !VOICE_RECORDING.load(Ordering::SeqCst) {
        return Err("Voice recording not running".into());
    }
    VOICE_RECORDING.store(false, Ordering::SeqCst);
    eprintln!("Voice recording stopped");
    Ok("Voice recording stopped".into())
}

#[tauri::command]
pub fn get_voice_recording_status() -> bool {
    VOICE_RECORDING.load(Ordering::SeqCst)
}
