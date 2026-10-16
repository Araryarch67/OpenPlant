//! Command palette.

use crate::app::*;
use crate::commands::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn palette_view(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(p) = &self.palette else {
            return div().into_any_element();
        };
        let q = p.input.read(cx).value().to_lowercase();
        let list = palette_matches(&q);
        let idx = p.idx.min(list.len().saturating_sub(1));
        let rows = list
            .iter()
            .enumerate()
            .map(|(i, c_)| {
                let id = c_.id;
                div()
                    .id(("pal", i))
                    .test_support()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py(px(5.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .when(i == idx, |d| d.bg(sel_bg()))
                    .child(c_.label)
                    .child(
                        div()
                            .ml_auto()
                            .text_size(px(11.))
                            .text_color(c(MUTED))
                            .child(c_.id),
                    )
                    .on_hover(cx.listener(move |this, h: &bool, _, cx| {
                        if *h && let Some(p) = &mut this.palette {
                            p.idx = i;
                            cx.notify();
                        }
                    }))
                    .on_click(cx.listener(move |this, _, w, cx| {
                        this.palette = None;
                        this.focus_canvas(w, cx);
                        this.run(id, w, cx);
                    }))
            })
            .collect::<Vec<_>>();
        let count = list.len();
        self.modal_backdrop("palette-backdrop", window, cx)
            .child(
                div()
                    .id("palette")
                    .test_support()
                    .w(px(560.))
                    .p(px(10.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(LINE))
                    .bg(c(RAISED))
                    .flex()
                    .flex_col()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .capture_key_down(cx.listener(move |this, ev: &KeyDownEvent, w, cx| {
                        let key = ev.keystroke.key.as_str();
                        match key {
                            "escape" => {
                                this.palette = None;
                                this.focus_canvas(w, cx);
                            }
                            "down" | "up" => {
                                if let Some(p) = &mut this.palette {
                                    p.idx = if key == "down" {
                                        (p.idx + 1).min(count.saturating_sub(1))
                                    } else {
                                        p.idx.saturating_sub(1)
                                    };
                                }
                            }
                            "enter" => {
                                let q = this
                                    .palette
                                    .as_ref()
                                    .map(|p| p.input.read(cx).value().to_lowercase())
                                    .unwrap_or_default();
                                let sel = this.palette.as_ref().map(|p| p.idx).unwrap_or(0);
                                this.palette = None;
                                this.focus_canvas(w, cx);
                                if let Some(c_) = palette_matches(&q).get(sel) {
                                    this.run(c_.id, w, cx);
                                }
                            }
                            _ => return,
                        }
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .child(field(&p.input).border_color(c(BLUE)))
                    .child(
                        div()
                            .id("pal-list")
                            .test_support()
                            .mt_2()
                            .max_h(px(300.))
                            .overflow_y_scroll()
                            .children(rows),
                    ),
            )
            .into_any_element()
    }
}
