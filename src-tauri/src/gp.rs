use once_cell::sync::Lazy;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

/// Serialize gphoto2 calls (the CLI is not concurrency-safe).
static CAPTURE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn unix_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Run gphoto2 with predictable output; returns (exit_code, stdout, stderr).
fn run_gphoto<I, S>(args: I) -> Result<(i32, String, String), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = Command::new("gphoto2")
        .args(args)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("spawn gphoto2: {e}"))?;

    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    ))
}

/// Ensure gphoto2 is installed and executable.
pub fn ensure_tool() -> Result<(), String> {
    Command::new("gphoto2")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("gphoto2 not found: {e}"))?
        .success()
        .then_some(())
        .ok_or("gphoto2 failed to run".into())
}

/// Return raw `--auto-detect` rows after the header.
pub fn auto_detect() -> Result<Vec<String>, String> {
    let (_c, so, _se) = run_gphoto(["--auto-detect"])?;
    let mut sep = false;
    let mut out = Vec::new();
    for line in so.lines() {
        let t = line.trim();
        if !sep {
            if t.starts_with("---") {
                sep = true;
            }
            continue;
        }
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }
    Ok(out)
}

/// Prefer JPG/PNG if both RAW+JPG were saved.
fn prefer_displayable(files: &[PathBuf]) -> Option<&PathBuf> {
    for ext in ["jpg", "jpeg", "png"] {
        if let Some(p) = files.iter().find(|p| {
            p.extension()
                .and_then(OsStr::to_str)
                .map(|e| e.eq_ignore_ascii_case(ext))
                .unwrap_or(false)
        }) {
            return Some(p);
        }
    }
    files.first()
}

/// Best-effort `capturetarget=Memory card` if camera supports it.
fn try_set_capture_target_card(port: Option<&str>) {
    let mut get_args = vec!["--get-config", "capturetarget"];
    if let Some(p) = port {
        get_args.extend(["--port", p]);
    }
    if let Ok((_c, so, _se)) = run_gphoto(get_args) {
        let desired = ["Memory card", "Card", "SD", "Memory card(1)"];
        let choices: Vec<(i32, String)> = so
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                if !l.starts_with("Choice:") {
                    return None;
                }
                let mut parts = l.split_whitespace();
                let _ = parts.next()?; // "Choice:"
                let idx = parts.next()?.parse::<i32>().ok()?;
                let label = parts.collect::<Vec<_>>().join(" ");
                Some((idx, label))
            })
            .collect();

        if let Some((idx, _)) = choices
            .iter()
            .find(|(_, label)| desired.iter().any(|d| label.to_lowercase().contains(&d.to_lowercase())))
        {
            let cfg = format!("capturetarget={idx}");
            let mut set_args: Vec<String> = vec!["--set-config".into(), cfg];
            if let Some(p) = port {
                set_args.push("--port".into());
                set_args.push(p.to_string());
            }
            let _ = run_gphoto(set_args);
        }
    }
}

/// Capture into `save_dir`; returns a displayable image path (or RAW preview).
pub fn capture_to(save_dir: &str, port: Option<&str>) -> Result<String, String> {
    let _guard = CAPTURE_LOCK.lock().unwrap();

    fs::create_dir_all(save_dir).map_err(|e| format!("mkdir: {e}"))?;
    let t = unix_ts();
    let prefix = format!("shot-{t}.");
    let pattern = format!("{save_dir}/shot-{t}.%C");

    try_set_capture_target_card(port);

    let mut args: Vec<String> = vec![
        "--capture-image-and-download".into(),
        "--force-overwrite".into(),
        "--keep".into(),
        "--filename".into(),
        pattern.clone(),
    ];
    if let Some(p) = port {
        args.push("--port".into());
        args.push(p.to_string());
    }
    let (code, _so, se) = run_gphoto(args)?;
    if code != 0 {
        return Err(format!("gphoto2 exit {code}: {se}"));
    }

    let mut candidates: Vec<PathBuf> = fs::read_dir(save_dir)
        .map_err(|e| format!("read_dir: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(OsStr::to_str)
                .map(|n| n.starts_with(&prefix))
                .unwrap_or(false)
        })
        .collect();
    candidates.sort();

    let chosen = candidates
        .is_empty()
        .then(|| None)
        .unwrap_or_else(|| prefer_displayable(&candidates))
        .cloned()
        .ok_or_else(|| "capture succeeded, but file was not found".to_string())?;

    let displayable = chosen
        .extension()
        .and_then(OsStr::to_str)
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        .unwrap_or(false);

    if !displayable {
        let preview = format!("{save_dir}/shot-{t}.preview.jpg");
        let mut preview_args: Vec<String> = vec![
            "--capture-preview".into(),
            "--force-overwrite".into(),
            "--filename".into(),
            preview.clone(),
        ];
        if let Some(p) = port {
            preview_args.push("--port".into());
            preview_args.push(p.to_string());
        }
        if let Ok((c, _so, _se)) = run_gphoto(preview_args) {
            if c == 0 && Path::new(&preview).exists() {
                return Ok(preview);
            }
        }
    }

    Ok(chosen.to_string_lossy().to_string())
}
