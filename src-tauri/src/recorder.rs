use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use chrono::Local;
use crossbeam_channel::tick;
use rdev::{listen, Event, EventType};
use screenshots::{image::EncodableLayout, Screen};
use serde::Serialize;
fn key_logger(event: Event, file_path: &Path) {
    if let EventType::KeyPress(key) = event.event_type {
        let timestamp = Local::now(); // Get the current date and time

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .expect("Failed to open file");

        writeln!(file, "Key {:?} pressed at {}", key, timestamp)
            .expect("Failed to write to file");
    }
}

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone)]
pub struct RecordingInfo {
    pub video_path: String,
    pub keylog_path: String,
    pub duration_seconds: u64,
}

use once_cell::sync::Lazy;
use std::sync::Mutex as StdMutex;

static LAST_RECORDING: Lazy<StdMutex<Option<RecordingInfo>>> = Lazy::new(|| StdMutex::new(None));

pub fn start_screen_recording() -> Result<(), String> {
    // Check that `ffmpeg` is available before spawning the recording thread.
    match Command::new("ffmpeg").arg("-version").stdout(Stdio::null()).stderr(Stdio::null()).status() {
        Ok(status) if status.success() => {}
        Ok(_) => return Err("ffmpeg is present but returned non-zero when queried".into()),
        Err(_) => return Err("ffmpeg not found in PATH. Install ffmpeg and ensure it's available in PATH".into()),
    }

    // Set running flag
    RUNNING.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        // Ensure recordings directory exists in project root
        let recordings_path = Path::new("../recordings");
        if !recordings_path.exists() {
            fs::create_dir_all(recordings_path).expect("Failed to create recordings directory");
        }

        // Generate a unique base name for the recording
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let video_file_path = recordings_path.join(format!("output_{}.mp4", timestamp));
        let log_file_path = recordings_path.join(format!("output_{}.txt", timestamp));

        // Start the keylogger in a separate thread
        let keylog_path_clone = log_file_path.clone();
        thread::spawn(move || {
            listen(move |event: Event| {
                key_logger(event, &keylog_path_clone);
            })
            .expect("Error listening for key events");
        });

        // Screen recording logic
        let screens = Screen::all().map_err(|e| e.to_string()).unwrap();
        let primary = screens.first().ok_or("No screens available").unwrap();
        let (width, height) = (primary.display_info.width, primary.display_info.height);

        // Start FFmpeg process capturing video only (no audio for now)
        // Video-only approach avoids ALSA audio issues
        let mut ffmpeg = match Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f", "rawvideo",
                "-pix_fmt", "bgra",
                "-s", &format!("{}x{}", width, height),
                "-framerate", "20",
                "-i", "-",
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", "23",
                "-pix_fmt", "yuv420p",
                video_file_path.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(p) => p,
            Err(err) => {
                eprintln!("Failed to start ffmpeg: {}", err);
                return;
            }
        };

        let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");
        let ticker = tick(Duration::from_millis(50)); // ~20 FPS

        // Timer for recording duration
        let start_time = Instant::now();
        let mut last_print = Instant::now();

        // Capture loop
        while RUNNING.load(Ordering::SeqCst) {
            ticker.recv().unwrap(); // Wait for next tick
            let image = match primary.capture() {
                Ok(img) => img,
                Err(err) => {
                    eprintln!("Failed to capture screen: {}", err);
                    break;
                }
            };

            if let Err(err) = stdin.write_all(image.as_bytes()) {
                eprintln!("Failed to write frame: {}", err);
                break;
            }
            if last_print.elapsed().as_secs() >= 1 {
                let elapsed = start_time.elapsed().as_secs();
                println!("Recording duration: {} seconds", elapsed);
                last_print = Instant::now();
            }
        }

        drop(stdin);
        let _ = ffmpeg.wait();

        // Store last recording info
        let duration = start_time.elapsed().as_secs();
        let info = RecordingInfo {
            video_path: video_file_path.to_string_lossy().into_owned(),
            keylog_path: log_file_path.to_string_lossy().into_owned(),
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
    if let Ok(lock) = LAST_RECORDING.lock() {
        return lock.clone();
    }
    None
}