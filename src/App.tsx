import { useEffect, useState } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import {appDataDir, join} from "@tauri-apps/api/path";

type LogEntry = { t: string; k: "ok" | "err" | "info" };

export default function App() {
  const [sdkReady, setSdkReady] = useState(false);
  const [sdkVersion, setSdkVersion] = useState<string | null>(null);

  const [foundCams, setFoundCams] = useState<string[]>([]);
  const [camIndex, setCamIndex] = useState(0);
  const [connected, setConnected] = useState(false);

  const [saveDir, setSaveDir] = useState("/tmp/VendOS");
  const [imagePath, setImagePath] = useState<string | null>(null);

  const [isBusy, setIsBusy] = useState(false);
  const [isLive, setIsLive] = useState(false);
  const [liveSrc, setLiveSrc] = useState<string | null>(null);

  const [logs, setLogs] = useState<LogEntry[]>([]);

  function log(k: LogEntry["k"], t: string) {
    setLogs((l) => [{ k, t }, ...l].slice(0, 200));
  }

  async function snap() {
    const base = await appDataDir();
    const dir = await join(base, "captures");
    const savedPath = await invoke<string>("gp_capture_cmd", { saveDir: dir });
    const url = convertFileSrc(savedPath);
    console.log(`url -> ${url}`)
  }

  async function initSdk() {
    try {
      await invoke("sdk_init_cmd");
      setSdkReady(true);
      const v = await invoke<string>("sdk_version_cmd");
      setSdkVersion(v);
      log("ok", `SDK ініціалізовано (${v})`);
    } catch (e: any) {
      setSdkReady(false);
      log("err", `SDK init: ${String(e)}`);
    }
  }

  async function scanCams() {
    try {
      const list = await invoke<string[]>("camera_scan_cmd");
      setFoundCams(list);
      log(list.length ? "ok" : "info", list.length ? `Знайдено: ${list.join(", ")}` : "Камери не знайдено");
    } catch (e: any) {
      log("err", `Scan: ${String(e)}`);
    }
  }

  async function connectByIndex() {
    try {
      await invoke("camera_connect_by_index_cmd", { index: camIndex });
      setConnected(true);
      log("ok", `Під’єднано: ${foundCams[camIndex] ?? "(невідомо)"}`);
    } catch (e: any) {
      setConnected(false);
      log("err", `Connect: ${String(e)}`);
    }
  }

  async function applySaveDir() {
    try {
      await invoke("camera_set_save_dir_cmd", { directoryPath: saveDir });
      log("ok", "Директорія збереження встановлена");
    } catch (e: any) {
      log("err", `SetSaveDir: ${String(e)}`);
    }
  }

  async function capture() {
    setIsBusy(true);
    try {
      const path = await invoke<string>("camera_capture_and_get_path_cmd", { timeoutMs: 15000 });
      setImagePath(path);
      log("ok", `Фото: ${path}`);
    } catch (e: any) {
      log("err", `Capture: ${String(e)}`);
    } finally {
      setIsBusy(false);
    }
  }

  // простий поллінг live-view (JPEG), ~6–7 fps
  useEffect(() => {
    if (!isLive) return;
    let stop = false;

    async function tick() {
      try {
        const arr = await invoke<number[]>("camera_liveview_frame_cmd");
        if (Array.isArray(arr) && arr.length > 0) {
          const blob = new Blob([new Uint8Array(arr)], { type: "image/jpeg" });
          const url = URL.createObjectURL(blob);
          setLiveSrc((old) => {
            if (old) URL.revokeObjectURL(old);
            return url;
          });
        }
      } catch (e) {
        log("err", `Live-view: ${String(e)}`);
      }
      if (!stop) setTimeout(tick, 150);
    }
    tick();
    return () => {
      stop = true;
      setLiveSrc((old) => {
        if (old) URL.revokeObjectURL(old);
        return null;
      });
    };
  }, [isLive]);

  async function checkConnected() {
    try {
      const c = await invoke<boolean>("camera_is_connected_cmd");
      setConnected(c);
      log(c ? "ok" : "info", c ? "Камера під’єднана" : "Камеру не знайдено");
    } catch (e: any) {
      log("err", `is_connected: ${String(e)}`);
    }
  }

  return (
    <div style={{ padding: 20, fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial", maxWidth: 860 }}>
      <h1 style={{ marginTop: 0 }}>VendOS • Linux Camera</h1>

      <section style={{ display: "grid", gap: 10 }}>
        <div>
          <b>SDK:</b>{" "}
          <span style={{ color: sdkReady ? "#0a7" : "#b00020" }}>
            {sdkReady ? `готовий${sdkVersion ? ` (${sdkVersion})` : ""}` : "не ініціалізовано"}
          </span>
        </div>

        <div>
          <b>Статус камери:</b>{" "}
          <span style={{ color: connected ? "#0a7" : "#b00020" }}>
            {connected ? "Під’єднано" : "Не під’єднано"}
          </span>
        </div>

        <div style={{display: "flex", gap: 8, flexWrap: "wrap"}}>
          <button onClick={initSdk}>Ініціалізувати SDK</button>
          <button onClick={scanCams} disabled={!sdkReady}>Сканувати</button>
          <select
            value={camIndex}
            onChange={(e) => setCamIndex(Number(e.target.value))}
            disabled={!foundCams.length}
          >
            {foundCams.map((c, i) => (
              <option key={i} value={i}>{`${i}: ${c}`}</option>
            ))}
          </select>
          <button onClick={connectByIndex} disabled={!foundCams.length}>Під’єднати</button>
          <button onClick={checkConnected}>Перевірити</button>
          <button onClick={snap}>📸 Зняти</button>
        </div>

        <label>
          Директорія збереження:
          <input
            style={{width: "100%", padding: 8, marginTop: 6 }}
            value={saveDir}
            onChange={(e) => setSaveDir(e.target.value)}
            placeholder="/tmp/VendOS"
          />
        </label>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <button onClick={applySaveDir} disabled={!connected}>Застосувати</button>
          <button onClick={() => setIsLive((v) => !v)} disabled={!connected}>
            {isLive ? "Стоп Live-view" : "Старт Live-view"}
          </button>
          <button onClick={capture} disabled={!connected || isBusy}>
            {isBusy ? "Знімаю…" : "Зняти фото"}
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
            <img
              src={liveSrc}
              style={{ maxWidth: 640, border: "1px solid #ddd", borderRadius: 8, marginTop: 6 }}
            />
          </div>
        )}

        <div style={{ marginTop: 12 }}>
          <div>Логи</div>
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
                  color: l.k === "err" ? "#b00020" : l.k === "ok" ? "#0a7" : "#555",
                }}
              >
                {l.t}
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  );
}
