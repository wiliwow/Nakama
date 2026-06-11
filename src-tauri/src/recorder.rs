use chrono::Local;
use crossbeam_channel::tick;
use rdev::{listen, Event, EventType};
use screenshots::Screen;
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

fn key_logger(event: Event, file_path: &Path) {
    if let EventType::KeyPress(key) = event.event_type {
        let timestamp = Local::now();
        let mut file = OpenOptions::new().create(true).append(true).open(file_path)
            .expect("Failed to open keylog file");
        writeln!(file, "Key {:?} pressed at {}", key, timestamp).expect("Failed to write keylog");
    }
}

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone)]
pub struct RecordingInfo {
    pub frames_dir: String,
    pub keylog_path: String,
    pub duration_seconds: u64,
}

use once_cell::sync::Lazy;
use std::sync::Mutex as StdMutex;

static LAST_RECORDING: Lazy<StdMutex<Option<RecordingInfo>>> = Lazy::new(|| StdMutex::new(None));

pub fn start_screen_recording() -> Result<(), String> {
    RUNNING.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        let recordings_path = Path::new("../recordings/frames");
        if !recordings_path.exists() {
            let _ = fs::create_dir_all(recordings_path);
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let frames_dir = recordings_path.join(format!("rec_{}", timestamp));
        let _ = fs::create_dir_all(&frames_dir);

        let keylog_path = Path::new("../recordings").join(format!("output_{}.txt", timestamp));

        let keylog_path_clone = keylog_path.clone();
        thread::spawn(move || {
            listen(move |event: Event| {
                key_logger(event, &keylog_path_clone);
            }).expect("Key listener error");
        });

        let screens = Screen::all().map_err(|e| e.to_string()).unwrap();
        let primary = screens.first().ok_or("No screens available").unwrap();

        let start_time = Instant::now();
        let mut frame_idx: u64 = 0;
        let ticker = tick(Duration::from_millis(500));

        while RUNNING.load(Ordering::SeqCst) {
            let _ = ticker.recv();
            if let Ok(img) = primary.capture() {
                let frame_path = frames_dir.join(format!("frame_{:06}.png", frame_idx));
                if let Err(e) = img.save(&frame_path) {
                    eprintln!("Failed to save frame: {}", e);
                }
                frame_idx += 1;
            }
        }

        let duration = start_time.elapsed().as_secs();
        let info = RecordingInfo {
            frames_dir: frames_dir.to_string_lossy().into_owned(),
            keylog_path: keylog_path.to_string_lossy().into_owned(),
            duration_seconds: duration,
        };
        if let Ok(mut lock) = LAST_RECORDING.lock() {
            *lock = Some(info);
        }
    });

    Ok(())
}

pub fn stop_screen_recording_signal() {
    RUNNING.store(false, Ordering::SeqCst);
}

pub fn get_last_recording_info() -> Option<RecordingInfo> {
    LAST_RECORDING.lock().ok().and_then(|lock| lock.clone())
}
