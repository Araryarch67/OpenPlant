//! Canvas context menu with Add / Add Diagram submenus.

use crate::app::*;
use crate::model::toolbox::DIAGRAM_KINDS;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn context_menu(
        &self,
        m: &CtxMenu,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        const W: f32 = 240.0;
        let vw = f32::from(window.viewport_size().width);
        let x = m.x.min(vw - W - 8.0);
        let sub_x = if x + W + 236.0 > vw {
            x - 234.0
        } else {
            x + W - 4.0
        };
        let has_sel = self.wb.has_selection() || m.shape.is_some();
        let row = |id: &'static str,
                   label: &'static str,
                   kbd: Option<&'static str>,
                   sub: Option<u8>,
                   off: bool,
                   cx: &mut Context<Self>| {
            div()
                .id(id)
                .test_support()
                .flex()
                .items_center()
                .gap_2()
                .px(px(10.))
                .py(px(6.))
                .rounded(px(6.))
                .whitespace_nowrap()
                .cursor_pointer()
                .hover(|s| s.bg(c(HOVER)))
                .when(sub.is_some() && m.sub == sub, |d| d.bg(c(HOVER)))
                .when(off, |d| d.text_color(c(MUTED)))
                .child(label)
                .child(div().flex_1())
                .children(kbd.map(|k| {
                    div()
                        .pl_6()
                        .text_size(px(11.))
                        .text_color(c(MUTED))
                        .child(k)
                }))
                .when(sub.is_some(), |d| d.child(chevron("right", c(MUTED))))
                .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                    if *h
                        && let Some(cm) = &mut this.ctx_menu
                        && cm.sub != sub
                    {
                        cm.sub = sub;
                        cx.notify();
                    }
                }))
        };
        fn act(
            id: &'static str,
            cx: &mut Context<OpenPlant>,
        ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
            cx.listener(move |this, _: &ClickEvent, w, cx| this.run(id, w, cx))
        }
        let panel = popup_panel()
            .w(px(W))
            .p(px(5.))
            .flex()
            .flex_col()
            .child(row("cm-add", "Add", None, Some(0), false, cx))
            .child(row(
                "cm-add-diagram",
                "Add Diagram",
                None,
                Some(1),
                false,
                cx,
            ))
            .child(sep())
            .child(
                row("cm-cut", "Cut", Some("Ctrl+X"), None, !has_sel, cx)
                    .on_click(act("edit.cut", cx)),
            )
            .child(
                row("cm-copy", "Copy", Some("Ctrl+C"), None, !has_sel, cx)
                    .on_click(act("edit.copy", cx)),
            )
            .child(
                row("cm-paste", "Paste", Some("Ctrl+V"), None, false, cx)
                    .on_click(act("edit.paste", cx)),
            )
            .child(
                row("cm-dup", "Duplicate", Some("Ctrl+D"), None, !has_sel, cx)
                    .on_click(act("edit.duplicate", cx)),
            )
            .child(
                row("cm-all", "Select All", Some("Ctrl+A"), None, false, cx)
                    .on_click(act("edit.select-all", cx)),
            )
            .child(
                row("cm-del", "Delete", Some("Del"), None, !has_sel, cx)
                    .on_click(act("edit.delete", cx)),
            )
            .child(
                row(
                    "cm-del-model",
                    "Delete From Model",
                    Some("Ctrl+Del"),
                    None,
                    !has_sel,
                    cx,
                )
                .on_click(act("edit.delete-all", cx)),
            )
            .child(sep())
            .child(
                row("cm-label", "Edit Label", Some("F2"), None, !has_sel, cx)
                    .on_click(act("edit.label", cx)),
            )
            .child(
                row(
                    "cm-front",
                    "To Front",
                    Some("Ctrl+Shift+F"),
                    None,
                    !has_sel,
                    cx,
                )
                .on_click(act("arrange.front", cx)),
            )
            .child(
                row(
                    "cm-back",
                    "To Back",
                    Some("Ctrl+Shift+B"),
                    None,
                    !has_sel,
                    cx,
                )
                .on_click(act("arrange.back", cx)),
            )
            .child(sep())
            .child(
                row("cm-explorer", "Select In Explorer", None, None, false, cx).on_click(
                    cx.listener(|this, _, _, cx| {
                        let id = this
                            .wb
                            .selected_shapes
                            .first()
                            .cloned()
                            .unwrap_or(this.wb.project.active_diagram_id.clone());
                        this.wb.selected_node = Some(id);
                        this.ctx_menu = None;
                        cx.notify();
                    }),
                ),
            )
            .child(
                row("cm-in-diagram", "Select In Diagram", None, None, false, cx)
                    .on_click(act("edit.select-all", cx)),
            );
        let sub = m.sub.map(|s| {
            let (wx, wy) = (m.wx, m.wy);
            let mut sp = popup_panel()
                .id("cm-sub")
                .test_support()
                .min_w(px(230.))
                .p_1()
                .flex()
                .flex_col();
            let y = if s == 0 { m.y + 6.0 } else { m.y + 37.0 };
            if s == 0 {
                for (i, tool) in [
                    "Class",
                    "Interface",
                    "Actor",
                    "Rectangle",
                    "Database",
                    "Package",
                    "Component",
                    "Usecase",
                    "Note",
                ]
                .into_iter()
                .enumerate()
                {
                    sp = sp.child(
                        div()
                            .id(("cm-add-item", i))
                            .test_support()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px(px(10.))
                            .py(px(6.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(|s| s.bg(c(HOVER)))
                            .child(
                                div()
                                    .w(px(18.))
                                    .flex()
                                    .justify_center()
                                    .child(tool_icon(tool, false)),
                            )
                            .child(tool)
                            .on_click(cx.listener(move |this, _, w, cx| {
                                this.ctx_menu = None;
                                let (sw, sh) = crate::model::toolbox::default_size(
                                    crate::model::toolbox::shape_kind(tool),
                                );
                                this.wb.add_shape(
                                    tool,
                                    wx - sw / 2.0,
                                    wy - sh / 2.0,
                                    Some(tool.to_string()),
                                );
                                this.focus_canvas(w, cx);
                                cx.notify();
                            })),
                    );
                }
            } else {
                let vh = f32::from(window.viewport_size().height);
                sp = sp.max_h(px(vh * 0.6)).overflow_y_scroll();
                for (i, k) in DIAGRAM_KINDS.iter().enumerate() {
                    let k = *k;
                    sp = sp.child(
                        div()
                            .id(("cm-diag-item", i))
                            .test_support()
                            .px(px(10.))
                            .py(px(6.))
                            .rounded(px(6.))
                            .cursor_pointer()
                            .hover(|s| s.bg(c(HOVER)))
                            .child(k)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.ctx_menu = None;
                                this.wb.add_diagram(k.trim_end_matches(" Diagram"));
                                cx.notify();
                            })),
                    );
                }
            }
            deferred(
                anchored()
                    .position(point(px(sub_x), px(y)))
                    .snap_to_window()
                    .child(sp),
            )
            .with_priority(4)
        });
        div()
            .child(
                deferred(
                    anchored()
                        .position(point(px(x), px(m.y)))
                        .snap_to_window()
                        .child(panel),
                )
                .with_priority(3),
            )
            .children(sub)
            .into_any_element()
    }
}
