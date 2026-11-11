# 2026-11-11 — Ganti logo aplikasi

## Ringkasan
Mengganti logo aplikasi dari icon gradien lama menjadi icon kamera video merah (`inspirasi-cam-logo.png` dari Downloads). Icon menu bar juga diperbarui dari sumber yang sama (dipakai sebagai template image → siluet monokrom di menu bar).

## Perubahan
- `app-icon.svg` dihapus, diganti `app-icon.png` (512×512, alpha) sebagai sumber icon.
- `tray-icon.svg` dihapus — icon tray kini digenerate dari `app-icon.png`.
- Semua file di `src-tauri/icons/` di-regenerate via `npx tauri icon` (icns, ico, semua ukuran png, android/ios), termasuk `tray.png` (44×44).
- `AGENTS.md`: perintah regenerate icon diperbarui.

## Verifikasi
- Hasil 128×128 dan tray 44×44 dicek visual: bentuk kamera video merah dengan background transparan.
