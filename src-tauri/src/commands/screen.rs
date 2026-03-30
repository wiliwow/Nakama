use base64::Engine as _;
use image::{EncodableLayout, ImageBuffer, ImageOutputFormat};
use screenshots::Screen;
use serde::Serialize;
use std::io::Cursor;

#[derive(Serialize)]
pub struct ScreenCapture {
    pub id: usize,
    pub width: u32,
    pub height: u32,
    pub data_url: String,
}

fn capture_to_data_url(screen: Screen) -> Result<ScreenCapture, String> {
    let image = screen.capture().map_err(|e| e.to_string())?;
    let width = image.width();
    let height = image.height();
    let bytes = image.as_bytes();

    let mut rgba = Vec::with_capacity(bytes.len());
    for chunk in bytes.chunks_exact(4) {
        rgba.push(chunk[2]);
        rgba.push(chunk[1]);
        rgba.push(chunk[0]);
        rgba.push(chunk[3]);
    }

    let buffer = ImageBuffer::from_raw(width, height, rgba)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;

    let dyn_image = image::DynamicImage::ImageRgba8(buffer);
    let mut png_bytes = Vec::new();
    dyn_image
        .write_to(&mut Cursor::new(&mut png_bytes), ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;

    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png_bytes)
    );

    Ok(ScreenCapture {
        id: screen.display_info.id as usize,
        width,
        height,
        data_url,
    })
}

#[tauri::command]
pub async fn capture_primary_screen() -> Result<ScreenCapture, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let primary = screens
        .into_iter()
        .min_by_key(|screen| screen.display_info.id)
        .ok_or_else(|| "No screens available".to_string())?;
    capture_to_data_url(primary)
}

#[tauri::command]
pub async fn capture_all_screens() -> Result<Vec<ScreenCapture>, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    screens
        .into_iter()
        .enumerate()
        .map(|(idx, screen)| {
            let mut capture = capture_to_data_url(screen)?;
            capture.id = idx;
            Ok(capture)
        })
        .collect()
}
