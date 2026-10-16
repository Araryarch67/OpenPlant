//! Working diagrams, toolbox and panel resizers.

use crate::app::*;
use crate::model::toolbox::{TOOL_GROUPS, ToolKind};
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let diagrams = self.wb.diagrams_sorted();
        let active = self.wb.project.active_diagram_id.clone();
        let many = diagrams.len() > 1;
        let rows = diagrams.into_iter().map(|(id, name)| {
            let is_active = id == active;
            let (a, b) = (id.clone(), id.clone());
            div()
                .id(eid(format!("wd-{id}")))
                .test_support()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py(px(5.))
                .cursor_pointer()
                .hover(|s| s.bg(c(HOVER)))
                .when(is_active, |d| d.bg(sel_bg()))
                .on_click(cx.listener(move |this, _, w, cx| {
                    this.wb.activate_diagram(&a);
                    this.wb.selected_node = Some(a.clone());
                    this.pending_fit = true;
                    this.focus_canvas(w, cx);
                    cx.notify();
                }))
                .child(diagram_doc_icon())
                .child(
                    div()
                        .truncate()
                        .when(is_active, |d| d.text_color(c(BLUE)))
                        .child(name.replace('\n', " ")),
                )
                .child(div().text_color(c(MUTED)).child("- Model"))
                .when(many, |d| {
                    d.child(div().flex_1()).child(
                        div()
                            .id(eid(format!("wdx-{b}")))
                            .test_support()
                            .px_1()
                            .rounded(px(6.))
                            .text_color(c(MUTED))
                            .hover(|s| s.bg(c(HOVER)).text_color(c(RED)))
                            .child("×")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.wb.delete_diagram(&b);
                                cx.notify();
                            })),
                    )
                })
        });
        let tool = self.wb.active_tool.clone();
        let mut groups = div()
            .id("toolbox")
            .test_support()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        for (i, g) in TOOL_GROUPS.iter().enumerate() {
            let open = self.wb.open_groups.get(i).copied().unwrap_or(false);
            groups = groups.child(
                div()
                    .id(("grp", i))
                    .test_support()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py(px(5.))
                    .cursor_pointer()
                    .hover(|s| s.bg(c(HOVER)))
                    .child(div().flex_1().truncate().child(g.name))
                    .child(chevron(if open { "up" } else { "down" }, c(MUTED)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(v) = this.wb.open_groups.get_mut(i) {
                            *v = !*v;
                        }
                        cx.notify();
                    })),
            );
            if !open {
                continue;
            }
            for (j, t) in g.tools.iter().enumerate() {
                let label = t.label;
                let edge = t.kind == ToolKind::Edge;
                groups = groups.child(
                    div()
                        .id(eid(format!("tool-{i}-{j}")))
                        .test_support()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py(px(5.))
                        .cursor_pointer()
                        .hover(|s| s.bg(c(HOVER)))
                        .when(tool == label, |d| d.bg(sel_bg()))
                        .child(
                            div()
                                .w(px(22.))
                                .flex()
                                .justify_center()
                                .child(tool_icon(label, edge)),
                        )
                        .child(div().flex_1().truncate().child(label))
                        .tooltip(move |w, cx| {
                            Tooltip::new(if edge {
                                "Klik lalu klik 2 shape — atau drag ke canvas"
                            } else {
                                "Klik lalu klik canvas — atau drag ke canvas"
                            })
                            .build(w, cx)
                        })
                        .on_drag(ToolDrag(label.to_string()), |d, _, _, cx| {
                            cx.new(|_| DragGhost(d.0.clone()))
                        })
                        .on_click(cx.listener(move |this, _, w, cx| {
                            this.wb.active_tool = label.to_string();
                            this.pending_from = None;
                            this.focus_canvas(w, cx);
                            cx.notify();
                        })),
                );
            }
        }
        div()
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(self.wb.sidebar_w))
            .h_full()
            .bg(c(PANEL))
            .child(header("WORKING DIAGRAMS"))
            .child(
                div()
                    .id("working")
                    .test_support()
                    .flex_none()
                    .h(px(150.))
                    .overflow_y_scroll()
                    .children(rows),
            )
            .child(header("TOOLBOX").mt(px(18.)))
            .child(
                div().flex().justify_center().pt(px(2.)).pb(px(6.)).child(
                    icon_button("tool-select")
                        .child(cursor_arrow())
                        .tooltip(|w, cx| Tooltip::new("Select").build(w, cx))
                        .on_click(cx.listener(|this, _, w, cx| {
                            this.wb.active_tool = "Select".into();
                            this.pending_from = None;
                            this.focus_canvas(w, cx);
                            cx.notify();
                        })),
                ),
            )
            .child(groups)
            .child(self.resizer(true, cx))
            .into_any_element()
    }

    pub(crate) fn resizer(&self, left: bool, cx: &mut Context<Self>) -> Btn {
        div()
            .id(if left { "resize-left" } else { "resize-right" })
            .test_support()
            .absolute()
            .top_0()
            .bottom_0()
            .w(px(7.))
            .when(left, |d| d.right(px(-3.)))
            .when(!left, |d| d.left(px(-3.)))
            .cursor(CursorStyle::ResizeLeftRight)
            .hover(|s| s.bg(ca(BLUE, 0.35)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    if ev.click_count >= 2 {
                        if left {
                            this.wb.sidebar_w = 264.0;
                        } else {
                            this.wb.navigator_w = 300.0;
                        }
                        this.settings.sidebar_w = this.wb.sidebar_w;
                        this.settings.navigator_w = this.wb.navigator_w;
                        this.settings.save();
                    } else {
                        let w0 = if left {
                            this.wb.sidebar_w
                        } else {
                            this.wb.navigator_w
                        };
                        this.drag = Drag::Splitter {
                            left,
                            sx: f32::from(ev.position.x),
                            w0,
                        };
                    }
                    cx.notify();
                }),
            )
    }
}
