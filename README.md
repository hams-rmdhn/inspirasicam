# InspirasiCam

Kamera melayang (floating) berbentuk kotak rounded untuk macOS — selalu di atas, bisa di-drag dan di-resize dengan rasio terkunci. Cocok untuk presentasi dan screen recording.

## Fitur
- Video kamera tanpa border, sudut rounded 12px
- Preset rasio: 16:9 Landscape, 9:16 Portrait, 1:1 Square, 4:3 Classic (luas area & titik tengah dipertahankan saat ganti)
- Pilih kamera (built-in, webcam USB, iPhone Continuity Camera); daftar ter-update otomatis saat kamera dicolok/dicabut
- Drag di mana saja untuk memindahkan
- Resize dari handle pojok kanan bawah (atau tepi window), rasio terkunci
- Always on top, termasuk di atas app fullscreen dan di semua Spaces
- Tanpa icon Dock — dikontrol dari icon menu bar
- Rasio, kamera, posisi & ukuran diingat
- Rekam ke `~/Movies/Rekaman-<timestamp>.mp4` — otomatis di-crop ke rasio aktif, audio dari mikrofon pilihan
- Hover → gradien halus + tombol × untuk keluar (juga Cmd+Q / Cmd+W)

## Menu Bar
| Item | Fungsi |
|---|---|
| InspirasiCam vX.Y.Z | Versi app |
| Sembunyikan / Tampilkan Overlay | Sembunyikan kamera tanpa menutup app (kamera dimatikan) |
| Mulai Rekam / Berhenti Rekam | Rekam video+audio ke folder Movies |
| Rasio ▸ | Ganti preset rasio |
| Kamera ▸ | Pilih kamera |
| Mikrofon ▸ | Pilih mikrofon untuk rekaman |
| Reset Posisi & Ukuran | Kembali ke tengah layar dengan ukuran default |
| Keluar | Tutup app |

## Kebutuhan
- macOS 12 (Monterey) atau lebih baru
- Node.js 22 LTS (Node 24 tidak mendukung Monterey), Rust, Xcode Command Line Tools

## Development
```bash
npm install
npm run tauri dev
```

## Build
```bash
npm run tauri build
```
Hasil: `src-tauri/target/release/bundle/macos/InspirasiCam.app` dan `src-tauri/target/release/bundle/dmg/`.
