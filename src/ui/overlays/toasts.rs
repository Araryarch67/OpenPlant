//! Toast notifications.

use crate::app::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use crate::workbench::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn toasts(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .absolute()
            .top(px(44.))
            .right(px(16.))
            .w(px(360.))
            .flex()
            .flex_col()
            .gap_2()
            .children(self.wb.toasts.iter().map(|t| {
                let color = match t.kind {
                    ToastKind::Error => RED,
                    ToastKind::Success => GREEN,
                    ToastKind::Info => BLUE,
                };
                let id = t.id;
                div()
                    .id(eid(format!("toast-{id}")))
                    .test_support()
                    .flex()
                    .overflow_hidden()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(c(LINE))
                    .bg(c(RAISED))
                    .shadow_lg()
                    .occlude()
                    .child(div().w(px(3.)).flex_none().bg(c(color)))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .gap(px(10.))
                            .px_3()
                            .py(px(10.))
                            .child(div().size(px(9.)).flex_none().rounded_full().bg(c(color)))
                            .child(div().flex_1().min_w_0().child(t.msg.clone()))
                            .child(
                                div()
                                    .id(eid(format!("toast-x-{id}")))
                                    .test_support()
                                    .p(px(2.))
                                    .rounded(px(6.))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(c(HOVER)))
                                    .child(close_x(c(MUTED)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.wb.dismiss_toast(id);
                                        cx.notify();
                                    })),
                            ),
                    )
            }))
            .into_any_element()
    }
}
