# 2026-11-24 — v0.4.0: Mode Layar Penuh (Cmd+Shift+1)

## Ringkasan
Menambahkan mode fullscreen native macOS (window pindah ke Space sendiri), di-toggle dengan global shortcut `Cmd+Shift+1`, keluar via shortcut yang sama / Esc / item menu bar "Layar Penuh".

## Perubahan
- `src-tauri/Cargo.toml`: dep baru `tauri-plugin-global-shortcut = "2"` (2.4.1); versi 0.4.0.
- `src-tauri/src/lib.rs`:
  - `set_collection_behavior()` — refactor dari `configure_overlay`.
  - `toggle_fullscreen()` — masuk: `FullScreenPrimary` + clear aspect + mask fullscreen-ready; keluar: poll `is_fullscreen` + jeda 500ms, restore `configure_overlay` → `setStyleMask` asli (`SAVED_MASK`) → `fit_window`.
  - `exit_fullscreen` command (dipanggil dari Esc di webview).
  - Plugin global-shortcut + `register("CmdOrCtrl+Shift+1")` di setup.
  - Menu item `fullscreen` "Layar Penuh (⇧⌘1)" tanpa accelerator (menghindari double-fire dengan global hotkey).
  - Guard: `fit_window` di-skip saat ganti rasio dalam fullscreen; `reset` diabaikan saat fullscreen.
- `src/main.ts`: keydown Esc → `exit_fullscreen`; listener `fullscreen-changed` toggle class `.fullscreen` di `#frame`.
- `src/styles.css`: `#frame.fullscreen #resize-handle { display: none }`.
- `package.json`, `src-tauri/tauri.conf.json`: versi 0.4.0.
- `README.md`: fitur + baris menu baru.

## Gotcha yang ditemukan
- tao `set_fullscreen` memanggil `toggleFullScreen` tapi: (a) memaksa `setLevel(0)` → always-on-top hilang tanpa restore; (b) butuh mask `Titled|Resizable` — default-nya tao swap mask sementara → title bar "InspirasiCam" terlihat + duplikasi visual saat transisi. Solusi: pasang mask sendiri sebelumnya (titlebar invisible), tao skip swap-nya.
- `setStyleMask` saat `contentAspectRatio` masih aktif → crash `_NSWindowSetFrameIvar` (AppKit hitung ulang frame untuk titlebar dengan constraint konflik). Selalu clear aspect dulu.
- `windowDidExitFullScreen` fired saat window masih pindah Space → beri jeda sebelum restore mask/frame.

## Verifikasi
- `tsc --noEmit` bersih, `cargo check` bersih.
- Uji manual dev: Cmd+Shift+1 masuk/keluar fullscreen OK, tidak ada title bar, transisi bersih, always-on-top pulih setelah keluar.
