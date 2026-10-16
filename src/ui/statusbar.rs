//! Status bar.

use crate::app::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn statusbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let doc = self.wb.doc();
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(6.))
            .px(px(10.))
            .py_1()
            .bg(c(PANEL))
            .border_t_1()
            .border_color(c(LINE))
            .child(
                icon_button("st-zoom-out")
                    .child("-")
                    .on_click(cx.listener(|this, _, w, cx| this.run("view.zoom-out", w, cx))),
            )
            .child(
                icon_button("st-zoom-in")
                    .child("+")
                    .on_click(cx.listener(|this, _, w, cx| this.run("view.zoom-in", w, cx))),
            )
            .child(
                div()
                    .ml(px(6.))
                    .text_color(c(MUTED))
                    .child(format!("{} - {} Diagram", doc.name, doc.kind)),
            )
            .child(div().flex_1())
            .child(div().size(px(10.)).rounded_full().bg(c(GREEN)))
            .child(format!("{}%", (self.wb.zoom * 100.0).round()))
            .into_any_element()
    }
}
