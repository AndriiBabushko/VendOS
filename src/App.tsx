import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { appDataDir, join } from "@tauri-apps/api/path";

type LogEntry = { level: "ok" | "err" | "info"; message: string };

type DetectedDevice = { label: string; model: string; port?: string };
type CupsPrinter = { name: string; is_default: boolean; state: string; description: string };

function parseAutoDetectLines(lines: string[]): DetectedDevice[] {
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

function buildRx1Options(size: "4x6" | "6x8" | "5x7", cut: "Normal" | "2Inch" | "NoWaste") {
  const PageSize = size === "4x6" ? "300dnp6x4"
    : size === "6x8" ? "310dnp6x8"
      : "210dnp5x7"; // 5x7 — сімейство 210
  return {
    PageSize,
    Cutter: cut,             // "Normal" = одна суцільна; "2Inch" = 2×6 смужки; "NoWaste" = дві 4×6 з 6×8
    Finish: "Glossy",
    Resolution: "300x300dpi",
    ColorModel: "RGB",
    BonusPrint: "False",
    PrintRetry: "True",
  } as Record<string,string>;
}


export default function App() {
  const [detected, setDetected] = useState<DetectedDevice[]>([]);
  const [selectedPort, setSelectedPort] = useState<string | "">("");
  const [saveDir, setSaveDir] = useState<string>("");
  const [isBusy, setIsBusy] = useState(false);
  const [lastImagePath, setLastImagePath] = useState<string | null>(null);
  const [lastImageUrl, setLastImageUrl] = useState<string | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const [printers, setPrinters] = useState<CupsPrinter[]>([]);
  const [selectedPrinter, setSelectedPrinter] = useState<string>("");
  const [copies, setCopies] = useState<number>(1);
  const [size, setSize] = useState<"4x6" | "6x8" | "5x7">("4x6");
  const [cutMode, setCutMode] = useState<"Normal" | "2Inch" | "NoWaste">("Normal");

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
      if (devices.length === 1 && devices[0].port) setSelectedPort(devices[0].port!);
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
      } catch {
        const base64 = await invoke<string>("fs_read_base64_cmd", { path });
        setLastImageUrl(`data:image/jpeg;base64,${base64}`);
      }
      log("ok", `Captured: ${path}`);
    } catch (e: any) {
      log("err", `Capture error: ${String(e)}`);
    } finally {
      setIsBusy(false);
    }
  }

  async function refreshPrinters() {
    try {
      const list = await invoke<CupsPrinter[]>("cups_list_printers_cmd");
      setPrinters(list);
      const def = list.find(p => p.is_default)?.name;
      if (def) setSelectedPrinter(def);
      // fallback to your target queue name if present in the system:
      const target = list.find(p => p.name.includes("termosublmatsyniy-fotoprinter-mitsubishi-cp-k60dw-s-IDYeKHF"));
      if (target) setSelectedPrinter(target.name);
      log(list.length ? "ok" : "info", list.length ? `Printers: ${list.map(p=>p.name).join(", ")}` : "No CUPS printers");
    } catch (e: any) {
      log("err", `CUPS: ${String(e)}`);
    }
  }

  async function printLast() {
    if (!lastImagePath) return log("info", "No image to print");
    if (!selectedPrinter) return log("info", "Select a printer");

    // Визначаємо RX1 по імені (можеш зробити точніше через PPD)
    const isRx1 = /Dai[_\s]Nippon[_\s]Printing[_\s-]?DS-?RX1/i.test(selectedPrinter);

    const options = isRx1
      ? buildRx1Options(size, cutMode)
      : { /* non-RX1 fallback */
        // якщо друк під інші принтери — твоя стара логіка
        // media/fit-to-page можуть знадобитись там
      };

    try {
      await invoke("cups_print_file_cmd", {
        printer: selectedPrinter,
        path: lastImagePath,
        copies,
        options,
      });
      log("ok", `Sent: ${selectedPrinter} (copies=${copies}, size=${size}, cut=${cutMode})`);
    } catch (e: any) {
      log("err", `Print error: ${String(e)}`);
    }
  }


  useEffect(() => { refreshPrinters(); }, []);

  return (
    <div style={{ padding: 20, fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial", maxWidth: 860 }}>
      <h1 style={{ marginTop: 0 }}>VendOS • Camera (gphoto2)</h1>

      <section style={{ display: "grid", gap: 12 }}>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <button onClick={detectCameras}>Detect cameras</button>
          <select value={selectedPort} onChange={(e) => setSelectedPort(e.target.value)} disabled={!detected.length}>
            <option value="">Auto (first device)</option>
            {detected.filter(d=>!!d.port).map((d, i) => (
              <option key={i} value={d.port!}>{d.port} — {d.model}</option>
            ))}
          </select>
          <button onClick={captureOnce} disabled={isBusy}>{isBusy ? "Capturing…" : "📸 Capture"}</button>
        </div>

        <label>
          Save directory
          <input style={{ width: "100%", padding: 8, marginTop: 6 }} value={saveDir} onChange={(e) => setSaveDir(e.target.value)} placeholder="Auto → appDataDir()/captures" />
        </label>

        {lastImageUrl && (
          <div style={{ marginTop: 8 }}>
            <div>Last image: {lastImagePath}</div>
            <img src={lastImageUrl} style={{ maxWidth: 640, border: "1px solid #ddd", borderRadius: 8, marginTop: 6 }} />
          </div>
        )}

        {/* Printing */}
        <div style={{ display: "grid", gap: 8, marginTop: 16, paddingTop: 12, borderTop: "1px solid #eee" }}>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <button onClick={refreshPrinters}>Refresh printers</button>
            <select value={selectedPrinter} onChange={(e)=>setSelectedPrinter(e.target.value)} disabled={!printers.length}>
              <option value="">Select printer</option>
              {printers.map((p)=>(
                <option key={p.name} value={p.name}>
                  {p.is_default ? "⭐ " : ""}{p.name} ({p.state})
                </option>
              ))}
            </select>
            <input type="number" min={1} value={copies} onChange={(e)=>setCopies(Number(e.target.value)||1)} style={{ width: 80 }} />
            <select value={size} onChange={(e)=>setSize(e.target.value as any)}>
              <option value="4x6">4×6 (300)</option>
              <option value="6x8">6×8 (310)</option>
              <option value="5x7">5×7 (210)</option>
            </select>
            <select value={cutMode} onChange={(e)=>setCutMode(e.target.value as any)}>
              <option value="Normal">Cut: Normal (one full)</option>
              <option value="2Inch">Cut: 2Inch (2×6 strips)</option>
              <option value="NoWaste">Cut: NoWaste (two 4×6 from 6×8)</option>
            </select>
            <button onClick={printLast} disabled={!lastImagePath}>🖨️ Print</button>
          </div>
        </div>

        <div style={{ marginTop: 12 }}>
          <div>Logs</div>
          <ul style={{ listStyle: "none", padding: 0, maxHeight: 220, overflow: "auto", background: "#fafafa", border: "1px solid #eee" }}>
            {logs.map((l, i) => (
              <li key={i} style={{ padding: "6px 8px", borderBottom: "1px solid #eee", color: l.level === "err" ? "#b00020" : l.level === "ok" ? "#0a7" : "#555" }}>
                {l.message}
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
