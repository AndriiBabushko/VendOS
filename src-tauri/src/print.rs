use serde::Serialize;
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Check that CUPS is available (`lpstat -r`).
pub fn ensure_cups() -> Result<(), String> {
    let out = Command::new("lpstat")
        .arg("-r")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("lpstat not found: {e}"))?;
    if !out.status.success() {
        return Err("CUPS scheduler not running".into());
    }
    Ok(())
}

#[derive(Serialize)]
pub struct CupsPrinter {
    pub name: String,
    pub is_default: bool,
    pub state: String,
    pub description: String,
}

/// List printers (marks default if present).
pub fn list_printers() -> Result<Vec<CupsPrinter>, String> {
    let p = Command::new("lpstat")
        .arg("-p")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("lpstat -p: {e}"))?;

    // `lpstat -d` may fail if no default is set — treat as None.
    let default_name: Option<String> = Command::new("lpstat")
        .arg("-d")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.split(':').nth(1).map(|t| t.trim().to_string())
        });

    let mut out_list = Vec::new();
    for line in String::from_utf8_lossy(&p.stdout).lines() {
        if !line.starts_with("printer ") {
            continue;
        }
        // format: "printer NAME is idle.  enabled since ..."
        let mut parts = line.split_whitespace();
        let _ = parts.next();
        let name = parts.next().unwrap_or("").to_string();
        let state = if line.contains("is idle") {
            "idle"
        } else if line.contains("now printing") {
            "printing"
        } else {
            "unknown"
        }
            .to_string();

        out_list.push(CupsPrinter {
            is_default: default_name
                .as_ref()
                .map(|n| n == &name)
                .unwrap_or(false),
            name,
            state,
            description: String::new(),
        });
    }
    Ok(out_list)
}

/// Raw `lpoptions -p <printer> -l` lines (UI can parse allowed values).
pub fn printer_options(printer: &str) -> Result<Vec<String>, String> {
    let o = Command::new("lpoptions")
        .args(["-p", printer, "-l"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("lpoptions: {e}"))?;
    if !o.status.success() {
        return Err(format!(
            "lpoptions exit {}",
            o.status.code().unwrap_or(-1)
        ));
    }
    Ok(String::from_utf8_lossy(&o.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .collect())
}

/// Print file: `lp -d <printer> -n <copies> -o k=v ... <path>`.
pub fn print_file(
    printer: &str,
    path: &str,
    copies: u32,
    options: Option<HashMap<String, String>>,
) -> Result<(), String> {
    let mut args: Vec<String> = vec![
        "-d".into(),
        printer.into(),
        "-n".into(),
        copies.to_string(),
    ];
    if let Some(opts) = options {
        for (k, v) in opts {
            args.push("-o".into());
            args.push(format!("{k}={v}"));
        }
    }
    args.push(path.into());

    let out = Command::new("lp")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("lp spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "lp exit {}: {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}
