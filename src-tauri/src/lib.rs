#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod sony_bridge;
mod gp;
use sony_bridge::*;

#[tauri::command]
fn sdk_init_cmd() -> Result<(), String> {
    if crsdk_init() {
        std::thread::sleep(std::time::Duration::from_millis(200));
        Ok(())
    } else {
        Err("CRSDK init failed".into())
    }
}

#[tauri::command]
fn sdk_version_cmd() -> String {
    let v = crsdk_version_raw();
    let major = (v >> 24) & 0xFF;
    let minor = (v >> 16) & 0xFF;
    let patch = (v >> 8)  & 0xFF;
    format!("{major}.{minor:02}.{patch:02} (raw={v})")
}

#[tauri::command]
fn camera_scan_cmd() -> Result<Vec<String>, String> {
    if !crsdk_enum_refresh() {
        Err(format!("Enum fail: {} ({})", crsdk_last_error_text(), crsdk_last_error()))
    } else {
        let n = crsdk_enum_count();
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n { out.push(crsdk_enum_model(i)); }
        Ok(out)
    }
}

#[tauri::command]
fn camera_connect_cmd() -> Result<(), String> {
    if crsdk_connect_first_usb() { Ok(()) }
    else { Err(format!("Connect failed: {} ({})", crsdk_last_error_text(), crsdk_last_error())) }
}

#[tauri::command]
fn camera_connect_by_index_cmd(index: u32) -> Result<(), String> {
    let code = crsdk_connect_index(index);
    if code == 0 { Ok(()) } else { Err(format!("CrError={code} {}", crsdk_last_error_text())) }
}

#[tauri::command]
fn camera_is_connected_cmd() -> bool { crsdk_is_connected() }

#[tauri::command]
fn camera_set_save_dir_cmd(directory_path: String) -> Result<(), String> {
    if crsdk_set_save_dir(&directory_path) { Ok(()) }
    else { Err(format!("SetSaveInfo failed: {} ({})", crsdk_last_error_text(), crsdk_last_error())) }
}

#[tauri::command]
fn camera_capture_and_get_path_cmd(timeout_ms: u32) -> Result<String, String> {
    let saved = crsdk_capture_blocking(timeout_ms);
    if saved.is_empty() {
        Err(format!("Capture failed: {} ({})", crsdk_last_error_text(), crsdk_last_error()))
    } else { Ok(saved) }
}

#[tauri::command]
fn camera_liveview_frame_cmd() -> Result<Vec<u8>, String> {
    let bytes = crsdk_liveview_frame();
    if bytes.is_empty() { Err(format!("No frame: {} ({})", crsdk_last_error_text(), crsdk_last_error())) }
    else { Ok(bytes) }
}


#[tauri::command]
fn gp_detect_cmd() -> Result<Vec<String>, String> {
    gp::ensure_tool()?;
    gp::auto_detect()
}

#[tauri::command]
fn gp_capture_cmd(save_dir: String) -> Result<String, String> {
    gp::ensure_tool()?;
    gp::capture_to(&save_dir)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gp_detect_cmd,
            gp_capture_cmd,
            sdk_init_cmd,
            sdk_version_cmd,
            camera_scan_cmd,
            camera_connect_cmd,
            camera_connect_by_index_cmd,
            camera_is_connected_cmd,
            camera_set_save_dir_cmd,
            camera_capture_and_get_path_cmd,
            camera_liveview_frame_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
