use std::{fs, path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};

fn ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn ensure_tool() -> Result<(), String> {
    Command::new("gphoto2").arg("--version")
        .status().map_err(|e| format!("gphoto2 not found: {e}"))?
        .success().then_some(()).ok_or("gphoto2 failed to run".into())
}

pub fn auto_detect() -> Result<Vec<String>, String> {
    let out = Command::new("gphoto2").arg("--auto-detect")
        .output().map_err(|e| format!("detect failed: {e}"))?;
    if !out.status.success() { return Err(format!("detect exit {}", out.status)); }
    let s = String::from_utf8_lossy(&out.stdout);
    let mut res = Vec::new();
    // пропускаємо заголовок із тире
    let mut sep_seen = false;
    for line in s.lines() {
        if !sep_seen { sep_seen = line.starts_with("---"); continue; }
        let l = line.trim();
        if !l.is_empty() { res.push(l.to_string()); }
    }
    Ok(res)
}

pub fn capture_to(dir: &str) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("mkdir: {e}"))?;
    let t = ts();
    let pattern = format!("{dir}/shot-{t}.%C");
    let status = Command::new("gphoto2")
        // зберігати на карту + скачати
        .args(["--set-config", "capturetarget=1"])
        .args(["--capture-image-and-download", "--force-overwrite", "--keep"])
        .args(["--filename", &pattern])
        // стабілізуємо локаль, щоб поведінка була передбачувана
        .env("LANG", "C")
        .status().map_err(|e| format!("spawn gphoto2: {e}"))?;
    if !status.success() { return Err(format!("gphoto2 exit status {status}")); }

    // gphoto2 підставляє розширення у %C — знаходимо створений файл
    let prefix = format!("shot-{t}.");
    let mut found: Option<PathBuf> = None;
    for e in fs::read_dir(dir).map_err(|e| format!("read_dir: {e}"))? {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with(&prefix) { found = Some(e.path()); break; }
    }
    let p = found.ok_or_else(|| "capture ok, але файл не знайдено".to_string())?;
    Ok(p.to_string_lossy().to_string())
}
