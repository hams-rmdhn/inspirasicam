# 2026-11-10 — InspirasiCam v0.3.0

## Ringkasan
Menambahkan perekaman video ke mp4 di folder Movies: indikator REC dengan timer, pilihan mikrofon, notifikasi sistem, dan crop otomatis ke rasio aktif via AVFoundation (`AVAssetExportSession`). Jalur `canvas.captureStream` sempat dicoba dan diabandon karena menghasilkan frame hitam di WebKit Monterey — solusinya merekam track kamera mentah lalu crop file hasilnya di native.

## Perubahan
- `src-tauri/src/recording.rs` (baru): commands `recording_start`/`recording_append`/`recording_stop` (tulis chunk MediaRecorder ke `~/Movies/Rekaman-<timestamp>.mp4`), `recording_error`, dan `crop_recording` — crop center ke rasio aktif via `AVMutableVideoComposition` + `AVAssetExportSession`, rename atomik timpa file asli; skip export bila rasio sudah cocok.
- `src-tauri/src/lib.rs`: `AppState.recording`, submenu Mikrofon, item menu "Mulai Rekam"/"Berhenti Rekam", kunci rasio & kamera selama rekam, `set_devices` generik (kamera/mic), quit menunggu rekaman berhenti (fallback force-quit 5 detik), event `recording-toggle`/`select-mic`/`request-quit`.
- `src-tauri/src/settings.rs`: `mic_id` di Settings.
- `src/main.ts`: `MediaRecorder` merekam video track mentah + audio mic via `AudioContext`→`MediaStreamDestination`; chunk → `recording_append`; setelah stop → `crop_recording`; ganti mic saat rekam; toast status; indikator REC.
- `index.html`/`src/styles.css`: elemen `#rec` (dot + timer), `#toast`.
- `src-tauri/Entitlements.plist` + `Info.plist`: entitlement + usage description mikrofon.
- `Cargo.toml`: `objc2-av-foundation`, `objc2-core-media`, `objc2-core-foundation`, `block2`, `tauri-plugin-notification`; versi 0.3.0.

## Verifikasi
- `npx tsc --noEmit` OK, `cargo check` OK.
- Rekaman 9:16 menghasilkan 270×480 dengan frame nyata (bukan hitam) dan audio AAC utuh — diverifikasi via `mdls` + thumbnail Quick Look.
- Rekaman 4:3 (rasio sudah cocok dengan kamera 640×480) melewati jalur early-return tanpa menghapus file — bug kehilangan file di jalur ini sudah diperbaiki.
- Uji manual `tauri dev` oleh user: OK.
