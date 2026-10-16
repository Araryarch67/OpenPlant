//! PlantUML script and markdown documentation sync.

use super::*;
use crate::plantuml::{generate_plantuml, parse_plantuml};
use gpui_kit::*;
use std::time::Duration;

impl OpenPlant {
    pub(crate) fn schedule_puml(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.puml_auto {
            return;
        }
        let text = self.puml_input.read(cx).value().to_string();
        // Text we just wrote from the canvas: nothing to parse.
        if text == self.puml_last_set || text.trim().is_empty() {
            return;
        }
        self.puml_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(700))
                .await;
            let _ = this.update(cx, |this, cx| this.apply_puml(&text, cx));
        }));
    }

    pub fn apply_puml(&mut self, text: &str, cx: &mut Context<Self>) {
        if text == generate_plantuml(self.wb.doc()) {
            return;
        }
        let parsed = parse_plantuml(text);
        if parsed.shapes.is_empty() {
            self.puml_status = "No classes found yet.".into();
            cx.notify();
            return;
        }
        self.puml_status = format!(
            "Rendered {} classes, {} relations.",
            parsed.shapes.len(),
            parsed.relations.len()
        );
        self.wb.replace_from_plantuml(parsed);
        cx.notify();
    }

    /// Canvas -> text: keep the script in step unless the user is typing.
    pub fn sync_puml_from_canvas(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.puml_follow || self.wb.bottom_tab.as_deref() != Some("puml") {
            return;
        }
        let generated = generate_plantuml(self.wb.doc());
        let focused = self.puml_input.read(cx).focus_handle(cx).is_focused(window);
        if !focused && generated != self.puml_last_set {
            self.puml_last_set = generated.clone();
            self.puml_input
                .update(cx, |s, cx| s.set_value(generated, window, cx));
        }
    }

    pub fn load_puml_from_canvas(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let generated = generate_plantuml(self.wb.doc());
        self.puml_last_set = generated.clone();
        self.puml_input
            .update(cx, |s, cx| s.set_value(generated, window, cx));
        self.puml_status = "Loaded from canvas.".into();
    }

    pub fn sync_md(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = &self.wb.project.active_diagram_id;
        let doc = self.wb.node_docs.get(id).cloned().unwrap_or_default();
        if doc != self.md_doc {
            self.md_doc = doc.clone();
            self.md_input
                .update(cx, |s, cx| s.set_value(doc, window, cx));
        }
    }
}
