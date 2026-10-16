//! Open, import, save and export through native file dialogs.

use super::*;
use crate::model::*;
use crate::plantuml::parse_plantuml;
use crate::workbench::*;
use gpui_kit::*;
use std::path::PathBuf;

impl OpenPlant {
    pub fn open_file(&mut self, puml_only: bool, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(if puml_only {
                "Import PlantUML".into()
            } else {
                "Open".into()
            }),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = rx.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let contents = std::fs::read_to_string(&path);
            let _ = this.update_in(cx, |this, window, cx| {
                this.load_contents(path, contents, puml_only, window, cx)
            });
        })
        .detach();
    }

    pub fn load_contents(
        &mut self,
        path: PathBuf,
        contents: std::io::Result<String>,
        puml_only: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let text = match contents {
            Ok(t) => t,
            Err(e) => {
                return self.toast(
                    ToastKind::Error,
                    format!("Could not open {name}: {e}"),
                    window,
                    cx,
                );
            }
        };
        if !puml_only && let Ok(p) = serde_json::from_str::<Project>(&text) {
            self.wb.load_project(p);
            self.wb.file_path = Some(path);
            self.pending_fit = true;
            return self.toast(ToastKind::Success, format!("Opened {name}."), window, cx);
        }
        let parsed = parse_plantuml(&text);
        if parsed.shapes.is_empty() {
            return self.toast(
                ToastKind::Error,
                format!("Could not open {name}: no classes found."),
                window,
                cx,
            );
        }
        let n = parsed.shapes.len();
        self.wb.replace_from_plantuml(parsed);
        self.pending_fit = true;
        self.toast(
            ToastKind::Success,
            format!("Imported {n} classes from {name}."),
            window,
            cx,
        );
    }

    pub fn save(&mut self, save_as: bool, window: &mut Window, cx: &mut Context<Self>) {
        let json = match serde_json::to_string_pretty(&self.wb.project) {
            Ok(j) => j,
            Err(e) => return self.toast(ToastKind::Error, format!("Save failed: {e}"), window, cx),
        };
        if !save_as && let Some(path) = self.wb.file_path.clone() {
            return self.write_file(path, json.into_bytes(), "Project saved.", true, window, cx);
        }
        let name = format!("{}.json", self.wb.project.name);
        self.save_dialog(name, json.into_bytes(), "Project saved.", true, window, cx);
    }

    pub fn save_dialog(
        &mut self,
        name: String,
        bytes: Vec<u8>,
        ok: &'static str,
        is_project: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dir = self
            .wb
            .file_path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        let rx = cx.prompt_for_new_path(&dir, Some(&name));
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(path))) = rx.await else { return };
            let _ = this.update_in(cx, |this, window, cx| {
                this.write_file(path, bytes, ok, is_project, window, cx)
            });
        })
        .detach();
    }

    pub(crate) fn write_file(
        &mut self,
        path: PathBuf,
        bytes: Vec<u8>,
        ok: &'static str,
        is_project: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                if is_project {
                    self.wb.file_path = Some(path);
                    self.wb.project.modified = false;
                }
                self.toast(ToastKind::Success, ok, window, cx);
            }
            Err(e) => self.toast(ToastKind::Error, format!("Save failed: {e}"), window, cx),
        }
    }
}
