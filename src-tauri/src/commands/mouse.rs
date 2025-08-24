use enigo::{Button, Key};
use std::sync::Mutex;
use crate::input_controller::InputController;

pub struct AppState {
    pub input: Mutex<InputController>,
}

#[tauri::command]
pub async fn mouse_move(x: i32, y: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.input.lock().unwrap().mouse_move(x, y);
    Ok(())
}

#[tauri::command]
pub async fn mouse_click(button: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let btn = match button {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    };
    state.input.lock().unwrap().mouse_click(btn);
    Ok(())
}

#[tauri::command]
pub async fn type_text(text: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.input.lock().unwrap().key_sequence(text);
    Ok(())
}

#[tauri::command]
pub async fn press_key(key: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let key = match key {
        "enter" => Key::Return,
        "tab" => Key::Tab,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "escape" => Key::Escape,
        _ => Key::Unicode(key.chars().next().unwrap()),
    };
    state.input.lock().unwrap().key_click(key);
    Ok(())
}
