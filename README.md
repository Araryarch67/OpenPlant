# OpenPlant

Aplikasi desktop UML & diagram native (alternatif StarUML / draw.io) yang ditulis penuh di Rust dengan GPUI.

Fokus: cepat, ringan, offline-first, dengan sinkronisasi dua arah PlantUML di kanvas sendiri — tanpa server, JVM, Electron, atau web view.

![license](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)
![rust](https://img.shields.io/badge/built_with-Rust-orange)
![platform](https://img.shields.io/badge/platform-Linux-lightgrey)

## Daftar Isi

- [Fitur](#fitur)
- [Persyaratan](#persyaratan)
- [Instalasi & Menjalankan](#instalasi--menjalankan)
- [Penggunaan](#penggunaan)
- [Format File](#format-file)
- [PlantUML Two-Way Sync](#plantuml-two-way-sync)
- [Generator AI](#generator-ai)
- [Shortcut](#shortcut)
- [Struktur Proyek](#struktur-proyek)
- [Roadmap](#roadmap)
- [Berkontribusi](#berkontribusi)
- [Lisensi](#lisensi)

## Fitur

- **Kanvas vektor native** — shape dan edge digambar manual (bukan web view), dengan routing ortogonal, minimap interaktif (klik/drag), dan thumbnail diagram.
- **Multi-diagram** — tambah, ganti, dan hapus diagram dalam satu project.
- **Toolbox per kategori** — Class, Use Case, Container (Package/Folder/Frame/Rectangle/Queue/Cloud), Deployment (Node/Artifact/Component/Database/Storage), dan Note.
- **Relasi UML lengkap** — Association, Directed Association, Aggregation, Composition, Dependency, Generalization, Realization, Note Link.
- **PlantUML live sync dua arah** — edit di kanvas memperbarui teks `.puml`, dan sebaliknya. Parser + layout sendiri, round-trip tested di `sample-puml/`.
- **Undo/redo, cut/copy/paste/duplicate** — satu checkpoint per aksi pengguna.
- **Element picker, styles editor, validasi dasar, panel properti.**
- **Export SVG dan PNG** — SVG standalone sesuai tampilan kanvas, PNG diraster via `resvg` (skala 2x).
- **Generator diagram AI** — API chat kompatibel OpenAI; Ollama lokal atau cloud.
- **Shortcut gaya draw.io** — full keymap, aman saat mengetik (konteks `Canvas` vs global `OpenPlant`).
- **Tema Everblush** — layout ala StarUML: menu bar, sidebar, canvas, tool strip, navigator, drawer, status bar.

## Persyaratan

- Rust stable terbaru (`rustup update`)
- Linux dengan GPU/wgpu yang berfungsi (pengembangan utama di Hyprland/Arch)
- Build pertama lambat (~5 menit: kompilasi GPUI + wgpu); build incremental hanya hitungan detik
- Direktori `target/` debug bisa ~9 GB — `cargo clean` untuk merebut kembali ruang disk

## Instalasi & Menjalankan

```bash
cargo build
cargo test
cargo run
```

Buka file langsung:

```bash
cargo run -- diagram.puml
cargo run -- project.json
```

Build rilis:

```bash
cargo build --release
./target/release/openplant [diagram.puml | project.json]
```

Preferensi disimpan di:

```
$XDG_CONFIG_HOME/openplant/settings.json
# default: ~/.config/openplant/settings.json
```

## Penggunaan

1. Pilih tool dari tool strip (Node atau Edge).
2. Klik-drag di kanvas untuk menaruh shape; untuk edge, drag dari shape sumber ke shape target.
3. Edit label dengan `F2` / `Enter`, atau lewat panel Properties.
4. Atur gaya (fill, stroke, font, dash) lewat Styles editor.
5. Simpan sebagai `project.json` (`Ctrl+S`) atau sebagai `.puml` (`Save As`), export SVG/PNG dari menu/diagram aktif.
6. Gunakan drawer PlantUML untuk melihat/mengedit teks dan sinkronisasi langsung ke kanvas.

## Format File

Dua format yang didukung:

| Format | Ekstensi | Isi |
|---|---|---|
| Project | `.json` | Project, model tree, dan semua diagram (`camelCase` JSON) |
| PlantUML | `.puml` | Teks PlantUML subset (parse + generate sendiri) |

Contoh project default berisi diagram `Main` dengan `Customer —places→ Order`. Contoh PlantUML kompleks ada di `sample-puml/enterprise-library-management.puml`.

## PlantUML Two-Way Sync

- Parser subset sendiri di `src/plantuml/` — grouped layout, notes, komentar `' pos`, `title`, `legend`, `preamble`.
- Tidak memakai server/jar PlantUML atau split preview; semua dirender di kanvas OpenPlant.
- Round-trip (`parse → layout → generate → parse`) diuji terhadap file di `sample-puml/`.

## Generator AI

Drawer AI mendukung dua provider (OpenAI-compatible chat API):

- **Lokal (default):** Ollama di `http://localhost:11434/v1`
  ```bash
  ollama serve
  ollama pull llama3.1   # atau model lain
  ```
  Daftar model diambil otomatis dari `GET /api/tags`. Pilih model, tulis prompt, Generate.
- **Cloud:** isi endpoint (default `https://api.openai.com/v1`), API key (hanya tersimpan lokal di `settings.json`), dan nama model.

Prompt dikirim dengan system prompt generator PlantUML class diagram; hasil yang diawali `@startuml` / diakhiri `@enduml` langsung diparse menjadi shape + relasi di kanvas.

## Shortcut

Keymap gaya draw.io (`src/commands/keymap.rs`). Global memakai konteks `OpenPlant`, editing memakai konteks `Canvas` agar mengetik di field tidak mengedit diagram.

| Aksi | Shortcut |
|---|---|
| New / Open / Save / Save As | `Ctrl+N` `Ctrl+O` `Ctrl+S` `Ctrl+Shift+S` |
| Undo / Redo | `Ctrl+Z` / `Ctrl+Shift+Z`, `Ctrl+Y` |
| Cut / Copy / Paste / Duplicate | `Ctrl+X` `Ctrl+C` `Ctrl+V` `Ctrl+D` |
| Delete selection / Delete all | `Del`, `Backspace` / `Ctrl+Del` |
| Select all / none / vertices / edges | `Ctrl+A` / `Ctrl+Shift+A` / `Ctrl+Shift+I` / `Ctrl+Shift+E` |
| Edit label / Cancel | `F2`, `Enter` / `Esc` |
| Command palette / Shortcut sheet | `F1` / `Ctrl+/` |
| Zoom in / out / reset / fit | `Ctrl+=` `Ctrl+-` `Ctrl+0` / `Ctrl+Shift+H`, `Ctrl+J` |
| Toggle grid / outline / format / shapes | `Ctrl+Shift+G` `Ctrl+Shift+O` `Ctrl+Shift+P` `Ctrl+Shift+K` |
| To front / to back | `Ctrl+Shift+F` / `Ctrl+Shift+B` |
| Nudge / grid-nudge | `Arrows` / `Shift+Arrows` |
| Resize / grid-resize | `Ctrl+Arrows` / `Ctrl+Shift+Arrows` |

## Struktur Proyek

Satu crate di repo root, binary `openplant`:

```
src/
  main.rs            entry point, window 1600x900
  theme.rs           palet Everblush
  model/             project/diagram/shape/relation (JSON camelCase), toolbox
  plantuml/          parse, layout, generate PlantUML subset
  workbench.rs → workbench/  state tunggal: project, selection, view,
                     panels, clipboard, undo/redo
  commands/          actions, keymap, menus, palette, shortcut sheet
  canvas/            routing, hit testing, painting shape/edge, minimap
  app/               root view: state machine Drag, dispatch run(id),
                     dialogs, sync PlantUML, inline editing
  ui/                render per region + drawer + overlays
  services/          export (SVG/PNG), settings, AI generator
  icons.rs → ui/icons.rs  ikon sebagai path ops (tanpa icon pack)
```

Dependensi: `gpui-kit`, `serde`, `serde_json`, `regex`, `resvg`, `ureq`. Tanpa framework UI tambahan, canvas lib, atau icon pack.

## Roadmap

- Render PlantUML lebih faithful: `skinparam` warna, legend, layout berperingkat, member class, stereotype, multiplicity
- Diagram selain class: sequence, usecase, activity, component, deployment, statechart
- Rename diagram langsung dari daftar working diagrams
- Context menu lanjutan (submenu Format, Add cloud models, Tag Editor)
- Extension system (registry + API menu/toolbox)
- Validation rules lanjutan, print & HTML export, filter explorer by kind, i18n, MCP server

Detail ada di `AGENTS.md`.

## Berkontribusi

1. Fork & buat branch dari `main`.
2. Pastikan `cargo build` bebas warning dan `cargo test` hijau sebelum PR.
3. Ikuti aturan di `AGENTS.md`: Cargo only, kanvas tetap ringan, tanpa dependensi UI baru, clean-room (StarUML/draw.io hanya referensi perilaku — jangan salin kode/aset/font mereka).

## Lisensi

Copyright (C) 2026 Araryarch67 dan kontributor OpenPlant.

Program ini free software: boleh didistribusikan dan/atau dimodifikasi di bawah ketentuan **GNU Affero General Public License v3.0 atau versi setelahnya** seperti diterbitkan Free Software Foundation. Lihat file [`LICENSE`](LICENSE) untuk teks lengkap.

Konsekuensi praktis AGPL-v3: jika menjalankan versi modifikasi untuk pengguna lewat jaringan, wajib menyediakan Corresponding Source ke pengguna tersebut (pasal 13).
