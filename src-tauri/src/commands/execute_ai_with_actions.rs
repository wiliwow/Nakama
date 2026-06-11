use tauri::{Window, State, Emitter};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
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
    image_b64: Option<String>,
) -> Result<(), String> {
    let ai = ai_state.lock().await;

    // One-shot completion so we have the full text to parse actions from
    let full_response = ai
        .generate_completion(prompt, &conversation_history, image_b64.as_deref())
        .await?;

    // Stream the same response into the UI and signal completion
    let _ = window.emit("ai-stream-chunk", &full_response);
    let _ = window.emit("ai-stream-done", ());

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

    Ok(())
}