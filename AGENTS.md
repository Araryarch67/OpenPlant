# OpenPlant — agent rules

Native Rust desktop app (StarUML / draw.io alternative) built on GPUI via
`gpui-kit` 0.7 (gpui-pre + gpui-component). One crate at the repo root,
binary `openplant`. Full Rust: no web frontend, no Tauri, no Node/Bun.

## Non-negotiables

- **Cargo only.** `cargo build`, `cargo test`,
  `cargo run -- [diagram.puml | project.json]`. Release: `cargo build --release`.
- **Warning-free build and green `cargo test`** before calling work done.
- **Speed + lightweight is priority #1.** Dependencies are gpui-kit, serde,
  serde_json, regex, resvg (PNG export, already pulled in by GPUI) and ureq
  (AI generator). No extra UI frameworks, canvas libs or icon packs: icons are
  path ops in `src/icons.rs`, the canvas is painted by hand in `src/diagram.rs`.
- **UI stays matching `shots/`.** Those screenshots (1600x900) are the
  reference look: Everblush palette (`src/theme.rs`), StarUML layout
  (menu bar, sidebar, canvas, tool strip, navigator, drawer, status bar).
- **draw.io keyboard shortcuts.** Keymap lives in `src/commands.rs`. Editing
  keys are bound in the `Canvas` key context so typing in a field never edits
  the diagram; global keys use the `OpenPlant` context.
- **PlantUML renders on our own canvas.** Parser + layout in
  `src/plantuml.rs`. Never add a PlantUML server/jar preview or split view.
- **Clean-room.** StarUML and draw.io are behaviour/layout references only.
  Never copy their code, assets, icons or fonts.

## Architecture (`src/`)

- `model.rs` — project/diagram/shape/relation types (camelCase JSON on disk).
- `toolbox.rs` — tool groups, diagram kinds, default shape sizes.
- `plantuml.rs` — PlantUML subset parse (grouped layout, notes, `' pos`
  comments) and generate; round-trip tested on `sample-puml/`.
- `workbench.rs` — the single state struct: project, selection, view,
  panels, clipboard, undo/redo (`checkpoint()` once per user gesture).
- `commands.rs` — actions, keymap, menus, palette list, shortcut sheet.
- `diagram.rs` — routing, hit testing, shape/edge painting, minimap, thumbs.
- `app.rs` — root view: canvas interaction state machine (`Drag`), command
  dispatch (`run(id)`), file dialogs, PlantUML two-way sync, inline editing.
- `ui.rs` — region rendering, one method per workbench region.
- `export.rs` — standalone SVG and PNG (resvg) of the active diagram.
- `settings.rs` — prefs at `$XDG_CONFIG_HOME/openplant/settings.json`.
- `ai.rs` — AI generator (OpenAI-compatible chat API; Ollama or cloud).

## TODO (belum diimplementasi — jangan lupa)

- **Render PlantUML lebih faithful di canvas sendiri**: warna dari
  `skinparam`, legend, layout berperingkat (ikut arah relasi dan
  `left to right direction`), member class (atribut/operasi),
  stereotype, multiplicity.
- **Diagram selain class**: sequence, usecase, activity, component,
  deployment, statechart (canvas + toolbox per-diagram).
- **Rename diagram langsung** dari daftar working diagrams (sekarang lewat
  properti `name`).
- **Context menu lanjutan**: submenu Format, Open Sub-Diagram,
  Tag Editor.
  - Submenu Add: AWS Model, Azure Model, GCP Model (cloud models).
  - Submenu Format (penuh, sesuai StarUML): Font... (Ctrl+Shift+F),
    Fill Color... (Ctrl+Shift+I), Line Color... (Ctrl+Shift+L),
    Line Style, Auto Resize (Ctrl+Shift+R), Show Shadow ✓
    (Ctrl+Shift+H), Inherit Style (Ctrl+Shift+K), Stereotype Display >,
    Word Wrap (Ctrl+Shift+W), Show Visibility (V), Show Namespace (N),
    Show Property (B), Show Type (Y), Show Multiplicity (M),
    Show Operation Signature (J), Show Diagram Name (D),
    Suppress Attributes (A), Suppress Operations (O),
    Suppress Receptions (E), Suppress Literals (T),
    Suppress Property Values, Suppress Constraints, Suppress Parts,
    Suppress Ports, Suppress References, Suppress Values,
    Suppress Properties, Suppress Flow Properties,
    Suppress Columns (Ctrl+Shift+U).
    Catatan: beberapa shortcut StarUML ini bentrok dengan draw.io
    (Ctrl+Shift+F/H/K); draw.io menang.
- **Extension system real**: registry, install/enable, API untuk
  menambah menu/toolbox (saat ini daftar statis).
- **Validation rules lanjutan** di luar cek dasar.
- **Print & HTML export** (StarUML punya). SVG/PNG sudah ada.
- **Search explorer**: sort sudah ada; tambah filter by kind.
- **i18n** (StarUML multi-bahasa).
- **MCP server ala plantuml-mcp/LightCraft**: expose OpenPlant lewat
  Model Context Protocol (stdio + HTTP) supaya agen eksternal
  (OpenCode lokal, Claude) bisa drive aplikasi: `diagram.list`,
  `diagram.add_class`, `diagram.render`, dst. Lihat
  `plantuml/plantuml-mcp` untuk pola transport-nya
  (repo itu masih MVP 1-tool, jangan di-bundle — tiru polanya saja).

Sudah selesai di versi GPUI: undo/redo, cut/copy/paste/duplicate,
multi-diagram (switch, tambah, hapus), PlantUML live sync dua arah,
element picker, styles editor, shortcut draw.io lengkap, minimap
interaktif (klik/drag), export SVG/PNG.

## Workflow

- First build is slow (~5 min: GPUI + wgpu); incremental builds take
  seconds. `target/` is large (~9 GB debug); `cargo clean` reclaims it.
- Visual check: compare against `shots/` at 1600x900.
- Hyprland here uses the Lua config API: `hyprctl dispatch` takes Lua,
  e.g. `hyprctl dispatch 'hl.dsp.window.fullscreen({ mode = "maximized" })'`.
- The old React + Tauri build is archived at
  `~/Documents/OpenPlant-react-tauri-backup-2026-10-16.tar.gz`.
