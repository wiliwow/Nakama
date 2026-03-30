#[tauri::command]
pub fn fetch_files(files: Vec<String>) -> Vec<String> {
  println!("Received files from frontend: {:#?}", files);
  for file in &files {
    println!(" - {}", file);
  }
  files
}