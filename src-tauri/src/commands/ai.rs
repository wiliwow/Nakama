use crate::ai_companion::AICompanion;
use std::sync::Arc;
use tauri::Emitter;
use tauri::{State, Window};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use serde::{Serialize, Deserialize};
use crate::conversation::ConversationMessage;
use regex::Regex;

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

/// Action to be executed by the AI agent
#[derive(Clone, Debug)]
pub struct AIAction {
    pub action_type: String,
    pub params: serde_json::Value,
}

#[cfg(feature = "swiftide_integration")]
pub async fn build_prompt_with_context(
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
pub async fn build_prompt_with_context(
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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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
    // Input validation
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

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

/// Parse AI response for action commands. Supports natural language patterns:
/// - "move to 100,200" or "go to coordinates (100,200)" or "I'll move the mouse to (100,200)"
/// - "click at 100,200" or "right click" or "double click" or "I'm clicking at..."
/// - "drag from 100,100 to 200,200"
/// - "scroll up 50" or "scroll down 100" or "scroll by 50"
/// - "type hello world" or "type "hello"" or "typing hello"
/// - "press enter" or "hit escape" or "press ctrl+c" or "ctrl c"
pub async fn parse_ai_actions(ai_response: &str) -> Vec<AIAction> {
    let mut actions = Vec::new();

    // Pattern for mouse movement: looks for coordinates in various formats
    // Matches: "move to 100,200", "go to (100,200)", "coordinates 100,200", "(100, 200)"
    let coord_pattern = Regex::new(r"(?:\(?(\d{1,5})\s*,\s*(\d{1,5})\)?)").unwrap();

    // Find all coordinate pairs in the response
    let all_coords: Vec<(i32, i32)> = coord_pattern
        .captures_iter(ai_response)
        .filter_map(|caps| {
            let x = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let y = caps.get(2)?.as_str().parse::<i32>().ok()?;
            Some((x, y))
        })
        .collect();

    // Check for mouse movement keywords
    let has_move = ai_response.to_lowercase().contains("move") ||
        ai_response.to_lowercase().contains("go to") ||
        ai_response.to_lowercase().contains("navigate to");

    // Check for drag keywords
    let has_drag = ai_response.to_lowercase().contains("drag");

    // Check for click keywords  
    let has_click = ai_response.to_lowercase().contains("click");

    // Determine action based on keywords
    if has_click && all_coords.len() >= 2 && has_drag {
        // Drag operation: use first two coordinate pairs
        let (start_x, start_y) = all_coords[0];
        let (end_x, end_y) = all_coords[1];
        actions.push(AIAction {
            action_type: "mouse_drag".to_string(),
            params: serde_json::json!({"startX": start_x, "startY": start_y, "endX": end_x, "endY": end_y}),
        });
    } else if has_click && !all_coords.is_empty() {
        // Click at coordinates - move then click
        let (x, y) = all_coords[0];
        actions.push(AIAction {
            action_type: "mouse_move".to_string(),
            params: serde_json::json!({"x": x, "y": y}),
        });
        let button = if ai_response.to_lowercase().contains("right") { "right" }
            else if ai_response.to_lowercase().contains("middle") { "middle" }
            else { "left" };
        actions.push(AIAction {
            action_type: "mouse_click".to_string(),
            params: serde_json::json!({"button": button}),
        });
    } else if has_move && !all_coords.is_empty() && !has_click && !has_drag {
        // Just movement
        let (x, y) = all_coords[0];
        actions.push(AIAction {
            action_type: "mouse_move".to_string(),
            params: serde_json::json!({"x": x, "y": y}),
        });
    }

    // Pattern for scroll: "scroll up 50" or "scroll down 100"
    if let Some(scroll_caps) = Regex::new(r"scroll\s+(?:up|down)?\s*(-?\d+)").unwrap().captures(&ai_response.to_lowercase()) {
        let amount: i32 = scroll_caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(50);
        if ai_response.to_lowercase().contains("up") {
            actions.push(AIAction {
                action_type: "mouse_scroll".to_string(),
                params: serde_json::json!({"amount": amount}),
            });
        } else if ai_response.to_lowercase().contains("down") {
            actions.push(AIAction {
                action_type: "mouse_scroll".to_string(),
                params: serde_json::json!({"amount": -amount.abs()}),
            });
        }
    }

    // Pattern for typing: "type hello" or "type "hello"" or "typing hello"
    if ai_response.to_lowercase().contains("type") {
        if let Some(caps) = Regex::new(r#"type\s+["']?([^"'}\s][^"'}]*)["']?"#).unwrap().captures(ai_response) {
            if let Some(text) = caps.get(1) {
                let text = text.as_str();
                // Avoid matching things like "type of" or "prototype"
                if text.len() > 1 && !text.contains("type") {
                    actions.push(AIAction {
                        action_type: "type_text".to_string(),
                        params: serde_json::json!({"text": text}),
                    });
                }
            }
        }
    }

    // Pattern for key presses: "press enter" or "hit escape" or "press ctrl c"
    let key_pattern = r"(?:press|hit|key)\s+([a-z0-9\s\+]+?)[,\.\s]";
    if let Some(caps) = Regex::new(key_pattern).unwrap().captures(&ai_response.to_lowercase()) {
        if let Some(key_match) = caps.get(1) {
            let key_str = key_match.as_str().trim();
            // Filter out common false positives
            if key_str.len() > 1 && !key_str.starts_with("the ") && key_str.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '+') {
                actions.push(AIAction {
                    action_type: "press_key".to_string(),
                    params: serde_json::json!({"key": key_str}),
                });
            }
        }
    }

    actions
}
