//! Canvas element: painter, input wiring, tool hint, label editor.

use crate::app::*;
use crate::canvas::*;
use crate::commands::*;
use crate::model::*;
use crate::theme::*;
use gpui_kit::component::input::Input;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn canvas_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let doc = self.wb.doc().clone();
        let idle = matches!(self.drag, Drag::None);
        let ov = Overlay {
            selected: self.wb.selected_shapes.clone(),
            selected_rel: self.wb.selected_relations.clone(),
            hover: self.hover.clone(),
            pending_from: self.pending_from.clone(),
            rubber: match &self.drag {
                Drag::Rubber { sx, sy, cx, cy, .. } if (cx - sx).abs() + (cy - sy).abs() > 3.0 => {
                    Some((*sx, *sy, *cx, *cy))
                }
                _ => None,
            },
            connect: match &self.drag {
                Drag::Connect {
                    from,
                    cx,
                    cy,
                    target,
                } => Some((from.clone(), *cx, *cy, target.clone())),
                _ => None,
            },
            show_arrows: idle && self.wb.active_tool == "Select" && self.editing.is_none(),
            grid: self.wb.show_grid,
            minimap: self.wb.show_minimap && self.wb.ext_on("thumbs"),
            editing: self.editing.as_ref().map(|e| match &e.target {
                EditTarget::Shape(id) | EditTarget::Relation(id) => id.clone(),
            }),
        };
        let (pan, k) = (self.wb.pan, self.wb.zoom);
        let cell = self.canvas_bounds.clone();
        let entity = cx.entity();
        let painter = canvas(
            move |b, window, _| {
                let now = (
                    f32::from(b.origin.x),
                    f32::from(b.origin.y),
                    f32::from(b.size.width),
                    f32::from(b.size.height),
                );
                // A resized canvas re-renders once so view state (auto-fit,
                // hit testing) sees the new bounds.
                if cell.replace(now) != now {
                    window.request_animation_frame();
                }
            },
            move |b, _, window, cx| {
                let xf = Xf {
                    ox: f32::from(b.origin.x),
                    oy: f32::from(b.origin.y),
                    pan_x: pan.0,
                    pan_y: pan.1,
                    k,
                };
                window.with_content_mask(Some(ContentMask { bounds: b }), |window| {
                    paint_canvas(&doc, &xf, b, &ov, window, cx)
                });
                // Window-wide listeners keep drags alive outside the canvas.
                let e = entity.clone();
                window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble {
                        e.update(cx, |this, cx| this.root_move(ev, window, cx));
                    }
                });
                let e = entity.clone();
                window.on_mouse_event(move |ev: &MouseUpEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble {
                        e.update(cx, |this, cx| this.root_up(ev, window, cx));
                    }
                });
            },
        )
        .size_full();

        let tool = self.wb.active_tool.clone();
        let hint = (tool != "Select").then(|| {
            let text = if crate::model::toolbox::is_edge_tool(&tool) {
                if self.pending_from.is_some() {
                    format!("{tool}: klik shape tujuan (Esc batal)")
                } else {
                    format!("{tool}: klik shape sumber (Esc batal)")
                }
            } else {
                format!("{tool}: klik canvas untuk menaruh (Esc selesai)")
            };
            div()
                .absolute()
                .top(px(12.))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .child(
                    div()
                        .id("tool-hint")
                        .test_support()
                        .flex()
                        .items_center()
                        .gap_2()
                        .rounded_full()
                        .border_1()
                        .border_color(c(CHIP))
                        .bg(ca(PANEL, 0.9))
                        .px_4()
                        .py(px(6.))
                        .text_xs()
                        .text_color(c(PAPER))
                        .shadow_lg()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(text)
                        .child(
                            div()
                                .id("tool-hint-x")
                                .test_support()
                                .px_1()
                                .cursor_pointer()
                                .font_weight(FontWeight::BOLD)
                                .text_color(c(YELLOW))
                                .child("✕")
                                .on_click(cx.listener(|this, _, w, cx| {
                                    this.pending_from = None;
                                    this.wb.active_tool = "Select".into();
                                    this.focus_canvas(w, cx);
                                    cx.notify();
                                })),
                        ),
                )
        });
        let editor = self.editing.as_ref().and_then(|ed| {
            let xf = self.xf();
            let (cx0, cy0, _, _) = self.canvas_bounds.get();
            let doc = self.wb.doc();
            let (x, y, w) = match &ed.target {
                EditTarget::Shape(id) => {
                    let s = doc.shape(id)?;
                    let top = match s.kind() {
                        ShapeKind::Class | ShapeKind::Interface | ShapeKind::Component => s.y + 2.0,
                        ShapeKind::Package => s.y + 26.0,
                        ShapeKind::Actor => s.y + s.h - 40.0,
                        _ => s.y + s.h / 2.0 - 16.0 / xf.k,
                    };
                    (xf.sx(s.x), xf.sy(top), (s.w * xf.k).max(150.0))
                }
                EditTarget::Relation(id) => {
                    let r = doc.relations.iter().find(|r| &r.id == id)?;
                    let route = ortho_route(doc.shape(&r.from)?, doc.shape(&r.to)?);
                    (
                        xf.sx(route.label.0) - 80.0,
                        xf.sy(route.label.1) - 18.0,
                        160.0,
                    )
                }
            };
            Some(
                div()
                    .id("label-editor")
                    .test_support()
                    .absolute()
                    .left(px(x - cx0))
                    .top(px(y - cy0))
                    .w(px(w))
                    .capture_key_down(cx.listener(|this, ev: &KeyDownEvent, w, cx| {
                        if ev.keystroke.key == "escape" {
                            cx.stop_propagation();
                            this.editing = None;
                            this.focus_canvas(w, cx);
                            cx.notify();
                        }
                    }))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        Input::new(&ed.input)
                            .bg(c(0xffffff))
                            .text_color(c(0x111111))
                            .border_color(c(BLUE)),
                    ),
            )
        });
        div()
            .id("diagram-wrap")
            .test_support()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_hidden()
            .bg(c(PAPER))
            .key_context(CANVAS)
            .track_focus(&self.canvas_focus)
            .cursor(self.cursor)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev, w, cx| this.canvas_down(ev, w, cx)),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, ev, w, cx| this.canvas_down(ev, w, cx)),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, ev, w, cx| this.canvas_down(ev, w, cx)),
            )
            .on_scroll_wheel(cx.listener(|this, ev, w, cx| this.canvas_wheel(ev, w, cx)))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, w, cx| {
                if ev.keystroke.key == "space"
                    && this.canvas_focus.is_focused(w)
                    && !this.space_down
                {
                    this.space_down = true;
                    this.cursor = CursorStyle::OpenHand;
                    cx.notify();
                }
            }))
            .on_key_up(cx.listener(|this, ev: &KeyUpEvent, _, cx| {
                if ev.keystroke.key == "space" {
                    this.space_down = false;
                    this.cursor = CursorStyle::Arrow;
                    cx.notify();
                }
            }))
            .on_drop(cx.listener(|this, d: &ToolDrag, w, cx| {
                let tool = d.0.clone();
                this.drop_tool(&tool, w, cx);
            }))
            .child(painter)
            .children(hint)
            .children(editor)
            .into_any_element()
    }
}
