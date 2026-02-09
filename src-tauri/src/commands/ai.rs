use tauri::Emitter;
use crate::ai_companion::AICompanion;
use std::sync::Arc;
use tauri::{State, Window};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

#[tauri::command]
pub async fn ask_ai(
    prompt: String,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<String, String> {
    let ai = ai_state.lock().await;
    ai.process_request(prompt).await
}

/// Start a streaming AI response. Emits the following window events:
/// - `ai-stream-chunk` with payload `String` for partial chunks
/// - `ai-stream-error` with payload `String` on error
/// - `ai-stream-done` when the stream completes
#[tauri::command]
pub async fn ask_ai_stream(
    prompt: String,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<(), String> {
    // grab the model name while holding the lock briefly
    let model = {
        let ai = ai_state.lock().await;
        ai.model_clone()
    };

    // spawn a task to stream without blocking the command caller
    tokio::spawn(async move {
        // create a fresh Ollama client for this task
        let client = ollama_rs::Ollama::default();
        let request = ollama_rs::generation::completion::request::GenerationRequest::new(model, prompt);

        match client.generate_stream(request).await {
            Ok(mut stream) => {
                while let Some(chunk_res) = stream.next().await {
                    match chunk_res {
                        Ok(chunks) => {
                            for piece in chunks {
                                let _ = window.emit("ai-stream-chunk", piece.response.clone());
                            }
                        }
                        Err(e) => {
                            let _ = window.emit("ai-stream-error", e.to_string());
                            let _ = window.emit("ai-stream-done", ());
                            return;
                        }
                    }
                }
                let _ = window.emit("ai-stream-done", ());
            }
            Err(e) => {
                let _ = window.emit("ai-stream-error", e.to_string());
                let _ = window.emit("ai-stream-done", ());
            }
        }
    });

    Ok(())
}
