use crate::AppState;
use enigo::{Button, Key};
use rand::Rng;
use tokio::time::sleep;
use std::time::Duration;

const MIN_COORD: i32 = 0;
const MAX_COORD: i32 = 10_000; // Support up to 10K displays

fn validate_coords(x: i32, y: i32) -> Result<(), String> {
    if x < MIN_COORD || x > MAX_COORD || y < MIN_COORD || y > MAX_COORD {
        return Err(format!("Coordinates out of range ({}..{}) got ({}, {})", MIN_COORD, MAX_COORD, x, y));
    }
    Ok(())
}

fn parse_button(button: &str) -> Button {
    match button {
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    }
}

// Add random human-like delay
async fn human_delay(min_ms: u64, max_ms: u64) {
    let delay = {
        let mut rng = rand::thread_rng();
        rng.gen_range(min_ms..=max_ms)
    };
    sleep(Duration::from_millis(delay)).await;
}

// Smooth curved mouse movement using Bézier curves
async fn smooth_mouse_move(state: &tauri::State<'_, AppState>, start_x: i32, start_y: i32, end_x: i32, end_y: i32, duration_ms: u64) {
    let (control_x, control_y) = {
        let mut rng = rand::thread_rng();
        let cx = (start_x + end_x) / 2 + rng.gen_range(-50..=50);
        let cy = (start_y + end_y) / 2 + rng.gen_range(-50..=50);
        (cx, cy)
    };

    let steps = 50; // Number of interpolation points
    let step_duration = duration_ms / steps as u64;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;

        // Quadratic Bézier curve formula
        let x = ((1.0 - t).powi(2) * start_x as f64) +
                (2.0 * (1.0 - t) * t * control_x as f64) +
                (t.powi(2) * end_x as f64);

        let y = ((1.0 - t).powi(2) * start_y as f64) +
                (2.0 * (1.0 - t) * t * control_y as f64) +
                (t.powi(2) * end_y as f64);

        state.input.lock().unwrap().mouse_move(x as i32, y as i32);
        sleep(Duration::from_millis(step_duration)).await;
    }
}

#[tauri::command]
pub async fn mouse_move(x: i32, y: i32, state: tauri::State<'_, AppState>) -> Result<(), String> {
    validate_coords(x, y)?;
    // Get current position (approximate)
    let current_pos = (0, 0); // enigo doesn't provide get_position, so we'll assume from center

    // Calculate distance for dynamic timing
    let distance = (((x - current_pos.0) as f64).powi(2) + ((y - current_pos.1) as f64).powi(2)).sqrt();
    let duration = (distance * 2.0).max(200.0).min(800.0) as u64; // 200-800ms based on distance

    smooth_mouse_move(&state, current_pos.0, current_pos.1, x, y, duration).await;
    human_delay(50, 150).await; // Brief pause after movement
    Ok(())
}

#[tauri::command]
pub async fn mouse_click(button: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let btn = parse_button(button);
    human_delay(100, 300).await; // Human reaction time before clicking
    state.input.lock().unwrap().mouse_down(btn);
    human_delay(50, 150).await; // Hold time
    state.input.lock().unwrap().mouse_up(btn);
    human_delay(200, 500).await; // Recovery time after click
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
    validate_coords(start_x, start_y)?;
    validate_coords(end_x, end_y)?;
    // Move to start position smoothly
    smooth_mouse_move(&state, 0, 0, start_x, start_y, 400).await;
    human_delay(150, 300).await; // Pause before starting drag

    // Press down
    state.input.lock().unwrap().mouse_down(Button::Left);
    human_delay(100, 200).await; // Hold time before moving

    // Drag to end position with slight curve
    let distance = (((end_x - start_x) as f64).powi(2) + ((end_y - start_y) as f64).powi(2)).sqrt();
    let drag_duration = (distance * 1.5).max(300.0).min(1000.0) as u64;
    smooth_mouse_move(&state, start_x, start_y, end_x, end_y, drag_duration).await;

    human_delay(50, 150).await; // Brief hold at end
    state.input.lock().unwrap().mouse_up(Button::Left);
    human_delay(300, 600).await; // Recovery time after drag
    Ok(())
}

#[tauri::command]
pub async fn type_text(text: &str, state: tauri::State<'_, AppState>) -> Result<(), String> {
    for ch in text.chars() {
        state.input.lock().unwrap().key_click(Key::Unicode(ch));
        // Variable typing speed - faster for common chars, slower for punctuation
        let delay = match ch {
            ' ' | 'a' | 'e' | 'i' | 'o' | 'u' | 't' | 'n' | 's' | 'r' => 80..150,
            '.' | ',' | '!' | '?' | ':' | ';' => 200..400,
            _ => 100..200,
        };
        human_delay(delay.start, delay.end).await;
    }
    human_delay(300, 600).await; // Pause after typing
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
        "up" => Key::UpArrow,
        "down" => Key::DownArrow,
        "left" => Key::LeftArrow,
        "right" => Key::RightArrow,
        "ctrl" => Key::Control,
        "alt" => Key::Alt,
        "shift" => Key::Shift,
        "win" | "cmd" => Key::Meta,
        "delete" => Key::Delete,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
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
        _ => Key::Unicode(key.chars().next().unwrap_or(' ')),
    };
    human_delay(50, 150).await;
    state.input.lock().unwrap().key_click(key);
    human_delay(100, 250).await;
    Ok(())
}
