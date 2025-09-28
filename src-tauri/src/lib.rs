#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gp;
mod print;

use std::collections::HashMap;
use tauri::async_runtime::spawn_blocking;

#[tauri::command]
async fn gp_detect_cmd() -> Result<Vec<String>, String> {
    spawn_blocking(|| {
        gp::ensure_tool()?;
        gp::auto_detect()
    })
        .await
        .map_err(|e| format!("detect join error: {e}"))?
}

#[tauri::command]
async fn gp_capture_cmd(save_dir: String, port: Option<String>) -> Result<String, String> {
    spawn_blocking(move || {
        gp::ensure_tool()?;
        gp::capture_to(&save_dir, port.as_deref())
    })
        .await
        .map_err(|e| format!("capture join error: {e}"))?
}

#[tauri::command]
async fn fs_read_base64_cmd(path: String) -> Result<String, String> {
    use base64::Engine;
    spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    })
        .await
        .map_err(|e| format!("fs_read join error: {e}"))?
}

#[tauri::command]
async fn cups_list_printers_cmd() -> Result<Vec<print::CupsPrinter>, String> {
    spawn_blocking(|| {
        print::ensure_cups()?;
        print::list_printers()
    })
        .await
        .map_err(|e| format!("cups join error: {e}"))?
}

#[tauri::command]
async fn cups_printer_options_cmd(printer: String) -> Result<Vec<String>, String> {
    spawn_blocking(move || print::printer_options(&printer))
        .await
        .map_err(|e| format!("cups options join error: {e}"))?
}

#[tauri::command]
async fn cups_print_file_cmd(
    printer: String,
    path: String,
    copies: Option<u32>,
    options: Option<HashMap<String, String>>,
) -> Result<(), String> {
    spawn_blocking(move || {
        print::ensure_cups()?;
        print::print_file(&printer, &path, copies.unwrap_or(1), options)
    })
        .await
        .map_err(|e| format!("cups print join error: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gp_detect_cmd,
            gp_capture_cmd,
            fs_read_base64_cmd,
            cups_list_printers_cmd,
            cups_printer_options_cmd,
            cups_print_file_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
