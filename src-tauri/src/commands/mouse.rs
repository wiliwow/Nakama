use crate::AppState;
use enigo::{Button, Key};
use std::{thread, time::Duration};

fn parse_button(button: &str) -> Button {
    match button {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    }
}

#[tauri::command]
pub async fn mouse_move(x: i32, y: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.input.lock().unwrap().mouse_move(x, y);
    Ok(())
}

#[tauri::command]
pub async fn mouse_click(button: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let btn = parse_button(button);
    state.input.lock().unwrap().mouse_click(btn);
    Ok(())
}

#[tauri::command]
pub async fn mouse_down(button: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let btn = parse_button(button);
    state.input.lock().unwrap().mouse_down(btn);
    Ok(())
}

#[tauri::command]
pub async fn mouse_up(button: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let btn = parse_button(button);
    state.input.lock().unwrap().mouse_up(btn);
    Ok(())
}

#[tauri::command]
pub async fn mouse_scroll(amount: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.input.lock().unwrap().mouse_scroll(amount);
    Ok(())
}

#[tauri::command]
pub async fn mouse_drag(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut locked = state.input.lock().unwrap();
    locked.mouse_move(start_x, start_y);
    locked.mouse_down(Button::Left);
    drop(locked);
    thread::sleep(Duration::from_millis(50));
    state.input.lock().unwrap().mouse_move(end_x, end_y);
    state.input.lock().unwrap().mouse_up(Button::Left);
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
        _ => Key::Unicode(key.chars().next().unwrap_or(' ')),
    };
    state.input.lock().unwrap().key_click(key);
    Ok(())
}
