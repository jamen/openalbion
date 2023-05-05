// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::api::dialog::blocking::FileDialogBuilder;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_dialog])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn open_dialog() -> String {
    match FileDialogBuilder::new().pick_folder() {
        Some(x) => x.as_os_str().to_str().unwrap().to_string(),
        None => "".to_string(),
    }
}
