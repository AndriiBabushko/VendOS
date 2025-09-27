import { useState } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir, join } from "@tauri-apps/api/path";

type LogEntry = { level: "ok" | "err" | "info"; message: string };

type DetectedDevice = {
  label: string;   // full line after --auto-detect header
  model: string;   // extracted model
  port?: string;   // extracted port like "usb:001,005" (if present)
};

function parseAutoDetectLines(lines: string[]): DetectedDevice[] {
  // Lines are like: "Sony Alpha ...             usb:001,007"
  // We split by two+ spaces; last token might be a port.
  return lines.map((raw) => {
    const parts = raw.split(/\s{2,}/).map((s) => s.trim()).filter(Boolean);
    let model = raw.trim();
    let port: string | undefined;

    if (parts.length >= 2) {
      model = parts.slice(0, parts.length - 1).join(" ");
      const last = parts[parts.length - 1];
      if (/^usb:\d+,\d+$/.test(last)) port = last;
    }
    return { label: raw, model, port };
  });
}

export default function App() {
  const [detected, setDetected] = useState<DetectedDevice[]>([]);
  const [selectedPort, setSelectedPort] = useState<string | "">("");
  const [saveDir, setSaveDir] = useState<string>("");
  const [isBusy, setIsBusy] = useState(false);
  const [lastImagePath, setLastImagePath] = useState<string | null>(null);
  const [lastImageUrl, setLastImageUrl] = useState<string|null>(null)
  const [logs, setLogs] = useState<LogEntry[]>([]);

  function log(level: LogEntry["level"], message: string) {
    setLogs((l) => [{ level, message }, ...l].slice(0, 200));
  }

  async function ensureSaveDir() {
    if (saveDir) return saveDir;
    const base = await appDataDir();
    const dir = await join(base, "captures");
    setSaveDir(dir);
    return dir;
  }

  async function detectCameras() {
    try {
      const lines = await invoke<string[]>("gp_detect_cmd");
      const devices = parseAutoDetectLines(lines);
      setDetected(devices);
      if (devices.length === 1 && devices[0].port) {
        setSelectedPort(devices[0].port!);
      }
      log(devices.length ? "ok" : "info", devices.length ? `Detected: ${devices.map(d => d.label).join(" | ")}` : "No cameras detected");
    } catch (e: any) {
      log("err", `Detect error: ${String(e)}`);
    }
  }

  async function captureOnce() {
    setIsBusy(true);
    try {
      const dir = await ensureSaveDir();
      const port = selectedPort || undefined;
      const path = await invoke<string>("gp_capture_cmd", { saveDir: dir, port });
      setLastImagePath(path);

      const asset = convertFileSrc(path);
      try {
        const res = await fetch(asset, { cache: "no-store" });
        if (!res.ok) throw new Error(`asset fetch ${res.status}`);
        setLastImageUrl(asset);
      } catch (e) {
        console.warn("asset:// failed, falling back to data:URI", e);
        const base64 = await invoke<string>("fs_read_base64_cmd", { path });
        setLastImageUrl(`data:image/jpeg;base64,${base64}`);
      }
    } catch (e: any) {
      log("err", `Capture error: ${String(e)}`);
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <div style={{ padding: 20, fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial", maxWidth: 860 }}>
      <h1 style={{ marginTop: 0 }}>VendOS • Camera (gphoto2)</h1>

      <section style={{ display: "grid", gap: 12 }}>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <button onClick={detectCameras}>Detect cameras</button>

          <select
            value={selectedPort}
            onChange={(e) => setSelectedPort(e.target.value)}
            disabled={!detected.length}
          >
            <option value="">Auto (first device)</option>
            {detected
              .filter(d => !!d.port)
              .map((d, i) => (
                <option key={i} value={d.port!}>
                  {d.port} — {d.model}
                </option>
              ))}
          </select>

          <button onClick={captureOnce} disabled={isBusy}>
            {isBusy ? "Capturing…" : "📸 Capture"}
          </button>
        </div>

        <label>
          Save directory
          <input
            style={{ width: "100%", padding: 8, marginTop: 6 }}
            value={saveDir}
            onChange={(e) => setSaveDir(e.target.value)}
            placeholder="Auto → appDataDir()/captures"
          />
        </label>

        {lastImageUrl && (
          <div style={{ marginTop: 8 }}>
            <div>Last image: {lastImagePath}</div>
            <img src={lastImageUrl} style={{ maxWidth: 640, border: "1px solid #ddd", borderRadius: 8, marginTop: 6 }} />
          </div>
        )}

        <div style={{ marginTop: 12 }}>
          <div>Logs</div>
          <ul
            style={{
              listStyle: "none",
              padding: 0,
              maxHeight: 220,
              overflow: "auto",
              background: "#fafafa",
              border: "1px solid #eee",
            }}
          >
            {logs.map((l, i) => (
              <li
                key={i}
                style={{
                  padding: "6px 8px",
                  borderBottom: "1px solid #eee",
                  color: l.level === "err" ? "#b00020" : l.level === "ok" ? "#0a7" : "#555",
                }}
              >
                {l.message}
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
