use std::{
    fs::{self, OpenOptions, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use alsa::pcm::{Access, Format, HwParams, PCM};
use chrono::Local;
use crossbeam_channel::tick;
use rdev::{listen, Event, EventType, Key};
use screenshots::{image::EncodableLayout, Screen};
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

static RUNNING: AtomicBool = AtomicBool::new(true);

fn start_hotkey_listener() {
    let ctrl = Arc::new(Mutex::new(false));
    let alt = Arc::new(Mutex::new(false));
    let shift = Arc::new(Mutex::new(false));

    let ctrl_clone = Arc::clone(&ctrl);
    let alt_clone = Arc::clone(&alt);
    let shift_clone = Arc::clone(&shift);

    thread::spawn(move || {
        listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) => *ctrl_clone.lock().unwrap() = true,
                EventType::KeyRelease(Key::ControlLeft) | EventType::KeyRelease(Key::ControlRight) => *ctrl_clone.lock().unwrap() = false,
                EventType::KeyPress(Key::Alt) => *alt_clone.lock().unwrap() = true,
                EventType::KeyRelease(Key::Alt) => *alt_clone.lock().unwrap() = false,
                EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => *shift_clone.lock().unwrap() = true,
                EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => *shift_clone.lock().unwrap() = false,
                EventType::KeyPress(Key::KeyC) => {
                    if *ctrl_clone.lock().unwrap() && *alt_clone.lock().unwrap() && *shift_clone.lock().unwrap() {
                        RUNNING.store(false, Ordering::SeqCst);
                    }
                }
                _ => {}
            }
        }).expect("Error listening for hotkey");
    });
}


pub fn start_screen_recording() -> Result<(), String> {
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
        thread::spawn(move || {
            listen(move |event: Event| {
                if let EventType::KeyPress(key) = event.event_type {
                    let timestamp = Local::now(); // Get the current date and time

                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_file_path)
                        .expect("Failed to open key log file");

                    writeln!(file, "Key {:?} pressed at {}", key, timestamp)
                        .expect("Failed to write to key log file");
                }
            })
            .expect("Error listening for key events");
        });

        // Screen recording logic
        let screens = Screen::all().map_err(|e| e.to_string()).unwrap();
        let primary = screens.first().ok_or("No screens available").unwrap();
        let (width, height) = (primary.display_info.width, primary.display_info.height);

        // Start FFmpeg process
        let mut ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f", "rawvideo",
                "-pix_fmt", "bgra",
                "-s", &format!("{}x{}", width / 2, height / 2), // Reduce resolution by half
                "-i", "-",
                "-r", "20", // Reduce frame rate to 20 FPS
                "-c:v", "libx264",
                "-preset", "ultrafast", // Use ultrafast preset for better performance
                "-crf", "30", // Increase CRF (higher value = lower quality)
                "-b:v", "500k", // Set bitrate to 500kbps
                "-f", "segment",
                "-segment_time", "300",
                "-reset_timestamps", "1",
                video_file_path.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg");

        let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");
        let ticker = tick(Duration::from_millis(33));

        // Timer for recording duration
        let start_time = Instant::now();
        let mut last_print = Instant::now();

        // Capture loop
        while RUNNING.load(Ordering::SeqCst) {
            ticker.recv().unwrap(); // Wait for next tick
            let image = primary.capture().expect("Failed to capture screen");
            stdin.write_all(image.as_bytes()).expect("Failed to write frame");
            if last_print.elapsed().as_secs() >= 1 {
                let elapsed = start_time.elapsed().as_secs();
                println!("Recording duration: {} seconds", elapsed);
                last_print = Instant::now();
            }
        }

        drop(stdin);
        ffmpeg.wait().expect("FFmpeg process failed");
    });

    Ok(())
}