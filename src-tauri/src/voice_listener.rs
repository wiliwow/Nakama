use alsa::pcm::{Access, Format, HwParams, PCM};
use bytemuck;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static LISTENING: AtomicBool = AtomicBool::new(false);


pub fn start_voice_listening() -> Result<(), String> {
    if LISTENING.load(Ordering::SeqCst) {
        return Err("Voice listening is already running.".to_string());
    }

    LISTENING.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        // Open the PCM device for capture
        let pcm = PCM::new("default", alsa::Direction::Capture, false)
            .expect("Failed to open PCM device");

        // Set hardware parameters
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_access(Access::RWInterleaved)
            .expect("Failed to set access");
        hwp.set_format(Format::S16LE)
            .expect("Failed to set format");
        hwp.set_rate(44100, alsa::ValueOr::Nearest)
            .expect("Failed to set rate");
        hwp.set_channels(1)
            .expect("Failed to set channels");
        pcm.hw_params(&hwp).expect("Failed to set hardware parameters");

        // Prepare the PCM device
        pcm.prepare().expect("Failed to prepare PCM device");

        // Open a file to save the audio data
        let mut file = File::create("../recordings/audio_capture.raw")
            .expect("Failed to create audio file");

        // Buffer to store audio data
        let mut buffer = [0i16; 1024];

        // Start capturing audio
        let io = pcm.io_i16().unwrap();
        while LISTENING.load(Ordering::SeqCst) {
            match io.readi(&mut buffer) {
                Ok(size) => {
                    // Write captured audio to the file
                    file.write_all(bytemuck::cast_slice(&buffer[..size]))
                        .expect("Failed to write audio data");
                }
                Err(err) => eprintln!("Error capturing audio: {:?}", err),
            }
        }

        println!("Voice listening stopped.");
    });

    Ok(())
}