//! Property editor inputs bound to the selected model node.

use super::*;
use crate::model::*;
use crate::workbench::*;
use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::*;

impl OpenPlant {
    /// (Re)build property inputs when the selected node changes.
    pub fn ensure_prop_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let node = self.wb.selected_node.clone();
        let kind = node
            .as_ref()
            .and_then(|n| self.wb.project.root.find(n))
            .map(|n| n.kind);
        let Some(kind) = kind else {
            self.prop = PropInputs {
                node: None,
                fields: vec![],
                doc: None,
                font_size: None,
                _subs: vec![],
            };
            return;
        };
        let id = node.clone().unwrap();
        if self.prop.node.as_deref() == Some(id.as_str()) {
            // Reflect renames made elsewhere (canvas, script) into idle fields.
            let name = self
                .wb
                .project
                .root
                .find(&id)
                .map(|n| n.name.clone())
                .unwrap_or_default();
            if let Some((_, inp)) = self.prop.fields.iter().find(|(k, _)| *k == "name") {
                let st = inp.read(cx);
                if !st.focus_handle(cx).is_focused(window) && st.value() != name.as_str() {
                    inp.update(cx, |s, cx| s.set_value(name, window, cx));
                }
            }
            return;
        }
        let mut fields = vec![];
        let mut subs = vec![];
        for f in crate::ui::schema(kind) {
            if !matches!(f.ty, crate::ui::FieldTy::Text | crate::ui::FieldTy::Ref) {
                continue;
            }
            let value = if f.key == "name" {
                self.wb
                    .project
                    .root
                    .find(&id)
                    .map(|n| n.name.clone())
                    .unwrap_or_default()
            } else {
                self.wb.attr_text(&id, f.key).unwrap_or_else(|| {
                    if f.ty == crate::ui::FieldTy::Ref {
                        "—".into()
                    } else {
                        String::new()
                    }
                })
            };
            let key = f.key;
            let inp = cx.new(|cx| InputState::new(window, cx).default_value(value));
            let nid_ = id.clone();
            subs.push(
                cx.subscribe_in(
                    &inp,
                    window,
                    move |this, inp, ev: &InputEvent, _, cx| match ev {
                        InputEvent::Focus => this.wb.checkpoint(),
                        InputEvent::Change => {
                            let v = inp.read(cx).value().to_string();
                            if key == "name" {
                                this.wb.rename_node(&nid_, &v);
                            } else {
                                this.wb.set_attr(&nid_, key, AttrValue::Text(v));
                            }
                            cx.notify();
                        }
                        _ => {}
                    },
                ),
            );
            fields.push((key, inp));
        }
        let doc_text = self.wb.node_docs.get(&id).cloned().unwrap_or_default();
        let doc = cx.new(|cx| TextareaState::new(window, cx).default_value(doc_text));
        let nid_ = id.clone();
        subs.push(
            cx.subscribe_in(&doc, window, move |this, inp, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.wb
                        .node_docs
                        .insert(nid_.clone(), inp.read(cx).value().to_string());
                }
            }),
        );
        let fs = self
            .wb
            .doc()
            .shape(&id)
            .and_then(|s| s.style.as_ref().and_then(|st| st.font_size))
            .unwrap_or(DEFAULT_FONT_SIZE);
        let font_size = cx.new(|cx| InputState::new(window, cx).default_value(format!("{fs}")));
        subs.push(
            cx.subscribe_in(&font_size, window, |this, inp, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change)
                    && let Ok(n) = inp.read(cx).value().trim().parse::<f32>()
                {
                    let n = n.clamp(8.0, 72.0);
                    this.wb.set_style(|st| st.font_size = Some(n));
                    cx.notify();
                }
            }),
        );
        self.prop = PropInputs {
            node: Some(id),
            fields,
            doc: Some(doc),
            font_size: Some(font_size),
            _subs: subs,
        };
    }
}
