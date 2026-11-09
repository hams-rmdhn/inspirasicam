# InspirasiCam

Kamera melayang (floating) berbentuk kotak rounded 9:16 untuk macOS — selalu di atas, bisa di-drag dan di-resize dengan rasio terkunci. Cocok untuk presentasi dan screen recording.

## Fitur v1
- Video kamera default sistem, tanpa border, sudut rounded 12px
- Drag di mana saja untuk memindahkan
- Resize dari handle pojok kanan bawah (atau tepi window), rasio 9:16 terkunci
- Always on top, termasuk di atas app fullscreen dan di semua Spaces
- Tanpa icon Dock
- Posisi & ukuran diingat
- Hover → tombol × untuk keluar (juga Cmd+Q / Cmd+W)

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
