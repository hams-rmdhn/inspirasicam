# AGENTS.md — InspirasiCam

## Environment
- macOS 12.7.6 (Monterey), Intel x86_64.
- Node 22 LTS via nvm (`~/.nvm`). **Jangan pakai Node 24+** (butuh macOS ≥ 13.5). Jika `npm` tidak ditemukan di shell non-interaktif, tambahkan `~/.nvm/versions/node/v22.23.4/bin` ke PATH.
- Rust stable (cargo 1.93), Xcode CLT.

## Perintah
- Dev: `npm run tauri dev` (kompilasi Rust pertama ~12 menit; berikutnya cepat)
- Typecheck frontend: `npx tsc --noEmit`
- Cek Rust: `cd src-tauri && cargo check`
- Build release: `npm run tauri build` → `.app` + `.dmg` di `src-tauri/target/release/bundle/`
- Regenerate icon app: `npx tauri icon app-icon.svg`
- Regenerate icon menu bar: `npx tauri icon tray-icon.svg -o /tmp/trayicon --png 44 && cp /tmp/trayicon/44x44.png src-tauri/icons/tray.png`
- Release: `gh release create vX.Y.Z src-tauri/target/release/bundle/dmg/InspirasiCam_X.Y.Z_x64.dmg` (gh ada di `~/.local/bin/gh`)
- Versi harus di-bump di 3 tempat: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.

## Arsitektur
- Frontend vanilla TS: `index.html`, `src/main.ts`, `src/styles.css`.
- Rust: `src-tauri/src/lib.rs` + `src-tauri/src/settings.rs`
  - `ActivationPolicy::Accessory` → tanpa Dock icon (plus `LSUIElement` di `Info.plist`).
  - `configure_overlay()` (objc2-app-kit): collection behavior `CanJoinAllSpaces | FullScreenAuxiliary | Stationary`, level `NSStatusWindowLevel`.
  - `fit_window()` = satu-satunya tempat mengatur ukuran sesuai rasio: min size, `setContentAspectRatio`, ukuran dari luas area, opsional pertahankan titik tengah.
  - `tauri-plugin-window-state` (SIZE | POSITION) di-restore manual di `setup`, lalu `fit_window`.
  - Menu bar (`build_tray`, `handle_menu`): ID item `ratio:<id>`, `camera:<deviceId>`, `toggle`, `reset`, `quit`.
  - Settings (`ratio`, `cameraId`) disimpan di `app_config_dir()/settings.json`.
- Alur data: Rust = sumber kebenaran rasio/menu/settings. JS memanggil `get_settings` saat start dan `set_cameras` (daftar dari `enumerateDevices`) untuk mengisi submenu Kamera. Rust mengirim event `ratio-changed`, `select-camera`, `overlay-visibility` ke webview.
- Definisi rasio hanya di `settings.rs` (`Ratio`); JS menerima `[w, h]` lewat `get_settings`/`ratio-changed`.
- Tauri tidak punya API lock aspect ratio (tauri#5332) → native di Rust + handle resize JS (`#resize-handle`) yang menjaga rasio via `setSize`.
- `startResizeDragging` tidak didukung di macOS, jangan dipakai.
- Versi `objc2` (0.6) / `objc2-app-kit` / `objc2-foundation` (0.3) harus sama dengan yang dipakai tauri (cek `cargo tree -i objc2`).

## Konvensi
- Ringkasan perubahan ditulis di `logs/`.
