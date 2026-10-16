//! PlantUML script tab.

use crate::app::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::component::input::Textarea;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn puml_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .mb_2()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        button("puml-load", "Load from Canvas").on_click(cx.listener(
                            |this, _, w, cx| {
                                this.load_puml_from_canvas(w, cx);
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        self.check_label(
                            "puml-auto",
                            self.puml_auto,
                            "Auto-render while typing",
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.puml_auto = !this.puml_auto;
                            cx.notify();
                        })),
                    )
                    .child(
                        self.check_label(
                            "puml-follow",
                            self.puml_follow,
                            "Live update dari canvas",
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.puml_follow = !this.puml_follow;
                            cx.notify();
                        })),
                    )
                    .child(div().text_color(c(MUTED)).child(self.puml_status.clone())),
            )
            .child(
                div().font_family(MONO_FONT).text_size(px(15.5)).child(
                    Textarea::new(&self.puml_input)
                        .h(px(200.))
                        .bg(c(INPUTBG))
                        .border_color(c(LINE)),
                ),
            )
            .into_any_element()
    }
}
