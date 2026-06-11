use crate::ai_companion::AICompanion;
use std::sync::Arc;
use tauri::Emitter;
use tauri::{State, Window};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use crate::conversation::ConversationMessage;
use regex::Regex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AiResponse {
    pub response: String,
    pub rag_used: bool,
    pub passages_retrieved: usize,
    pub passages: Vec<()>,
}

pub fn build_prompt_with_context(
    user_prompt: String,
) -> (String, bool, usize, Vec<()>) {
    (user_prompt, false, 0, vec![])
}

#[tauri::command]
pub async fn ask_ai(
    prompt: String,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<AiResponse, String> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

    let (enhanced_prompt, rag_used, passages_retrieved, _passages) = build_prompt_with_context(prompt);
    let ai = ai_state.lock().await;
    let response = ai.process_request(enhanced_prompt).await?;

    Ok(AiResponse {
        response,
        rag_used,
        passages_retrieved,
        passages: vec![],
    })
}

#[tauri::command]
pub async fn ask_ai_stream(
    prompt: String,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
) -> Result<(), String> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

    let (enhanced_prompt, rag_used, passages_retrieved, _passages) = build_prompt_with_context(prompt);

    let _ = window.emit("ai-stream-metadata", serde_json::json!({
        "rag_used": rag_used,
        "passages_retrieved": passages_retrieved,
        "passages": serde_json::Value::Null,
    }));

    let ai = ai_state.lock().await;
    ai.stream_chat_with_images(ai.model_clone(), enhanced_prompt, &[], None, &window).await?;
    Ok(())
}

#[tauri::command]
pub async fn ask_ai_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    image_b64: Option<String>,
) -> Result<AiResponse, String> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

    let prompt_with_context = {
        let ai = ai_state.lock().await;
        ai.build_prompt_with_context(prompt.clone(), &conversation_history)
    };

    let ai = ai_state.lock().await;
    let response = if let Some(ref img) = image_b64 {
        ai.process_request_with_images(prompt_with_context, Some(img), &conversation_history).await?
    } else {
        ai.process_request_with_context(prompt_with_context, &conversation_history).await?
    };

    Ok(AiResponse {
        response,
        rag_used: false,
        passages_retrieved: 0,
        passages: vec![],
    })
}

#[tauri::command]
pub async fn ask_ai_stream_with_conversation(
    prompt: String,
    conversation_history: Vec<ConversationMessage>,
    window: Window,
    ai_state: State<'_, Arc<Mutex<AICompanion>>>,
    image_b64: Option<String>,
) -> Result<(), String> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(format!("Prompt too long: maximum {} characters", MAX_PROMPT_LEN));
    }

    let metadata: serde_json::Value = serde_json::json!({
        "rag_used": false,
        "passages_retrieved": 0,
        "passages": serde_json::Value::Null,
    });
    let _ = window.emit("ai-stream-metadata", metadata);

    let prompt_with_context = {
        let ai = ai_state.lock().await;
        ai.build_prompt_with_context(prompt.clone(), &conversation_history)
    };

    let ai = ai_state.lock().await;
    ai.stream_chat_with_images(
        ai.model_clone(),
        prompt_with_context,
        &conversation_history,
        image_b64.as_deref(),
        &window,
    )
    .await?;
    Ok(())
}

pub async fn parse_ai_actions(ai_response: &str) -> Vec<AIAction> {
    let mut actions = Vec::new();

    let coord_pattern = Regex::new(r"(?:\(?(\d{1,5})\s*,\s*(\d{1,5})\)?)").unwrap();

    let all_coords: Vec<(i32, i32)> = coord_pattern
        .captures_iter(ai_response)
        .filter_map(|caps| {
            let x = caps.get(1)?.as_str().parse::<i32>().ok()?;
            let y = caps.get(2)?.as_str().parse::<i32>().ok()?;
            Some((x, y))
        })
        .collect();

    let has_move = ai_response.to_lowercase().contains("move")
        || ai_response.to_lowercase().contains("go to")
        || ai_response.to_lowercase().contains("navigate to");

    let has_drag = ai_response.to_lowercase().contains("drag");
    let has_click = ai_response.to_lowercase().contains("click");

    if has_click && all_coords.len() >= 2 && has_drag {
        let (start_x, start_y) = all_coords[0];
        let (end_x, end_y) = all_coords[1];
        actions.push(AIAction {
            action_type: "mouse_drag".to_string(),
            params: serde_json::json!({"startX": start_x, "startY": start_y, "endX": end_x, "endY": end_y}),
        });
    } else if has_click && !all_coords.is_empty() {
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
        let (x, y) = all_coords[0];
        actions.push(AIAction {
            action_type: "mouse_move".to_string(),
            params: serde_json::json!({"x": x, "y": y}),
        });
    }

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

    if ai_response.to_lowercase().contains("type") {
        if let Some(caps) = Regex::new(r#"type\s+["']?([^"'}\s][^"'}]*)["']?"#).unwrap().captures(ai_response) {
            if let Some(text) = caps.get(1) {
                let text = text.as_str();
                if text.len() > 1 && !text.contains("type") {
                    actions.push(AIAction {
                        action_type: "type_text".to_string(),
                        params: serde_json::json!({"text": text}),
                    });
                }
            }
        }
    }

    let key_pattern = r"(?:press|hit|key)\s+([a-z0-9\s\+]+?)[,\.\s]";
    if let Some(caps) = Regex::new(key_pattern).unwrap().captures(&ai_response.to_lowercase()) {
        if let Some(key_match) = caps.get(1) {
            let key_str = key_match.as_str().trim();
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

#[derive(Clone, Debug)]
pub struct AIAction {
    pub action_type: String,
    pub params: serde_json::Value,
}
