//! Keyboard shortcut sheet.

use crate::app::*;
use crate::commands::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn shortcuts_view(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let col = |sec: &(&'static str, &'static [(&'static str, &'static str)])| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .mb_1()
                        .text_size(px(11.))
                        .text_color(c(MUTED))
                        .child(sec.0.to_uppercase()),
                )
                .children(sec.1.iter().map(|(k, d)| {
                    div()
                        .flex()
                        .gap_3()
                        .child(
                            div()
                                .w(px(220.))
                                .flex_none()
                                .font_family(MONO_FONT)
                                .text_xs()
                                .text_color(c(BLUE))
                                .child(*k),
                        )
                        .child(div().text_xs().child(*d))
                }))
        };
        self.modal_backdrop("shortcuts-backdrop", window, cx)
            .child(
                div()
                    .id("shortcuts")
                    .test_support()
                    .w(px(820.))
                    .max_h(px(f32::from(window.viewport_size().height) * 0.76))
                    .overflow_y_scroll()
                    .p_4()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(LINE))
                    .bg(c(RAISED))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .mb_3()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Keyboard Shortcuts"),
                            )
                            .child(
                                div()
                                    .ml_2()
                                    .text_xs()
                                    .text_color(c(MUTED))
                                    .child("draw.io compatible"),
                            )
                            .child(div().flex_1())
                            .child(icon_button("shortcuts-x").child(close_x(c(FG))).on_click(
                                cx.listener(|this, _, w, cx| {
                                    this.shortcuts_open = false;
                                    this.focus_canvas(w, cx);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_6()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .children(SHORTCUTS.iter().step_by(2).map(col)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .children(SHORTCUTS.iter().skip(1).step_by(2).map(col)),
                            ),
                    ),
            )
            .into_any_element()
    }
}
