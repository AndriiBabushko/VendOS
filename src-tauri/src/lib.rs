mod sony_bridge;

use std::path::Path;
use std::process::Command;
use sony_bridge::*;

// ---------- macOS: налаштувати пошук CRSDK dylib і адаптерів до Init() ----------
#[cfg(target_os = "macos")]
fn setup_crsdk_dylib_search_paths() {
    use std::env;
    use std::path::PathBuf;

    // кандидати директорій у порядку пріоритету
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // .../target/debug (або .../VendOS.app/Contents/MacOS у релізі)
            candidates.push(exe_dir.to_path_buf());
            // .../target/debug/CrAdapter
            candidates.push(exe_dir.join("CrAdapter"));
            // .../target/debug/../../ — під наш rpath @executable_path/../..
            if let Some(up2) = exe_dir.parent().and_then(|p| p.parent()) {
                candidates.push(up2.to_path_buf());
            }
        }
    }

    // dev-шляхи з build.rs
    let dev_libs = env!("CRSDK_DEV_LIBS");
    let dev_adapter = env!("CRSDK_DEV_ADAPTER");
    candidates.push(PathBuf::from(dev_libs));
    candidates.push(PathBuf::from(dev_adapter));

    // залишимо тільки наявні директорії та приберемо дублікати
    let mut uniq: Vec<String> = Vec::new();
    for p in candidates {
        if p.is_dir() {
            let s = p.to_string_lossy().into_owned();
            if !uniq.iter().any(|x| x == &s) {
                uniq.push(s);
            }
        }
    }
    if uniq.is_empty() {
        return; // нічого додавати
    }
    let joined = uniq.join(":");

    // DYLD_LIBRARY_PATH
    match env::var("DYLD_LIBRARY_PATH") {
        Ok(cur) if !cur.is_empty() => {
            env::set_var("DYLD_LIBRARY_PATH", format!("{joined}:{cur}"));
        }
        _ => env::set_var("DYLD_LIBRARY_PATH", &joined),
    }

    // і страховка: DYLD_FALLBACK_LIBRARY_PATH
    match env::var("DYLD_FALLBACK_LIBRARY_PATH") {
        Ok(cur) if !cur.is_empty() => {
            env::set_var("DYLD_FALLBACK_LIBRARY_PATH", format!("{joined}:{cur}"));
        }
        _ => env::set_var("DYLD_FALLBACK_LIBRARY_PATH", &joined),
    }
}

#[cfg(not(target_os = "macos"))]
fn setup_crsdk_dylib_search_paths() { /* noop на інших ОС */ }

// demo з шаблону
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn camera_set_save_dir_cmd(directory_path: String) -> Result<(), String> {
    if crsdk_set_save_dir(&directory_path) { Ok(()) } else { Err(format!("SetSaveInfo failed (last_error={} {})", crsdk_last_error(), crsdk_last_error_text())) }
}

#[tauri::command]
fn camera_capture_and_get_path_cmd(timeout_ms: u32) -> Result<String, String> {
    let saved = crsdk_capture_blocking(timeout_ms);
    if saved.is_empty() {
        Err(format!("Capture timeout or error (last_error={} {})", crsdk_last_error(), crsdk_last_error_text()))
    } else {
        Ok(saved)
    }
}

#[tauri::command]
fn camera_liveview_frame_cmd() -> Result<Vec<u8>, String> {
    let bytes = crsdk_liveview_frame();
    if bytes.is_empty() {
        Err(format!("No frame (last_error={} {})", crsdk_last_error(), crsdk_last_error_text()))
    } else {
        Ok(bytes)
    }
}

#[tauri::command]
fn camera_is_connected_cmd() -> bool {
    crsdk_is_connected()
}

#[cfg(target_family = "unix")]
fn parse_lpstat(output: &str) -> (Vec<String>, Option<String>) {
    let mut printers = Vec::new();
    let mut default_printer = None;

    for line in output.lines() {
        if let Some(name) = line.strip_prefix("printer ") {
            let name = name.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() { printers.push(name); }
        } else if let Some(def) = line.strip_prefix("system default destination: ") {
            default_printer = Some(def.trim().to_string());
        }
    }
    (printers, default_printer)
}

#[cfg(target_family = "unix")]
#[tauri::command]
fn cups_list_printers_cmd() -> Result<(Vec<String>, Option<String>), String> {
    let out = Command::new("lpstat").arg("-p").arg("-d")
        .output().map_err(|e| format!("lpstat failed: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    Ok(parse_lpstat(&text))
}

#[cfg(target_family = "unix")]
#[tauri::command]
fn cups_print_file_cmd(
    printer: Option<String>,
    file_path: String,
    copies: Option<u32>,
    media: Option<String>,
    fit_to_page: Option<bool>,
) -> Result<(), String> {
    if !Path::new(&file_path).is_file() {
        return Err(format!("file not found: {file_path}"));
    }

    let mut cmd = Command::new("lp");
    if let Some(p) = printer { cmd.arg("-d").arg(p); }
    if let Some(n) = copies { if n>1 { cmd.arg("-n").arg(n.to_string()); } }
    if let Some(m) = media { cmd.arg("-o").arg(format!("media={}", m)); }
    if fit_to_page.unwrap_or(true) {
        cmd.arg("-o").arg("fit-to-page");
    }

    cmd.arg("-o").arg("page-left=0")
        .arg("-o").arg("page-right=0")
        .arg("-o").arg("page-top=0")
        .arg("-o").arg("page-bottom=0");

    cmd.arg(&file_path);

    let out = cmd.output().map_err(|e| format!("lp failed: {e}"))?;
    if out.status.success() { Ok(()) }
    else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("lp error: {err}"))
    }
}

#[cfg(target_family = "windows")]
#[tauri::command]
fn cups_list_printers_cmd() -> Result<(Vec<String>, Option<String>), String> {
    Err("CUPS доступний лише на macOS/Linux".into())
}
#[cfg(target_family = "windows")]
#[tauri::command]
fn cups_print_file_cmd(
    _printer: Option<String>, _file_path: String, _copies: Option<u32>, _media: Option<String>, _fit_to_page: Option<bool>
) -> Result<(), String> {
    Err("CUPS друк доступний лише на macOS/Linux".into())
}

#[tauri::command]
fn camera_connect_by_index_cmd(index: u32) -> Result<(), String> {
    let code = sony_bridge::crsdk_connect_index(index);
    if code == 0 { Ok(()) } else {
        Err(format!("Connect failed (CrError={} {})", crsdk_last_error(), crsdk_last_error_text()))
    }
}

#[tauri::command]
fn camera_connect_cmd() -> Result<(), String> {
    if sony_bridge::crsdk_connect_first_usb() {
        Ok(())
    } else {
        Err(format!("No camera or connect failed (last_error={} {})", crsdk_last_error(), crsdk_last_error_text()))
    }
}

#[cfg(target_os = "macos")]
fn macos_release_ptp() {
    use std::process::Command;
    let _ = Command::new("killall").arg("PTPCamera").output();
}

#[tauri::command]
fn camera_scan_cmd() -> Result<Vec<String>, String> {
    #[cfg(target_os = "macos")]
    macos_release_ptp();

    if !sony_bridge::crsdk_enum_refresh() {
        Err(format!(
            "EnumCameraObjects failed (last_error={} {})",
            crsdk_last_error(), crsdk_last_error_text()
        ))
    } else {
        let n = sony_bridge::crsdk_enum_count();
        let mut out = Vec::new();
        for i in 0..n { out.push(sony_bridge::crsdk_enum_model(i)); }
        Ok(out)
    }
}

#[tauri::command]
fn sdk_init_cmd() -> Result<(), String> {
    if crsdk_init() {
        std::thread::sleep(std::time::Duration::from_millis(300));
        Ok(())
    } else {
        Err("CRSDK init failed".into())
    }
}


#[tauri::command]
fn sdk_version_cmd() -> String {
    // CRSDK дає число, зазвичай у форматі MMmmpp (напр. 1.14.00 -> 11400).
    let v = sony_bridge::crsdk_version_raw();
    let major = v / 10000;
    let minor = (v / 100) % 100;
    let patch = v % 100;
    format!("{major}.{minor:02}.{patch:02} (raw={v})")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1) ДО будь-яких викликів SDK на macOS
    setup_crsdk_dylib_search_paths();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
             greet,
             sdk_init_cmd,
             camera_scan_cmd,
             camera_connect_cmd,
             camera_connect_by_index_cmd,
             camera_is_connected_cmd,
             camera_set_save_dir_cmd,
             sdk_version_cmd,
             camera_capture_and_get_path_cmd,
             camera_liveview_frame_cmd,
             cups_list_printers_cmd,
             cups_print_file_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
