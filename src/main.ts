import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

type Settings = { ratio: [number, number]; cameraId: string | null };

const MIN_SHORT_SIDE = 135;

const appWindow = getCurrentWindow();
const frame = document.querySelector<HTMLDivElement>("#frame")!;
const video = document.querySelector<HTMLVideoElement>("#cam")!;
const statusEl = document.querySelector<HTMLDivElement>("#status")!;
const closeBtn = document.querySelector<HTMLButtonElement>("#close")!;
const resizeHandle = document.querySelector<HTMLDivElement>("#resize-handle")!;

let stream: MediaStream | null = null;
let ratio: [number, number] = [9, 16];
let desiredCameraId: string | null = null;
let currentCameraId: string | null = null;
let cameraQueue: Promise<void> = Promise.resolve();

function showStatus(message: string) {
  statusEl.textContent = message;
  statusEl.hidden = false;
}

function errorName(error: unknown) {
  return (error as { name?: string } | null)?.name ?? "";
}

function stopCamera() {
  stream?.getTracks().forEach((track) => track.stop());
  stream = null;
  video.srcObject = null;
}

async function openStream(deviceId: string | null) {
  if (deviceId) {
    try {
      return await navigator.mediaDevices.getUserMedia({ video: { deviceId: { exact: deviceId } }, audio: false });
    } catch (error) {
      if (!["OverconstrainedError", "NotFoundError", "NotReadableError"].includes(errorName(error))) throw error;
    }
  }
  return navigator.mediaDevices.getUserMedia({ video: true, audio: false });
}

async function syncCameras() {
  const devices = await navigator.mediaDevices.enumerateDevices();
  const cameras = devices
    .filter((device) => device.kind === "videoinput")
    .map((device) => ({ id: device.deviceId, label: device.label }));
  await invoke("set_cameras", { cameras, selected: currentCameraId });
}

async function startCamera(deviceId: string | null) {
  stopCamera();
  try {
    stream = await openStream(deviceId);
    video.srcObject = stream;
    statusEl.hidden = true;
    currentCameraId = stream.getVideoTracks()[0]?.getSettings().deviceId ?? null;
    await syncCameras();
  } catch (error) {
    const name = errorName(error);
    if (name === "NotAllowedError") {
      showStatus("Akses kamera ditolak. Izinkan di System Preferences › Security & Privacy › Camera, lalu buka ulang app.");
    } else if (name === "NotFoundError") {
      showStatus("Kamera tidak ditemukan.");
    } else {
      showStatus(`Gagal membuka kamera: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
}

function queueCamera(action: () => Promise<void> | void) {
  cameraQueue = cameraQueue.then(action, action);
}

async function quit() {
  stopCamera();
  await appWindow.close();
}

frame.addEventListener("mousedown", (event) => {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement;
  if (target.closest("#close, #resize-handle")) return;
  appWindow.startDragging();
});

closeBtn.addEventListener("click", quit);

window.addEventListener("keydown", (event) => {
  if (event.metaKey && (event.key === "q" || event.key === "w")) {
    event.preventDefault();
    quit();
  }
});

resizeHandle.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  event.preventDefault();
  event.stopPropagation();
  resizeHandle.setPointerCapture(event.pointerId);

  const [rw, rh] = ratio;
  const aspect = rh / rw;
  const minWidth = MIN_SHORT_SIDE * Math.max(1, rw / rh);
  const startX = event.screenX;
  const startY = event.screenY;
  const startWidth = window.innerWidth;
  let pending: number | null = null;
  let nextWidth = startWidth;

  const onMove = (move: PointerEvent) => {
    const dx = move.screenX - startX;
    const dy = (move.screenY - startY) / aspect;
    nextWidth = Math.max(minWidth, Math.round(startWidth + Math.max(dx, dy)));
    if (pending !== null) return;
    pending = requestAnimationFrame(() => {
      pending = null;
      appWindow.setSize(new LogicalSize(nextWidth, Math.round(nextWidth * aspect)));
    });
  };

  const onUp = (up: PointerEvent) => {
    resizeHandle.releasePointerCapture(up.pointerId);
    resizeHandle.removeEventListener("pointermove", onMove);
    resizeHandle.removeEventListener("pointerup", onUp);
    resizeHandle.removeEventListener("pointercancel", onUp);
  };

  resizeHandle.addEventListener("pointermove", onMove);
  resizeHandle.addEventListener("pointerup", onUp);
  resizeHandle.addEventListener("pointercancel", onUp);
});

navigator.mediaDevices.addEventListener("devicechange", () => {
  queueCamera(() => {
    if (!stream) return;
    const ended = stream.getVideoTracks().every((track) => track.readyState === "ended");
    return ended ? startCamera(desiredCameraId) : syncCameras();
  });
});

async function init() {
  await listen<[number, number]>("ratio-changed", (event) => {
    ratio = event.payload;
  });
  await listen<string>("select-camera", (event) => {
    desiredCameraId = event.payload;
    queueCamera(() => startCamera(desiredCameraId));
  });
  await listen<boolean>("overlay-visibility", (event) => {
    queueCamera(() => (event.payload ? startCamera(desiredCameraId) : stopCamera()));
  });

  const settings = await invoke<Settings>("get_settings");
  ratio = settings.ratio;
  desiredCameraId = settings.cameraId;
  queueCamera(() => startCamera(desiredCameraId));
}

init();
