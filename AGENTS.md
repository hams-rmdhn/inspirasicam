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
- Regenerate icon: `npx tauri icon app-icon.svg`

## Arsitektur
- Frontend vanilla TS: `index.html`, `src/main.ts`, `src/styles.css`.
- Rust: `src-tauri/src/lib.rs`
  - `ActivationPolicy::Accessory` → tanpa Dock icon (plus `LSUIElement` di `Info.plist`).
  - `configure_overlay()` memakai `objc2-app-kit`: `setContentAspectRatio(9:16)`, collection behavior `CanJoinAllSpaces | FullScreenAuxiliary | Stationary`, level `NSStatusWindowLevel`.
  - `tauri-plugin-window-state` (SIZE | POSITION) di-restore manual di `setup`, lalu `normalize_aspect_ratio()`.
- Tauri tidak punya API lock aspect ratio (tauri#5332) → native di Rust + handle resize JS (`#resize-handle`) yang menjaga rasio via `setSize`.
- `startResizeDragging` tidak didukung di macOS, jangan dipakai.
- Versi `objc2` (0.6) / `objc2-app-kit` / `objc2-foundation` (0.3) harus sama dengan yang dipakai tauri (cek `cargo tree -i objc2`).
- Konstanta rasio ada di dua tempat: `ASPECT_*` di `lib.rs` dan `ASPECT` di `main.ts`.

## Konvensi
- Ringkasan perubahan ditulis di `logs/`.
