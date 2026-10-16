//! Bottom drawer frame and its tabs.

use crate::app::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn drawer(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tab = self.wb.bottom_tab.clone().unwrap_or_default();
        let title = match tab.as_str() {
            "ext" => "EXTENSIONS",
            "diagrams" => "DIAGRAM THUMBNAILS",
            "model" => "RELATIONSHIPS",
            "nav" => "MARKDOWN DOCUMENTATION",
            "share" => "MODEL VALIDATION",
            "puml" => "PLANTUML SCRIPT",
            "ai" => "AI DIAGRAM GENERATOR",
            _ => "",
        };
        let tall = tab == "puml" || tab == "ai";
        let gate =
            |id: &'static str,
             name: &'static str,
             body: AnyElement,
             this: &Self,
             cx: &mut Context<Self>| {
                if this.wb.ext_on(id) {
                    body
                } else {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_color(c(MUTED))
                        .italic()
                        .child(format!("{name} is disabled."))
                        .child(button_sm(eid(format!("enable-{id}")), "Enable").on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.wb.extensions.insert(id.into(), true);
                                cx.notify();
                            }),
                        ))
                        .into_any_element()
                }
            };
        let body = match tab.as_str() {
            "ext" => self.extensions_tab(cx),
            "diagrams" => {
                let b = self.thumbs_tab(cx);
                gate("thumbs", "Diagram Thumbnails", b, self, cx)
            }
            "model" => self.relations_tab(),
            "nav" => {
                let b = self.markdown_tab(cx);
                gate("markdown", "Markdown Documentation", b, self, cx)
            }
            "share" => {
                let b = self.validation_tab();
                gate("validation", "Model Validation", b, self, cx)
            }
            "puml" => {
                let b = self.puml_tab(cx);
                gate("puml", "PlantUML Script", b, self, cx)
            }
            "ai" => {
                let b = self.ai_tab(window, cx);
                gate("ai", "AI Diagram Generator", b, self, cx)
            }
            _ => div().into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .h(px(if tall { 330. } else { 220. }))
            .bg(c(PANEL))
            .border_t_1()
            .border_color(c(LINE))
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_3()
                    .py(px(6.))
                    .text_size(px(11.))
                    .text_color(c(MUTED))
                    .child(title)
                    .child(div().flex_1())
                    .child(
                        icon_button("drawer-close")
                            .child(close_x(c(FG)))
                            .tooltip(|w, cx| Tooltip::new("Close panel").build(w, cx))
                            .on_click(cx.listener(|this, _, w, cx| {
                                this.wb.bottom_tab = None;
                                this.focus_canvas(w, cx);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .id("drawer-body")
                    .test_support()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_3()
                    .pt_1()
                    .pb_3()
                    .child(body),
            )
            .into_any_element()
    }
}

mod ai;
mod extensions;
mod markdown;
mod puml;
mod relations;
mod thumbs;
mod validation;
