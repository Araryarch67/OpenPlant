//! Element picker dialog.

use crate::app::*;
use crate::model::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use crate::workbench::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn picker_view(
        &self,
        key: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut entries: Vec<(String, String, &'static str)> = vec![];
        fn walk(n: &ModelNode, out: &mut Vec<(String, String, &'static str)>) {
            let kind = match n.kind {
                NodeKind::Project => "project",
                NodeKind::Model => "model",
                NodeKind::Package => "package",
                NodeKind::Class => "class",
                NodeKind::Interface => "interface",
                NodeKind::Diagram => "diagram",
            };
            out.push((n.id.clone(), n.name.clone(), kind));
            n.children.iter().for_each(|c| walk(c, out));
        }
        walk(&self.wb.project.root, &mut entries);
        for d in self.wb.project.diagrams.values() {
            for s in &d.shapes {
                if !entries.iter().any(|e| e.0 == s.id) {
                    entries.push((s.id.clone(), s.label.clone(), "class"));
                }
            }
        }
        self.modal_backdrop("picker-backdrop", window, cx)
            .child(
                div()
                    .id("picker")
                    .test_support()
                    .w(px(480.))
                    .max_h(px(f32::from(window.viewport_size().height) * 0.6))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(LINE))
                    .bg(c(RAISED))
                    .flex()
                    .flex_col()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px(px(14.))
                            .py(px(10.))
                            .border_b_1()
                            .border_color(c(LINE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Select element")
                            .child(div().flex_1())
                            .child(icon_button("picker-x").child(close_x(c(FG))).on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.picker_for = None;
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .id("picker-list")
                            .test_support()
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p(px(6.))
                            .children(entries.into_iter().enumerate().map(
                                |(i, (_, name, kind))| {
                                    let n2 = name.replace('\n', " ");
                                    div()
                                        .id(("pick", i))
                                        .test_support()
                                        .flex()
                                        .gap_2()
                                        .px(px(10.))
                                        .py(px(6.))
                                        .rounded(px(6.))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(c(HOVER)))
                                        .child(
                                            div().text_color(c(MUTED)).child(format!("[{kind}]")),
                                        )
                                        .child(n2.clone())
                                        .on_click(cx.listener(move |this, _, w, cx| {
                                            this.picker_for = None;
                                            if let Some(id) = this.wb.selected_node.clone() {
                                                this.wb.set_attr(
                                                    &id,
                                                    key,
                                                    AttrValue::Text(n2.clone()),
                                                );
                                            }
                                            if let Some((_, inp)) =
                                                this.prop.fields.iter().find(|(k, _)| *k == key)
                                            {
                                                let v = n2.clone();
                                                inp.update(cx, |s, cx| s.set_value(v, w, cx));
                                            }
                                            cx.notify();
                                        }))
                                },
                            )),
                    ),
            )
            .into_any_element()
    }
}
