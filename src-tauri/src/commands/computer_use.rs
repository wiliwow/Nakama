use enigo::Button;
use tauri::{Emitter, State, Window};
use serde_json::json;

use crate::AppState;

/// Execute a Computer Use API tool
#[tauri::command]
pub async fn execute_computer_tool(
    action: String,
    x: Option<i32>,
    y: Option<i32>,
    text: Option<String>,
    window: Window,
    app_state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let result = match action.as_str() {
        "screenshot" => execute_screenshot().await,
        "mouse_move" => {
            let x = x.ok_or("mouse_move requires x coordinate")?;
            let y = y.ok_or("mouse_move requires y coordinate")?;
            execute_mouse_move(&app_state, x, y)
        }
        "left_click" => execute_mouse_click(&app_state, Button::Left),
        "right_click" => execute_mouse_click(&app_state, Button::Right),
        "middle_click" => execute_mouse_click(&app_state, Button::Middle),
        "double_click" => execute_double_click(&app_state),
        "left_click_drag" => {
            let x = x.ok_or("left_click_drag requires x coordinate")?;
            let y = y.ok_or("left_click_drag requires y coordinate")?;
            execute_left_click_drag(&app_state, x, y)
        }
        "key" => {
            let text = text.ok_or("key action requires text parameter")?;
            execute_key(&app_state, &text)
        }
        "type" => {
            let text = text.ok_or("type action requires text parameter")?;
            execute_type(&app_state, &text)
        }
        "cursor_position" => Ok(json!({ "type": "computer_output", "x": 0, "y": 0 })),
        "scroll" => {
            let scroll_x = x.unwrap_or(0);
            let scroll_y = y.ok_or("scroll requires y delta value")?;
            execute_scroll(&app_state, scroll_x, scroll_y)
        }
        _ => return Err(format!("Unknown computer tool action: {}", action)),
    };

    emit_result(&window, "computer-tool-result", "computer-tool-error", &result);
    result
}

/// Execute a Text Editor API tool
#[tauri::command]
pub async fn execute_text_editor_tool(
    command: String,
    path: Option<String>,
    file_text: Option<String>,
    old_str: Option<String>,
    new_str: Option<String>,
    insert_line: Option<i32>,
    view_range: Option<[i32; 2]>,
    window: Window,
) -> Result<serde_json::Value, String> {
    let result = match command.as_str() {
        "view" => execute_view_file(path.as_deref(), view_range).await,
        "create" => execute_create_file(path.as_deref(), file_text.as_deref()).await,
        "str_replace" => {
            execute_str_replace(path.as_deref(), old_str.as_deref(), new_str.as_deref()).await
        }
        "insert" => {
            execute_insert_line(path.as_deref(), insert_line, new_str.as_deref()).await
        }
        "undo" => Err("Undo not yet implemented".to_string()),
        _ => return Err(format!("Unknown text editor command: {}", command)),
    };

    emit_result(&window, "text-editor-tool-result", "text-editor-tool-error", &result);
    result
}

/// Execute a Bash API tool
#[tauri::command]
pub async fn execute_bash_tool(command: String, window: Window) -> Result<serde_json::Value, String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let status = output.status.code().unwrap_or(-1);
    let result = json!({
        "type": "bash_output",
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "exit_code": status
    });

    let _ = window.emit("bash-tool-result", result.clone());
    Ok(result)
}

// Computer tool implementations

async fn execute_screenshot() -> Result<serde_json::Value, String> {
    let img = crate::commands::screen::capture_primary_screen()
        .await
        .map_err(|e| format!("Failed to capture screenshot: {}", e))?;

    let base64_img = img.data_url.strip_prefix("data:image/png;base64,").unwrap_or(&img.data_url).to_string();

    Ok(json!({
        "type": "computer_screenshot",
        "data": base64_img,
        "width": img.width,
        "height": img.height
    }))
}

fn execute_mouse_move(state: &AppState, x: i32, y: i32) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().mouse_move(x, y);
    Ok(json!({ "type": "computer_output", "x": x, "y": y }))
}

fn execute_mouse_click(state: &AppState, button: Button) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().mouse_click(button);
    Ok(json!({ "type": "computer_output" }))
}

fn execute_double_click(state: &AppState) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().mouse_click(Button::Left);
    std::thread::sleep(std::time::Duration::from_millis(50));
    state.input.lock().unwrap().mouse_click(Button::Left);
    Ok(json!({ "type": "computer_output" }))
}

fn execute_left_click_drag(state: &AppState, start_x: i32, start_y: i32) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().mouse_move(start_x, start_y);
    state.input.lock().unwrap().mouse_down(Button::Left);
    Ok(json!({ "type": "computer_output", "x": start_x, "y": start_y }))
}

fn execute_key(state: &AppState, text: &str) -> Result<serde_json::Value, String> {
    let key = parse_key(text);
    state.input.lock().unwrap().key_click(key);
    Ok(json!({ "type": "computer_output", "key": text }))
}

fn execute_type(state: &AppState, text: &str) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().key_sequence(text);
    Ok(json!({ "type": "computer_output", "text": text }))
}

fn execute_scroll(state: &AppState, _x: i32, y: i32) -> Result<serde_json::Value, String> {
    state.input.lock().unwrap().mouse_scroll(y);
    Ok(json!({ "type": "computer_output", "x": _x, "y": y }))
}

// Text editor implementations

async fn execute_view_file(path: Option<&str>, view_range: Option<[i32; 2]>) -> Result<serde_json::Value, String> {
    let path = path.ok_or("view command requires path")?;
    let mut content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    if let Some([start, end]) = view_range {
        let lines: Vec<&str> = content.lines().collect();
        let start = start.max(1) as usize;
        let end = end.min(lines.len() as i32) as usize;
        content = lines[start.saturating_sub(1)..end].join("\n");
    }

    Ok(json!({ "type": "text_editor_output", "file_text": content }))
}

async fn execute_create_file(path: Option<&str>, file_text: Option<&str>) -> Result<serde_json::Value, String> {
    let path = path.ok_or("create command requires path")?;
    let content = file_text.ok_or("create command requires file_text")?;
    tokio::fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to create file {}: {}", path, e))?;
    Ok(json!({ "type": "text_editor_output" }))
}

async fn execute_str_replace(
    path: Option<&str>,
    old_str: Option<&str>,
    new_str: Option<&str>,
) -> Result<serde_json::Value, String> {
    let path = path.ok_or("str_replace command requires path")?;
    let old = old_str.ok_or("str_replace command requires old_str")?;
    let new = new_str.ok_or("str_replace command requires new_str")?;

    let mut content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    if !content.contains(old) {
        return Err(format!("String not found in file: {}", old));
    }

    content = content.replace(old, new);
    tokio::fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file {}: {}", path, e))?;

    Ok(json!({ "type": "text_editor_output" }))
}

async fn execute_insert_line(
    path: Option<&str>,
    insert_line: Option<i32>,
    new_str: Option<&str>,
) -> Result<serde_json::Value, String> {
    let path = path.ok_or("insert command requires path")?;
    let new = new_str.ok_or("insert command requires new_str")?;
    let line = insert_line.ok_or("insert command requires insert_line")?;

    let mut content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    let lines: Vec<&str> = content.lines().collect();
    if !(0..=lines.len() as i32).contains(&line) {
        return Err(format!("Line {} out of range (0-{})", line, lines.len()));
    }

    let mut new_lines = Vec::new();
    for (i, line_content) in lines.iter().enumerate() {
        if i as i32 == line {
            new_lines.push(new);
        }
        new_lines.push(line_content);
    }
    if line == lines.len() as i32 {
        new_lines.push(new);
    }

    content = new_lines.join("\n");
    tokio::fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file {}: {}", path, e))?;

    Ok(json!({ "type": "text_editor_output" }))
}

// Helper functions

fn emit_result(window: &Window, success_event: &str, error_event: &str, result: &Result<serde_json::Value, String>) {
    match result {
        Ok(res) => {
            let _ = window.emit(success_event, res.clone());
        }
        Err(e) => {
            let _ = window.emit(error_event, e.clone());
        }
    }
}

fn parse_key(key_str: &str) -> enigo::Key {
    let lower = key_str.to_lowercase();
    match lower.as_str() {
        "enter" => enigo::Key::Return,
        "tab" => enigo::Key::Tab,
        "space" => enigo::Key::Space,
        "backspace" => enigo::Key::Backspace,
        "escape" | "esc" => enigo::Key::Escape,
        "up" => enigo::Key::UpArrow,
        "down" => enigo::Key::DownArrow,
        "left" => enigo::Key::LeftArrow,
        "right" => enigo::Key::RightArrow,
        "ctrl" | "control" => enigo::Key::Control,
        "alt" => enigo::Key::Alt,
        "shift" => enigo::Key::Shift,
        "win" | "cmd" | "meta" => enigo::Key::Meta,
        "delete" | "del" => enigo::Key::Delete,
        "home" => enigo::Key::Home,
        "end" => enigo::Key::End,
        "pageup" | "pgup" => enigo::Key::PageUp,
        "pagedown" | "pgdn" => enigo::Key::PageDown,
        "f1" => enigo::Key::F1,
        "f2" => enigo::Key::F2,
        "f3" => enigo::Key::F3,
        "f4" => enigo::Key::F4,
        "f5" => enigo::Key::F5,
        "f6" => enigo::Key::F6,
        "f7" => enigo::Key::F7,
        "f8" => enigo::Key::F8,
        "f9" => enigo::Key::F9,
        "f10" => enigo::Key::F10,
        "f11" => enigo::Key::F11,
        "f12" => enigo::Key::F12,
        _ => key_str.chars().next().map(enigo::Key::Unicode).unwrap_or(enigo::Key::Unicode(' ')),
    }
}