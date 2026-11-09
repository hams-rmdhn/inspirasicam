import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

const ASPECT = 16 / 9;
const MIN_WIDTH = 135;

const appWindow = getCurrentWindow();
const frame = document.querySelector<HTMLDivElement>("#frame")!;
const video = document.querySelector<HTMLVideoElement>("#cam")!;
const statusEl = document.querySelector<HTMLDivElement>("#status")!;
const closeBtn = document.querySelector<HTMLButtonElement>("#close")!;
const resizeHandle = document.querySelector<HTMLDivElement>("#resize-handle")!;

let stream: MediaStream | null = null;

function showStatus(message: string) {
  statusEl.textContent = message;
  statusEl.hidden = false;
}

async function startCamera() {
  try {
    stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
    video.srcObject = stream;
    statusEl.hidden = true;
  } catch (error) {
    const name = error instanceof DOMException ? error.name : "";
    if (name === "NotAllowedError") {
      showStatus("Akses kamera ditolak. Izinkan di System Preferences › Security & Privacy › Camera, lalu buka ulang app.");
    } else if (name === "NotFoundError") {
      showStatus("Kamera tidak ditemukan.");
    } else {
      showStatus(`Gagal membuka kamera: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
}

async function quit() {
  stream?.getTracks().forEach((track) => track.stop());
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

  const startX = event.screenX;
  const startY = event.screenY;
  const startWidth = window.innerWidth;
  let pending: number | null = null;
  let nextWidth = startWidth;

  const onMove = (move: PointerEvent) => {
    const dx = move.screenX - startX;
    const dy = (move.screenY - startY) / ASPECT;
    nextWidth = Math.max(MIN_WIDTH, Math.round(startWidth + Math.max(dx, dy)));
    if (pending !== null) return;
    pending = requestAnimationFrame(() => {
      pending = null;
      appWindow.setSize(new LogicalSize(nextWidth, Math.round(nextWidth * ASPECT)));
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

startCamera();
