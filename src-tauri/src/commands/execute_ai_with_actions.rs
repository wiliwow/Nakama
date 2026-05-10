use tauri::{Window, State, Emitter};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::StreamExt;
use enigo::{Button, Key};
use crate::{AICompanion, AppState};
use crate::commands::ai::parse_ai_actions;

#[tauri::command]
pub async fn execute_ai_with_actions(
    prompt: String,
    conversation_history: Vec<crate::conversation::ConversationMessage>,
    window: Window,
    ai_state: State<'_, Arc<TokioMutex<AICompanion>>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Build prompt with action instructions system prompt
    let system_prompt = "You are a helpful AI assistant that can control the user's computer through mouse and keyboard actions.\n\nYou can execute computer control actions by including specific commands in your response:\n- Mouse movement: \"move to 100,200\" or \"go to coordinates (100,200)\"\n- Clicking: \"click at 100,200\" or \"right click\" or \"middle click\"\n- Dragging: \"drag from 100,100 to 200,200\"\n- Scrolling: \"scroll up 50\" or \"scroll down 100\"\n- Typing: \"type hello world\"\n- Key presses: \"press enter\" or \"hit escape\" or \"press ctrl+c\" or \"ctrl a\"\n\nWhen the user asks you to perform an action, describe what you're doing AND include the command.\nAnswer user questions clearly and directly.";

    // Build conversation context with system prompt
    let full_prompt = if conversation_history.is_empty() {
        format!("{}\n\nUser: {}\n\nAssistant:", system_prompt, prompt)
    } else {
        let mut p = system_prompt.to_string();
        for msg in &conversation_history {
            let role = match msg.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                _ => "Unknown",
            };
            p.push_str(&format!("\n\n{}: {}", role, msg.content));
        }
        p.push_str(&format!("\n\nUser: {}\n\nAssistant:", prompt));
        p
    };

    // Get model name
    let model = {
        let ai = ai_state.lock().await;
        ai.model_clone()
    };

    // Stream the response directly (no spawn to keep access to app_state)
    let client = ollama_rs::Ollama::default();
    let request = ollama_rs::generation::completion::request::GenerationRequest::new(model, full_prompt);
    let mut full_response = String::new();

    match client.generate_stream(request).await {
        Ok(mut stream) => {
            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunks) => {
                        for piece in chunks {
                            full_response.push_str(&piece.response);
                            let _ = window.emit("ai-stream-chunk", piece.response.clone());
                        }
                    }
                    Err(e) => {
                        let _ = window.emit("ai-stream-error", e.to_string());
                        return Err(e.to_string());
                    }
                }
            }
        }
        Err(e) => {
            let _ = window.emit("ai-stream-error", e.to_string());
            return Err(e.to_string());
        }
    }

    // Parse and execute AI actions from the full response
    let actions = parse_ai_actions(&full_response).await;
    let app = app_state.inner();
    for action in actions {
        let result = match action.action_type.as_str() {
            "mouse_move" => {
                let x = action.params["x"].as_i64().ok_or("missing x")? as i32;
                let y = action.params["y"].as_i64().ok_or("missing y")? as i32;
                app.input.lock().unwrap().mouse_move(x, y);
                Ok(format!("Moved mouse to ({}, {})", x, y))
            }
            "mouse_click" => {
                let btn = match action.params.get("button").and_then(|v| v.as_str()) {
                    Some("right") => Button::Right,
                    Some("middle") => Button::Middle,
                    _ => Button::Left,
                };
                app.input.lock().unwrap().mouse_click(btn);
                Ok(format!("Clicked mouse button"))
            }
            "mouse_scroll" => {
                let amount = action.params["amount"].as_i64().ok_or("missing amount")? as i32;
                app.input.lock().unwrap().mouse_scroll(amount);
                Ok(format!("Scrolled by {}", amount))
            }
            "mouse_drag" => {
                let start_x = action.params["startX"].as_i64().ok_or("missing startX")? as i32;
                let start_y = action.params["startY"].as_i64().ok_or("missing startY")? as i32;
                let end_x = action.params["endX"].as_i64().ok_or("missing endX")? as i32;
                let end_y = action.params["endY"].as_i64().ok_or("missing endY")? as i32;
                app.input.lock().unwrap().mouse_move(start_x, start_y);
                app.input.lock().unwrap().mouse_down(Button::Left);
                app.input.lock().unwrap().mouse_move(end_x, end_y);
                app.input.lock().unwrap().mouse_up(Button::Left);
                Ok(format!("Dragged from ({},{}) to ({},{})", start_x, start_y, end_x, end_y))
            }
            "type_text" => {
                let text = action.params["text"].as_str().ok_or("missing text")?;
                app.input.lock().unwrap().key_sequence(text);
                Ok(format!("Typed: {}", text))
            }
            "press_key" => {
                let key_str = action.params["key"].as_str().ok_or("missing key")?;
                let lower_key = key_str.to_lowercase();

                // Handle key combinations like "ctrl c" or "ctrl+c"
                if lower_key.contains("ctrl") || lower_key.contains("control") {
                    let parts: Vec<&str> = lower_key.split(|c| c == '+' || c == ' ').collect();
                    for part in parts {
                        let k = match part.trim() {
                            "ctrl" | "control" => continue,
                            "c" => Key::Unicode('c'),
                            "v" => Key::Unicode('v'),
                            "x" => Key::Unicode('x'),
                            "a" => Key::Unicode('a'),
                            "z" => Key::Unicode('z'),
                            "s" => Key::Unicode('s'),
                            "tab" => Key::Tab,
                            _ => Key::Unicode(part.chars().next().unwrap_or(' ')),
                        };
                        app.input.lock().unwrap().key_click(k);
                    }
                    Ok(format!("Pressed key combination: {}", key_str))
                } else {
                    let key = match lower_key.as_str() {
                        "enter" => Key::Return,
                        "tab" => Key::Tab,
                        "space" => Key::Space,
                        "backspace" => Key::Backspace,
                        "escape" | "esc" => Key::Escape,
                        "up" => Key::UpArrow,
                        "down" => Key::DownArrow,
                        "left" => Key::LeftArrow,
                        "right" => Key::RightArrow,
                        "ctrl" | "control" => Key::Control,
                        "alt" => Key::Alt,
                        "shift" => Key::Shift,
                        "win" | "cmd" | "meta" => Key::Meta,
                        "delete" | "del" => Key::Delete,
                        "home" => Key::Home,
                        "end" => Key::End,
                        "pageup" | "pgup" => Key::PageUp,
                        "pagedown" | "pgdn" => Key::PageDown,
                        "f1" => Key::F1,
                        "f2" => Key::F2,
                        "f3" => Key::F3,
                        "f4" => Key::F4,
                        "f5" => Key::F5,
                        "f6" => Key::F6,
                        "f7" => Key::F7,
                        "f8" => Key::F8,
                        "f9" => Key::F9,
                        "f10" => Key::F10,
                        "f11" => Key::F11,
                        "f12" => Key::F12,
                        _ => Key::Unicode(key_str.chars().next().unwrap_or(' ')),
                    };
                    app.input.lock().unwrap().key_click(key);
                    Ok(format!("Pressed: {}", key_str))
                }
            }
            _ => Err(format!("Unknown action type: {}", action.action_type)),
        };
        match result {
            Ok(res) => { let _ = window.emit("ai-action-executed", res); },
            Err(e) => { let _ = window.emit("ai-action-error", e); },
        }
    }

    let _ = window.emit("ai-stream-done", ());
    Ok(())
}