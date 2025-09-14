import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

type LogEntry = { t: string; k: "ok" | "err" | "info" };

export default function App() {
  const [saveDir, setSaveDir] = useState("/Users/andriibabushko/Pictures/VendOS");
  const [imagePath, setImagePath] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [connected, setConnected] = useState(false);
  const [liveSrc, setLiveSrc] = useState<string | null>(null);
  const [isLive, setIsLive] = useState(false);
  const [printers, setPrinters] = useState<string[]>([]);
  const [defaultPrinter, setDefaultPrinter] = useState<string | null>(null);
  const [selectedPrinter, setSelectedPrinter] = useState<string | null>(null);
  const [copies, setCopies] = useState(1);
  const [media, setMedia] = useState("4x6in"); // типова RX1 — 4x6
  const [fitToPage, setFitToPage] = useState(true);
  const [foundCams, setFoundCams] = useState<string[]>([]);
  const [camIndex, setCamIndex] = useState<number>(0);

  async function scanCams() {
    try {
      const list = await invoke<string[]>("camera_scan_cmd");
      setFoundCams(list);
      log("ok", list.length ? `Знайдено камери: ${list.join(", ")}` : "Камери не знайдено");
    } catch (e:any) { log("err", String(e)); }
  }

  async function connectByIndex() {
    try {
      await invoke("camera_connect_by_index_cmd", { index: camIndex });
      setConnected(true);
      log("ok", `Під’єднано: ${foundCams[camIndex] ?? "(невідомо)"}`);
    } catch (e:any) { log("err", String(e)); }
  }

  async function loadPrinters() {
    try {
      const [list, def] = await invoke<[string[], string|null]>("cups_list_printers_cmd");
      setPrinters(list);
      setDefaultPrinter(def);
      if (!selectedPrinter) setSelectedPrinter(def ?? (list[0] ?? null));
      log("ok", `Принтери: ${list.join(", ")}; default=${def ?? "—"}`);
    } catch (e:any) { log("err", "lpstat: " + e); }
  }

  async function printLast() {
    if (!imagePath) { log("err","Нема що друкувати"); return; }
    try {
      await invoke("cups_print_file_cmd", {
        printer: selectedPrinter,
        filePath: imagePath,
        copies,
        media,
        fitToPage
      });
      log("ok", `Відправлено у друк (${selectedPrinter ?? defaultPrinter ?? "за замовчуванням"})`);
    } catch (e:any) {
      log("err", "Друк: " + e);
    }
  }

  function log(k: LogEntry["k"], t: string) {
    setLogs((l) => [{ k, t }, ...l].slice(0, 200));
  }

  async function initAndConnect() {
    try {
      await invoke("sdk_init_cmd");
      log("ok", "SDK ініціалізовано");
      await invoke("camera_connect_cmd");
      log("ok", "Камеру під’єднано");
      await invoke("camera_set_save_dir_cmd", { directoryPath: saveDir });
      log("ok", "Директорія збереження: " + saveDir);
      const c = await invoke<boolean>("camera_is_connected_cmd");
      setConnected(c);
    } catch (e: any) {
      log("err", String(e));
    }
  }

  async function checkConnected() {
    const c = await invoke<boolean>("camera_is_connected_cmd");
    setConnected(c);
    log(c ? "ok" : "err", c ? "Камера під’єднана" : "Камеру не знайдено");
  }

  async function capture() {
    setIsBusy(true);
    try {
      const path = await invoke<string>("camera_capture_and_get_path_cmd", { timeoutMs: 15000 });
      setImagePath(path);
      log("ok", "Фото збережено: " + path);
    } catch (e: any) {
      log("err", "Помилка зйомки: " + e);
    } finally {
      setIsBusy(false);
    }
  }

  async function getVersion() {
    const version = await invoke<string>("sdk_version_cmd")
    log(version ? "ok" : "err", version ? `Версія SonySDK: ${version}` : "Помилка при отриманні версії sony SDK");
  }

  // простий live-view (поллінг раз на 150мс)
  useEffect(() => {
    if (!isLive) return;
    let stop = false;
    async function tick() {
      try {
        const bytes = await invoke<Uint8Array>("camera_liveview_frame_cmd");
        if (bytes && (bytes as any).length > 0) {
          const blob = new Blob([new Uint8Array(bytes as any)], { type: "image/jpeg" });
          const url = URL.createObjectURL(blob);
          setLiveSrc((old) => {
            if (old) URL.revokeObjectURL(old);
            return url;
          });
        }
      } catch (e) {
        log("err", "liveview: " + e);
      }
      if (!stop) setTimeout(tick, 150);
    }
    tick();
    return () => {
      stop = true;
      setLiveSrc((old) => { if (old) URL.revokeObjectURL(old); return null; });
    };
  }, [isLive]);

  return (
    <>
      <div style={{ padding: 20, fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial" }}>
        <h1>VendOS • Camera Test</h1>

        <div style={{ display: "grid", gap: 12, maxWidth: 760 }}>
          <div>
            <b>Статус:</b>{" "}
            <span style={{ color: connected ? "#0a7" : "#b00020" }}>
              {connected ? "Під’єднано" : "Не під’єднано"}
            </span>
          </div>

          <label>
            Директорія збереження:
            <input
              style={{ width: "100%", padding: 8, marginTop: 6 }}
              value={saveDir}
              onChange={(e) => setSaveDir(e.target.value)}
            />
          </label>

          <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
            <button onClick={initAndConnect}>Ініціалізувати + Під’єднати</button>
            <button onClick={checkConnected}>Перевірити під’єднання</button>
            <button onClick={getVersion}>Отримати версію sonySDK</button>
            <button onClick={() => setIsLive((v) => !v)} disabled={!connected}>
              {isLive ? "Стоп Live-view" : "Старт Live-view"}
            </button>
            <button onClick={capture} disabled={!connected || isBusy}>
              {isBusy ? "Знімаю..." : "Зняти фото"}
            </button>
          </div>


          {imagePath && (
            <div style={{ marginTop: 8 }}>
              <div>Останнє фото: {imagePath}</div>
              <img
                src={convertFileSrc(imagePath)}
                style={{ maxWidth: 640, border: "1px solid #ddd", borderRadius: 8, marginTop: 6 }}
              />
            </div>
          )}

          {liveSrc && (
            <div style={{ marginTop: 8 }}>
              <div>Live-view</div>
              <img src={liveSrc} style={{ maxWidth: 640, border: "1px solid #ddd", borderRadius: 8, marginTop: 6 }} />
            </div>
          )}

          <div style={{ marginTop: 12 }}>
            <div>Логи</div>
            <ul style={{ listStyle: "none", padding: 0, maxHeight: 220, overflow: "auto", background: "#fafafa", border: "1px solid #eee" }}>
              {logs.map((l, i) => (
                <li key={i} style={{ padding: "6px 8px", borderBottom: "1px solid #eee", color: l.k === "err" ? "#b00020" : l.k === "ok" ? "#0a7" : "#555" }}>
                  {l.t}
                </li>
              ))}
            </ul>
          </div>
        </div>

        <div style={{ marginTop: 16, paddingTop: 12, borderTop: "1px solid #eee" }}>
          <h2 style={{ margin: 0, fontSize: 18 }}>Друк (CUPS)</h2>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            <button onClick={loadPrinters}>Завантажити принтери</button>
            <select
              value={selectedPrinter ?? ""}
              onChange={(e) => setSelectedPrinter(e.target.value || null)}
            >
              {printers.map(p => <option key={p} value={p}>{p}</option>)}
              {(!printers.length) && <option value="">— немає —</option>}
            </select>

            <label>Копій:
              <input type="number" min={1} value={copies} onChange={e => setCopies(+e.target.value || 1)}
                     style={{ width: 70, marginLeft: 6 }} />
            </label>

            <label>Розмір:
              <select value={media} onChange={e => setMedia(e.target.value)} style={{ marginLeft: 6 }}>
                <option value="4x6in">4x6in</option>
                <option value="5x7in">5x7in</option>
                <option value="6x8in">6x8in</option>
                <option value="Custom.4x6in">Custom.4x6in</option>
              </select>
            </label>

            <label style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
              <input type="checkbox" checked={fitToPage} onChange={e => setFitToPage(e.target.checked)} />
              Підігнати під сторінку
            </label>

            <button onClick={printLast} disabled={!imagePath}>Друкувати останнє фото</button>
          </div>

          {defaultPrinter && (
            <div style={{ fontSize: 12, color: "#666", marginTop: 6 }}>
              Принтер за замовчуванням системи: {defaultPrinter}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
