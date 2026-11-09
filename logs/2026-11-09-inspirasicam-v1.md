# 2026-11-09 — InspirasiCam v1 (initial build)

## Ringkasan
Membangun aplikasi kamera melayang dari nol (bukan fork circle-camera) dengan Tauri v2 + Vanilla TypeScript untuk macOS Monterey 12.7.6.

## Environment
- Install nvm + Node 22.23.4 LTS (Node 24 tidak mendukung macOS 12).

## Perubahan
- Scaffold `create-tauri-app` (vanilla-ts, npm); hapus demo greet, plugin opener, dan aset template.
- `src-tauri/tauri.conf.json`: window 270×480 (min 135×240), tanpa dekorasi, transparan, tanpa shadow, always-on-top, visible on all workspaces, skipTaskbar; `macOSPrivateApi`; CSP untuk media; bundle `app`+`dmg`, min macOS 12.0, ad-hoc signing, entitlements.
- `src-tauri/Cargo.toml`: feature `macos-private-api`; dependensi `objc2`, `objc2-foundation`, `objc2-app-kit` (macOS only); `tauri-plugin-window-state`.
- `src-tauri/src/lib.rs`: activation policy Accessory, restore window state + normalisasi rasio 9:16, konfigurasi NSWindow native (aspect ratio, collection behavior, window level).
- `src-tauri/Info.plist`: `NSCameraUsageDescription`, `LSUIElement`.
- `src-tauri/Entitlements.plist`: `com.apple.security.device.camera`.
- `src-tauri/capabilities/default.json`: start-dragging, close, set-size.
- `index.html`, `src/main.ts`, `src/styles.css`: video getUserMedia di frame rounded, drag, tombol × saat hover, Cmd+Q/W, pesan status error kamera, handle resize dengan rasio terkunci.
- Icon baru dari `app-icon.svg`.
- `.gitignore`: baris `logs` dihapus agar folder log ikut ter-track.
- README.md & AGENTS.md diperbarui.
- Bundle identifier diganti `com.inspirasicam.app` → `com.inspirasicam.desktop` (hindari bentrok dengan ekstensi `.app`).
- Repo GitHub publik `hams-rmdhn/inspirasicam` + release `v0.1.0` dengan DMG x64.

## Verifikasi
- `npx tsc --noEmit` OK, `cargo check` OK tanpa warning.
- Uji manual `tauri dev` oleh user: OK.

## Ditunda ke v2
Preset rasio (16:9, 9:16, 1:1, 4:3), switch kamera, menu bar icon, animasi hover lanjutan.
