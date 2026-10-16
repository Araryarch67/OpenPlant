# OpenPlant — agent rules

Native Rust desktop app (alternatif StarUML / draw.io) di atas GPUI via
`gpui-kit` 0.7. Satu crate di repo root, binary `openplant`.
Full Rust: tanpa web frontend, Tauri, atau Node/Bun.
Lisensi AGPL-3.0-or-later (`LICENSE`, field `license` di `Cargo.toml`).
Komunikasi dengan user dalam Bahasa Indonesia.

## Perintah (Cargo only)

- `cargo build`, `cargo test`, `cargo run -- [diagram.puml | project.json]`
- Rilis: `cargo build --release`
- Wajib sebelum kerja dianggap selesai: build bebas warning + `cargo test` hijau.
- Build pertama lambat (~5 mnt: GPUI + wgpu); incremental hitungan detik.
- `target/` besar (~9 GB) dan di-gitignore — jangan pernah commit.

## Batasan

- Prioritas #1: cepat + ringan. Dependensi tetap: gpui-kit, serde,
  serde_json, regex, resvg, ureq. Tanpa UI framework, canvas lib, icon pack.
- Ikon = path ops di `src/ui/icons.rs`; kanvas digambar manual di `src/canvas/`.
- Keymap gaya draw.io di `src/commands/keymap.rs`: editing keys konteks
  `Canvas` (agar mengetik di field tak mengedit diagram), global `OpenPlant`.
- PlantUML render di kanvas sendiri (`src/plantuml/`). Jangan tambah
  server/jar/preview split.
- Clean-room: StarUML/draw.io hanya referensi perilaku — jangan salin
  kode, aset, ikon, atau font.

## Arsitektur (`src/`)

- `main.rs` — entry, window 1600x900
- `theme.rs` — palet Everblush
- `model/` — project/diagram/shape/relation (JSON camelCase), `toolbox.rs`
- `plantuml/` — parse, layout, generate; round-trip test di `sample-puml/`
- `workbench/` — state tunggal: selection, view, panels, clipboard,
  undo/redo (`checkpoint()` sekali per aksi pengguna)
- `commands/` — actions, keymap, menus, palette, shortcut sheet
- `canvas/` — routing, hit test, painting shape/edge, minimap
- `app/` — root view: state machine drag, dispatch `run(id)`, dialogs,
  sync PlantUML, inline editing
- `ui/` — render per region + drawer + overlays
- `services/` — `export.rs` (SVG + PNG via resvg), `settings.rs`
  (`~/.config/openplant/settings.json`), `ai.rs` (chat API
  OpenAI-compatible; Ollama atau cloud)
- `ui_tests/` — headless UI tests (`cargo test`)

## Keputusan yang sudah diambil

- Konflik shortcut StarUML vs draw.io (mis. `Ctrl+Shift+F/H/K`): draw.io menang.
- Fitur user-facing + roadmap tinggal di `README.md`; jangan duplikasi di sini.
