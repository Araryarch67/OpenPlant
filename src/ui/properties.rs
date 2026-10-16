//! Property editor: schema per node kind, sections, select boxes.

use crate::app::*;
use crate::model::*;
use crate::plantuml::describe_shape;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use crate::workbench::*;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;

#[derive(Clone, Copy, PartialEq)]
pub enum FieldTy {
    Text,
    Ref,
    Dropdown(&'static [&'static str]),
    Check,
}

pub struct Field {
    pub ty: FieldTy,
    pub key: &'static str,
}

pub(crate) const VISIBILITY: &[&str] = &["public", "private", "protected", "package"];

pub const FONTS: &[&str] = &[
    "Arial, sans-serif",
    "Georgia, serif",
    "Times New Roman, serif",
    "Courier New, monospace",
    "Verdana, sans-serif",
];

pub(crate) const fn f(ty: FieldTy, key: &'static str) -> Field {
    Field { ty, key }
}

pub(crate) const S_PROJECT: &[Field] = &[
    f(FieldTy::Text, "name"),
    f(FieldTy::Text, "author"),
    f(FieldTy::Text, "company"),
    f(FieldTy::Text, "copyright"),
    f(FieldTy::Text, "version"),
    f(FieldTy::Text, "documentVersion"),
];

pub(crate) const S_MODEL: &[Field] = &[
    f(FieldTy::Text, "name"),
    f(FieldTy::Ref, "stereotype"),
    f(FieldTy::Dropdown(VISIBILITY), "visibility"),
    f(FieldTy::Ref, "importedElements"),
    f(FieldTy::Text, "viewpoint"),
];

pub(crate) const S_PACKAGE: &[Field] = &[
    f(FieldTy::Text, "name"),
    f(FieldTy::Ref, "stereotype"),
    f(FieldTy::Dropdown(VISIBILITY), "visibility"),
];

pub(crate) const S_CLASS: &[Field] = &[
    f(FieldTy::Text, "name"),
    f(FieldTy::Ref, "stereotype"),
    f(FieldTy::Dropdown(VISIBILITY), "visibility"),
    f(FieldTy::Check, "isAbstract"),
];

pub(crate) const S_DIAGRAM: &[Field] = &[f(FieldTy::Text, "name")];

pub fn schema(kind: NodeKind) -> &'static [Field] {
    match kind {
        NodeKind::Project => S_PROJECT,
        NodeKind::Model => S_MODEL,
        NodeKind::Package | NodeKind::Interface => S_PACKAGE,
        NodeKind::Class => S_CLASS,
        NodeKind::Diagram => S_DIAGRAM,
    }
}

impl OpenPlant {
    pub(crate) fn section(
        &self,
        key: &'static str,
        title: &'static str,
        body: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = !self.closed_sections.contains(key);
        div()
            .mb(px(6.))
            .child(
                div()
                    .id(eid(format!("sec-{key}")))
                    .test_support()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px_1()
                    .py(px(6.))
                    .cursor_pointer()
                    .child(
                        div()
                            .w(px(14.))
                            .flex()
                            .justify_center()
                            .child(chevron(if open { "down" } else { "right" }, c(MUTED))),
                    )
                    .child(title)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.closed_sections.remove(key) {
                            this.closed_sections.insert(key);
                        }
                        cx.notify();
                    })),
            )
            .when(open, |d| {
                d.child(div().px_1().pt(px(2.)).pb_2().child(body))
            })
            .into_any_element()
    }

    pub(crate) fn property_editor(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let _ = window;
        let node = self
            .wb
            .selected_node
            .as_ref()
            .and_then(|n| self.wb.project.root.find(n));
        let Some(node) = node else {
            return div()
                .text_color(c(MUTED))
                .italic()
                .child("No selection")
                .into_any_element();
        };
        let id = node.id.clone();
        let mut props = div().flex().flex_col();
        for fd in schema(node.kind) {
            let key = fd.key;
            let row = div()
                .mb_2()
                .flex()
                .items_center()
                .gap_2()
                .child(label_cell(key));
            let input = self
                .prop
                .fields
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, e)| e.clone());
            let row = match fd.ty {
                FieldTy::Text => {
                    row.children(input.map(|e| div().flex_1().min_w_0().child(field(&e))))
                }
                FieldTy::Ref => row
                    .children(input.map(|e| div().flex_1().min_w_0().child(field(&e))))
                    .child(
                        icon_button(eid(format!("pick-{key}")))
                            .child(magnifier(c(FG)))
                            .tooltip(|w, cx| Tooltip::new("Select element").build(w, cx))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.picker_for = Some(key);
                                cx.notify();
                            })),
                    ),
                FieldTy::Dropdown(opts) => {
                    let v = self
                        .wb
                        .attr_text(&id, key)
                        .unwrap_or_else(|| "public".into());
                    row.child(self.select_box(key, v, opts, cx))
                }
                FieldTy::Check => {
                    let on = self.wb.attr_bool(&id, key);
                    let nid_ = id.clone();
                    row.child(
                        checkbox(eid(format!("chk-{key}")), on).on_click(cx.listener(
                            move |this, _, _, cx| {
                                this.wb.set_attr(&nid_, key, AttrValue::Bool(!on));
                                cx.notify();
                            },
                        )),
                    )
                }
            };
            props = props.child(row);
        }
        let mut out = div().flex().flex_col().child(self.section(
            "props",
            "Properties",
            props.into_any_element(),
            cx,
        ));
        if let Some(doc) = self.prop.doc.clone() {
            let body = div()
                .font_family(MONO_FONT)
                .text_size(px(15.))
                .child(
                    Textarea::new(&doc)
                        .h(px(130.))
                        .bg(c(INPUTBG))
                        .border_color(c(LINE)),
                )
                .into_any_element();
            out = out.child(self.section("docs", "Documentation", body, cx));
        }
        let shape = self
            .wb
            .doc()
            .shapes
            .iter()
            .find(|s| self.wb.selected_shapes.contains(&s.id))
            .cloned();
        if let Some(s) = &shape {
            let decl = describe_shape(s);
            let kw = decl.split(' ').next().unwrap_or("").to_string();
            let body = div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .mb_2()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(label_cell("keyword"))
                        .child(
                            div()
                                .flex_1()
                                .px(px(10.))
                                .py(px(7.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(c(LINE))
                                .bg(c(INPUTBG))
                                .font_family(MONO_FONT)
                                .text_xs()
                                .child(kw),
                        ),
                )
                .child(
                    div()
                        .px(px(10.))
                        .py_2()
                        .rounded(px(6.))
                        .border_1()
                        .border_color(c(LINE))
                        .bg(c(INPUTBG))
                        .font_family(MONO_FONT)
                        .text_xs()
                        .child(decl),
                )
                .into_any_element();
            out = out.child(self.section("puml", "PlantUML", body, cx));
            out = out.child(self.styles_section(s, cx));
        }
        out.into_any_element()
    }

    pub(crate) fn select_box(
        &self,
        key: &'static str,
        value: String,
        opts: &'static [&'static str],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = self.open_dropdown == Some(key);
        let list = open.then(|| {
            let mut panel = popup_panel().min_w(px(180.)).p_1().flex().flex_col();
            for (i, o) in opts.iter().enumerate() {
                let o = *o;
                panel = panel.child(
                    div()
                        .id(eid(format!("opt-{key}-{i}")))
                        .test_support()
                        .px_2()
                        .py(px(4.))
                        .rounded(px(6.))
                        .cursor_pointer()
                        .hover(|s| s.bg(c(HOVER)))
                        .when(o == value, |d| d.bg(sel_bg()))
                        .child(o)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.pick_option(key, o);
                            this.open_dropdown = None;
                            cx.notify();
                        })),
                );
            }
            div()
                .absolute()
                .top(px(36.))
                .left_0()
                .child(deferred(anchored().snap_to_window().child(panel)).with_priority(2))
        });
        div()
            .relative()
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .id(eid(format!("sel-{key}")))
                    .test_support()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px(px(10.))
                    .py(px(7.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(if open { c(BLUE) } else { c(LINE) })
                    .bg(c(INPUTBG))
                    .cursor_pointer()
                    .child(div().flex_1().truncate().child(value.clone()))
                    .child(chevron("down", c(MUTED)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_dropdown = if this.open_dropdown == Some(key) {
                            None
                        } else {
                            Some(key)
                        };
                        cx.notify();
                    })),
            )
            .children(list)
            .into_any_element()
    }

    pub fn pick_option(&mut self, key: &str, value: &str) {
        match key {
            "font" => {
                let v = value.to_string();
                self.wb.set_style(|s| s.font = Some(v.clone()));
            }
            "ai-model" => self.ai.local_model = value.to_string(),
            k => {
                if let Some(id) = self.wb.selected_node.clone() {
                    self.wb.set_attr(&id, k, AttrValue::Text(value.to_string()));
                }
            }
        }
    }
}
