# 2026-11-09 — InspirasiCam v0.2.0

## Ringkasan
Menambahkan icon menu bar sebagai pusat kontrol: preset rasio, pilih kamera, show/hide overlay, reset posisi & ukuran, versi, dan keluar. Pilihan rasio & kamera kini diingat. Ditambah gradien gelap saat hover.

## Perubahan
- `src-tauri/Cargo.toml`: feature tauri `tray-icon`, `image-png`; versi 0.2.0.
- `package.json`, `src-tauri/tauri.conf.json`: versi 0.2.0; `minWidth/minHeight` dihapus dari config (kini diatur dinamis per rasio di Rust).
- `src-tauri/src/settings.rs` (baru): enum `Ratio` (16:9, 9:16, 1:1, 4:3) + `Settings { ratio, cameraId }` disimpan sebagai JSON di app config dir.
- `src-tauri/src/lib.rs`:
  - `fit_window()` menggantikan `normalize_aspect_ratio()`: min size per rasio (sisi pendek ≥ 135), `setContentAspectRatio` native, ukuran dari luas area, opsional pertahankan titik tengah.
  - `apply_native_aspect()` dipisah dari `configure_overlay()`.
  - Menu bar (`build_tray`) dengan icon template monokrom `icons/tray.png`; handler `handle_menu` untuk rasio, kamera, toggle, reset, quit (simpan window state sebelum exit).
  - Commands `get_settings` dan `set_cameras` (rebuild submenu Kamera; label kosong → "Kamera N"; kamera fallback disimpan).
- `src/main.ts`: rasio dinamis untuk handle resize, `startCamera(deviceId)` dengan fallback ke kamera default, sinkronisasi daftar kamera (termasuk `devicechange`), listener `ratio-changed` / `select-camera` / `overlay-visibility`, antrean operasi kamera agar tidak balapan.
- `src/styles.css`: gradien gelap di atas saat hover; z-index tombol × dan handle resize.
- `tray-icon.svg` (baru) sebagai sumber icon menu bar.
- README.md & AGENTS.md diperbarui.

## Verifikasi
- `npx tsc --noEmit` OK, `cargo check` OK tanpa warning.
- Uji manual `tauri dev` oleh user: OK.
- Build release `.app` + `.dmg` dan rilis GitHub `v0.2.0`.
