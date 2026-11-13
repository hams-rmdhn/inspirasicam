# 2026-11-13 — v0.3.2: Icon aplikasi & menu bar baru

## Ringkasan
Mengganti icon aplikasi dari kamera merah menjadi rounded-square teal dengan outline kamera putih, dan icon menu bar menjadi siluet outline kamera yang lebih bersih.

## Perubahan
- `app-icon.png`: sumber icon baru (`inspirasi-cam-icon.png` dari Downloads, 1024×1024, alpha).
- `src-tauri/icons/tray.png`: siluet outline kamera 44×33 (`inspirasi-cam-bar-icon.png`). Non-square aman karena tray-icon crate set tinggi 18pt dengan lebar proporsional; `icon_as_template(true)` memakai alpha saja.
- Semua file di `src-tauri/icons/` di-regenerate via `npx tauri icon app-icon.png`.
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`: versi 0.3.2.

## Verifikasi
- `icon.icns` di bundle hasil build adalah icon baru (127.660 bytes).
- `tray.png` md5 cocok dengan sumber.
- Build release: `InspirasiCam_0.3.1_x64.dmg` → `InspirasiCam_0.3.2_x64.dmg`.
