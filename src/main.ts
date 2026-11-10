import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

type Settings = { ratio: [number, number]; cameraId: string | null; micId: string | null };

type RecordingSession = {
  recorder: MediaRecorder;
  audioCtx: AudioContext;
  dest: MediaStreamAudioDestinationNode;
  source: MediaStreamAudioSourceNode;
  micStream: MediaStream;
  writes: Promise<void>;
  stopped: Promise<void>;
  startedAt: number;
  timer: number;
};

const MIN_SHORT_SIDE = 135;

const appWindow = getCurrentWindow();
const frame = document.querySelector<HTMLDivElement>("#frame")!;
const video = document.querySelector<HTMLVideoElement>("#cam")!;
const statusEl = document.querySelector<HTMLDivElement>("#status")!;
const closeBtn = document.querySelector<HTMLButtonElement>("#close")!;
const resizeHandle = document.querySelector<HTMLDivElement>("#resize-handle")!;
const recEl = document.querySelector<HTMLDivElement>("#rec")!;
const recTimeEl = document.querySelector<HTMLSpanElement>("#rec-time")!;
const toastEl = document.querySelector<HTMLDivElement>("#toast")!;

let stream: MediaStream | null = null;
let ratio: [number, number] = [9, 16];
let desiredCameraId: string | null = null;
let currentCameraId: string | null = null;
let micId: string | null = null;
let session: RecordingSession | null = null;
let cameraQueue: Promise<void> = Promise.resolve();
let toastTimer: number | undefined;

function showStatus(message: string) {
  statusEl.textContent = message;
  statusEl.hidden = false;
}

function showToast(message: string) {
  toastEl.textContent = message;
  toastEl.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (toastEl.hidden = true), 4000);
}

function errorName(error: unknown) {
  return (error as { name?: string } | null)?.name ?? "";
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
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

async function syncDevices() {
  const devices = await navigator.mediaDevices.enumerateDevices();
  const pick = (kind: MediaDeviceKind) =>
    devices
      .filter((device) => device.kind === kind && device.deviceId)
      .map((device) => ({ id: device.deviceId, label: device.label }));
  await invoke("set_devices", { kind: "camera", devices: pick("videoinput"), selected: currentCameraId });
  await invoke("set_devices", { kind: "mic", devices: pick("audioinput"), selected: micId });
}

async function startCamera(deviceId: string | null) {
  stopCamera();
  try {
    stream = await openStream(deviceId);
    video.srcObject = stream;
    statusEl.hidden = true;
    currentCameraId = stream.getVideoTracks()[0]?.getSettings().deviceId ?? null;
    await syncDevices();
  } catch (error) {
    const name = errorName(error);
    if (name === "NotAllowedError") {
      showStatus("Akses kamera ditolak. Izinkan di System Preferences › Security & Privacy › Camera, lalu buka ulang app.");
    } else if (name === "NotFoundError") {
      showStatus("Kamera tidak ditemukan.");
    } else {
      showStatus(`Gagal membuka kamera: ${errorMessage(error)}`);
    }
  }
}

function queueCamera(action: () => Promise<void> | void) {
  cameraQueue = cameraQueue.then(action, action);
}

async function openMic(deviceId: string | null) {
  try {
    return await navigator.mediaDevices.getUserMedia({
      audio: deviceId ? { deviceId: { exact: deviceId } } : true,
      video: false,
    });
  } catch (error) {
    const name = errorName(error);
    if (name === "NotAllowedError") {
      throw new Error("Akses mikrofon ditolak. Izinkan di System Preferences › Security & Privacy › Microphone.");
    }
    if (name === "NotFoundError" || name === "OverconstrainedError") {
      throw new Error("Mikrofon tidak ditemukan. Pilih mikrofon lain di menu Mikrofon.");
    }
    throw new Error(`Gagal membuka mikrofon: ${errorMessage(error)}`);
  }
}

function timestamp() {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  return [
    now.getFullYear(),
    pad(now.getMonth() + 1),
    pad(now.getDate()),
    pad(now.getHours()),
    pad(now.getMinutes()),
    pad(now.getSeconds()),
  ].join("-");
}

function formatDuration(ms: number) {
  const total = Math.floor(ms / 1000);
  const pad = (value: number) => String(value).padStart(2, "0");
  const hours = Math.floor(total / 3600);
  const rest = `${pad(Math.floor((total % 3600) / 60))}:${pad(total % 60)}`;
  return hours ? `${hours}:${rest}` : rest;
}


async function failRecording(message: string) {
  showToast(message);
  await invoke("recording_error", { message }).catch(console.error);
}

async function startRecording() {
  const videoTrack = stream?.getVideoTracks()[0];
  if (!videoTrack || videoTrack.readyState === "ended") {
    await failRecording("Kamera tidak aktif.");
    return;
  }

  let micStream: MediaStream;
  try {
    micStream = await openMic(micId);
  } catch (error) {
    await failRecording(errorMessage(error));
    return;
  }

  let audioCtx: AudioContext | null = null;
  try {
    audioCtx = new AudioContext();
    await audioCtx.resume();
    const dest = audioCtx.createMediaStreamDestination();
    const source = audioCtx.createMediaStreamSource(micStream);
    source.connect(dest);

    const preferred = "video/mp4;codecs=avc1,mp4a";
    const mimeType = MediaRecorder.isTypeSupported(preferred) ? preferred : "video/mp4";
    const recorder = new MediaRecorder(
      new MediaStream([videoTrack, ...dest.stream.getAudioTracks()]),
      { mimeType },
    );

    await invoke("recording_start", { timestamp: timestamp() });

    const current: RecordingSession = {
      recorder,
      audioCtx,
      dest,
      source,
      micStream,
      writes: Promise.resolve(),
      stopped: new Promise((resolve) => (recorder.onstop = () => resolve())),
      startedAt: Date.now(),
      timer: 0,
    };
    recorder.ondataavailable = (event) => {
      if (!event.data.size) return;
      current.writes = current.writes
        .then(async () => invoke("recording_append", new Uint8Array(await event.data.arrayBuffer())))
        .then(() => undefined)
        .catch(console.error);
    };
    recorder.onerror = () => {
      showToast("Perekam mengalami error, rekaman disimpan.");
      queueCamera(stopRecording);
    };
    recorder.start(1000);

    session = current;
    recTimeEl.textContent = "00:00";
    recEl.hidden = false;
    current.timer = window.setInterval(() => {
      recTimeEl.textContent = formatDuration(Date.now() - current.startedAt);
    }, 1000);
    await syncDevices();
  } catch (error) {
    micStream.getTracks().forEach((track) => track.stop());
    audioCtx?.close();
    await failRecording(`Gagal memulai rekaman: ${errorMessage(error)}`);
  }
}

async function stopRecording() {
  const current = session;
  if (!current) return;
  session = null;

  if (current.recorder.state !== "inactive") current.recorder.stop();
  await Promise.race([current.stopped, new Promise((resolve) => setTimeout(resolve, 3000))]);
  await current.writes;

  clearInterval(current.timer);
  recEl.hidden = true;
  current.micStream.getTracks().forEach((track) => track.stop());
  current.audioCtx.close();

  try {
    const name = await invoke<string>("recording_stop");
    showToast("Memproses video…");
    try {
      await invoke<string>("crop_recording", { name, rw: ratio[0], rh: ratio[1] });
      showToast(`Tersimpan: ${name}`);
    } catch (cropError) {
      showToast(`Tersimpan tanpa crop: ${name} (${errorMessage(cropError)})`);
    }
  } catch (error) {
    showToast(`Gagal menyimpan rekaman: ${errorMessage(error)}`);
  }
}

async function switchMic(deviceId: string | null) {
  micId = deviceId;
  if (!session) return;
  try {
    const micStream = await openMic(deviceId);
    const current = session;
    if (!current) {
      micStream.getTracks().forEach((track) => track.stop());
      return;
    }
    const source = current.audioCtx.createMediaStreamSource(micStream);
    source.connect(current.dest);
    current.source.disconnect();
    current.micStream.getTracks().forEach((track) => track.stop());
    current.source = source;
    current.micStream = micStream;
  } catch (error) {
    showToast(`${errorMessage(error)} Tetap memakai mikrofon sebelumnya.`);
  }
}

function quit() {
  queueCamera(async () => {
    await stopRecording();
    stopCamera();
    await appWindow.close();
  });
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
  queueCamera(async () => {
    if (!stream) return;
    const ended = stream.getVideoTracks().every((track) => track.readyState === "ended");
    if (!ended) return syncDevices();
    if (session) showToast("Kamera terputus, rekaman disimpan.");
    await stopRecording();
    await startCamera(desiredCameraId);
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
  await listen<string | null>("select-mic", (event) => {
    queueCamera(() => switchMic(event.payload));
  });
  await listen<boolean>("overlay-visibility", (event) => {
    queueCamera(() => {
      if (!event.payload) return stopCamera();
      if (!stream) return startCamera(desiredCameraId);
    });
  });
  await listen("recording-toggle", () => {
    queueCamera(() => (session ? stopRecording() : startRecording()));
  });
  await listen("request-quit", quit);

  const settings = await invoke<Settings>("get_settings");
  ratio = settings.ratio;
  desiredCameraId = settings.cameraId;
  micId = settings.micId;
  queueCamera(() => startCamera(desiredCameraId));
}

init();
