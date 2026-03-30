use crate::ai_companion::AICompanion;
use std::sync::Arc;
use tauri::Emitter;
use tauri::{State, Window};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use serde::{Serialize, Deserialize};
use crate::conversation::ConversationMessage;

#[cfg(feature = "swiftide_integration")]
use crate::commands::rag::{rag_retrieve, RetrievedPassage};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiResponse {
    pub response: String,
    pub rag_used: bool,
    pub passages_retrieved: usize,
    #[cfg(feature = "swiftide_integration")]
    pub passages: Vec<RetrievedPassage>,
    #[cfg(not(feature = "swiftide_integration"))]
    pub passages: Vec<()>,
}

#[cfg(feature = "swiftide_integration")]
async fn build_prompt_with_context(
    user_prompt: String,
    config_manager: tauri::State<'_, std::sync::Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> (String, bool, usize, Vec<RetrievedPassage>) {
    match rag_retrieve(user_prompt.clone(), Some(5), config_manager).await {
        Ok(passages) if !passages.is_empty() => {
            let context = passages
                .iter()
                .map(|p| format!("Source: {}\nContent: {}", p.source.clone().unwrap_or_else(|| "Unknown".to_string()), p.content.clone()))
                .collect::<Vec<_>>()
                .join("\n\n");
            let enhanced_prompt = format!("Context from indexed files:\n{}\n\nUser question: {}", context, user_prompt);
            (enhanced_prompt, true, passages.len(), passages)
        }
        _ => (user_prompt, false, 0, vec![]),
    }
}

#[cfg(not(feature = "swiftide_integration"))]
async fn build_prompt_with_context(
    user_prompt: String,
) -> (String, bool, usize, Vec<()>) {
    (user_prompt, false, 0, vec![])
}

#[cfg(feature = "swiftide_integration")]
#[tauri::command]
pub async fn ask_ai(
    prompt: String,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    config_manager: State<'_, Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<AiResponse, String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt, config_manager).await;
    let ai = ai_state.lock().await;
    let response = ai.process_request(enhanced_prompt).await?;

    Ok(AiResponse {
        response,
        rag_used,
        passages_retrieved,
        passages,
    })
}

#[cfg(not(feature = "swiftide_integration"))]
#[tauri::command]
pub async fn ask_ai(
    prompt: String,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<AiResponse, String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt).await;
    let ai = ai_state.lock().await;
    let response = ai.process_request(enhanced_prompt).await?;

    Ok(AiResponse {
        response,
        rag_used,
        passages_retrieved,
        passages,
    })
}

#[cfg(feature = "swiftide_integration")]
#[tauri::command]
pub async fn ask_ai_stream(
    prompt: String,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    config_manager: State<'_, Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<(), String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt, config_manager).await;

    // Emit metadata about RAG retrieval
    let metadata = serde_json::json!({
        "rag_used": rag_used,
        "passages_retrieved": passages_retrieved,
        "passages": passages
    });
    let _ = window.emit("ai-stream-metadata", metadata);

    // grab the model name while holding the lock briefly
    let model = {
        let ai = ai_state.lock().await;
        ai.model_clone()
    };

    // spawn a task to stream without blocking the command caller
    tokio::spawn(async move {
        // create a fresh Ollama client for this task
        let client = ollama_rs::Ollama::default();
        let request =
            ollama_rs::generation::completion::request::GenerationRequest::new(model, enhanced_prompt);

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

#[cfg(not(feature = "swiftide_integration"))]
#[tauri::command]
pub async fn ask_ai_stream(
    prompt: String,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<(), String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt).await;

    // Emit metadata about RAG retrieval
    let metadata = serde_json::json!({
        "rag_used": rag_used,
        "passages_retrieved": passages_retrieved,
        "passages": passages
    });
    let _ = window.emit("ai-stream-metadata", metadata);

    // grab the model name while holding the lock briefly
    let model = {
        let ai = ai_state.lock().await;
        ai.model_clone()
    };

    // spawn a task to stream without blocking the command caller
    tokio::spawn(async move {
        // create a fresh Ollama client for this task
        let client = ollama_rs::Ollama::default();
        let request =
            ollama_rs::generation::completion::request::GenerationRequest::new(model, enhanced_prompt);

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

#[cfg(feature = "swiftide_integration")]
#[tauri::command]
pub async fn ask_ai_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    config_manager: State<'_, Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<AiResponse, String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt.clone(), config_manager).await;
    let ai = ai_state.lock().await;
    let response = ai.process_request_with_context(enhanced_prompt, &conversation_history).await?;

    Ok(AiResponse {
        response,
        rag_used,
        passages_retrieved,
        passages,
    })
}

#[cfg(not(feature = "swiftide_integration"))]
#[tauri::command]
pub async fn ask_ai_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<AiResponse, String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt.clone()).await;
    let ai = ai_state.lock().await;
    let response = ai.process_request_with_context(enhanced_prompt, &conversation_history).await?;

    Ok(AiResponse {
        response,
        rag_used,
        passages_retrieved,
        passages,
    })
}

#[cfg(feature = "swiftide_integration")]
#[tauri::command]
pub async fn ask_ai_stream_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    config_manager: State<'_, Arc<std::sync::Mutex<crate::config::ConfigManager>>>,
) -> Result<(), String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt.clone(), config_manager).await;

    // Emit metadata about RAG retrieval
    let metadata = serde_json::json!({
        "rag_used": rag_used,
        "passages_retrieved": passages_retrieved,
        "passages": passages
    });
    let _ = window.emit("ai-stream-metadata", metadata);

    let ai = ai_state.lock().await;
    let full_prompt = ai.build_prompt_with_context(enhanced_prompt, &conversation_history);

    // grab the model name while holding the lock briefly
    let model = ai.model_clone();

    // spawn a task to stream without blocking the command caller
    tokio::spawn(async move {
        // create a fresh Ollama client for this task
        let client = ollama_rs::Ollama::default();
        let request =
            ollama_rs::generation::completion::request::GenerationRequest::new(model, full_prompt);

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

#[cfg(not(feature = "swiftide_integration"))]
#[tauri::command]
pub async fn ask_ai_stream_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<(), String> {
    let (enhanced_prompt, rag_used, passages_retrieved, passages) = build_prompt_with_context(prompt.clone()).await;

    // Emit metadata about RAG retrieval
    let metadata = serde_json::json!({
        "rag_used": rag_used,
        "passages_retrieved": passages_retrieved,
        "passages": passages
    });
    let _ = window.emit("ai-stream-metadata", metadata);

    let ai = ai_state.lock().await;
    let full_prompt = ai.build_prompt_with_context(enhanced_prompt, &conversation_history);

    // grab the model name while holding the lock briefly
    let model = ai.model_clone();

    // spawn a task to stream without blocking the command caller
    tokio::spawn(async move {
        // create a fresh Ollama client for this task
        let client = ollama_rs::Ollama::default();
        let request =
            ollama_rs::generation::completion::request::GenerationRequest::new(model, full_prompt);

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
