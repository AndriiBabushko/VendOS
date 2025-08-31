mod sony_bridge;

use sony_bridge::*;

// demo з шаблону
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn sdk_init_cmd() -> Result<(), String> {
    if crsdk_init() { Ok(()) } else { Err("CRSDK init failed".into()) }
}

#[tauri::command]
fn camera_connect_cmd() -> Result<(), String> {
    if crsdk_connect_first_usb() { Ok(()) } else { Err("No camera or connect failed".into()) }
}

#[tauri::command]
fn camera_set_save_dir_cmd(directory_path: String) -> Result<(), String> {
    if crsdk_set_save_dir(&directory_path) { Ok(()) } else { Err("SetSaveInfo failed".into()) }
}

#[tauri::command]
fn camera_capture_and_get_path_cmd(timeout_ms: u32) -> Result<String, String> {
    let saved = crsdk_capture_blocking(timeout_ms);
    if saved.is_empty() { Err("Capture timeout or error".into()) } else { Ok(saved) }
}

#[tauri::command]
fn camera_liveview_frame_cmd() -> Result<Vec<u8>, String> {
    let bytes = crsdk_liveview_frame();
    if bytes.is_empty() { Err("No frame".into()) } else { Ok(bytes) }
}

#[tauri::command]
fn camera_is_connected_cmd() -> bool {
    crsdk_is_connected()
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            sdk_init_cmd,
            camera_connect_cmd,
            camera_set_save_dir_cmd,
            camera_capture_and_get_path_cmd,
            camera_liveview_frame_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
