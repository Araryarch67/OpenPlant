//! AI diagram generator tab.

use crate::app::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::component::input::Textarea;
use gpui_kit::prelude::*;
use gpui_kit::*;

pub(crate) const TEXT_TO_DIAGRAM: &[Op] = &[
    Op::Line(1.5, 4.5, 6.5, 4.5),
    Op::Line(1.5, 7.5, 6.5, 7.5),
    Op::Line(1.5, 10.5, 5.0, 10.5),
    Op::Path(&[(8.0, 9.0), (10.5, 9.0), (12.0, 7.5)]),
    Op::Line(10.5, 9.0, 12.0, 10.5),
    Op::Rect(11.0, 4.0, 5.5, 7.0),
    Op::Line(11.0, 6.5, 16.5, 6.5),
];

impl OpenPlant {
    pub(crate) fn ai_tab(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let _ = window;
        let ai = &self.ai;
        let mut provider = div().flex().ml_auto();
        for (i, b) in segmented(
            "ai-prov",
            &[("Local Ollama", ai.local), ("Cloud API", !ai.local)],
        )
        .into_iter()
        .enumerate()
        {
            provider = provider.child(b.on_click(cx.listener(move |this, _, w, cx| {
                this.ai.local = i == 0;
                if this.ai.local && this.ai.local_models.is_empty() {
                    this.ai_refresh_models(w, cx);
                }
                cx.notify();
            })));
        }
        let can_run = !ai.busy
            && (if ai.local {
                !ai.local_model.trim().is_empty()
            } else {
                true
            });
        let settings_row = |label: &'static str, el: AnyElement| {
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .w(px(90.))
                        .flex_none()
                        .text_right()
                        .text_color(c(MUTED))
                        .child(label),
                )
                .child(div().flex_1().min_w_0().child(el))
        };
        let settings = ai.settings_open.then(|| {
            let mut col = div().mt_2().flex().flex_col().gap_2().max_w(px(720.));
            if ai.local {
                let model = if ai.local_models.contains(&ai.local_model) {
                    ai.local_model.clone()
                } else {
                    ai.local_models.first().cloned().unwrap_or_else(|| "(no local models found)".into())
                };
                let models: &'static [&'static str] =
                    Box::leak(ai.local_models.iter().map(|m| &*Box::leak(m.clone().into_boxed_str())).collect::<Vec<&'static str>>().into_boxed_slice());
                col = col
                    .child(settings_row("Endpoint", field(&ai.local_endpoint).into_any_element()))
                    .child(settings_row(
                        "Model",
                        div()
                            .flex()
                            .gap_2()
                            .child(self.select_box("ai-model", model, models, cx))
                            .child(button_sm("ai-refresh", "Refresh").on_click(cx.listener(|this, _, w, cx| this.ai_refresh_models(w, cx))))
                            .into_any_element(),
                    ))
                    .when(!ai.models_error.is_empty(), |d| d.child(div().text_color(c(RED)).child(ai.models_error.clone())))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(c(MUTED))
                            .child("Needs Ollama running locally (ollama serve + a pulled model). No key needed."),
                    );
            } else {
                col = col
                    .child(settings_row("Endpoint", field(&ai.endpoint).into_any_element()))
                    .child(settings_row("API key", field(&ai.key).into_any_element()))
                    .child(settings_row("Model", field(&ai.model).into_any_element()));
            }
            col
        });
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .max_w(px(760.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .size(px(40.))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(10.))
                            .border_1()
                            .border_color(c(LINE))
                            .bg(c(RAISED))
                            .child(glyph_el(
                                TEXT_TO_DIAGRAM,
                                (18.0, 18.0),
                                c(BLUE),
                                22.0,
                                22.0,
                                1.5,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::BOLD)
                                    .child("Describe your domain"),
                            )
                            .child(div().text_color(c(MUTED)).child(
                                "An agentic loop drafts, executes, and refines the diagram.",
                            )),
                    )
                    .child(provider),
            )
            .child(
                div().text_size(px(14.)).child(
                    Textarea::new(&ai.prompt)
                        .h(px(76.))
                        .bg(c(INPUTBG))
                        .border_color(c(LINE)),
                ),
            )
            .child(
                div().flex().flex_wrap().gap_2().children(
                    crate::services::ai::EXAMPLES
                        .iter()
                        .enumerate()
                        .map(|(i, ex)| {
                            let ex = *ex;
                            div()
                                .id(("ai-ex", i))
                                .test_support()
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .border_1()
                                .border_color(c(LINE))
                                .bg(c(RAISED))
                                .text_xs()
                                .text_color(c(MUTED))
                                .cursor_pointer()
                                .hover(|s| s.border_color(c(BLUE)).text_color(c(FG)))
                                .child(ex)
                                .on_click(cx.listener(move |this, _, w, cx| {
                                    this.ai.prompt.update(cx, |s, cx| s.set_value(ex, w, cx));
                                    cx.notify();
                                }))
                        }),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        disabled(
                            div()
                                .id("ai-generate")
                                .test_support()
                                .px(px(22.))
                                .py(px(9.))
                                .rounded(px(6.))
                                .bg(c(BLUE))
                                .text_color(c(0x10161a))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .child(if ai.busy {
                                    "Generating…"
                                } else {
                                    "Generate Diagram"
                                }),
                            !can_run,
                        )
                        .on_click(cx.listener(move |this, _, w, cx| {
                            if can_run {
                                this.ai_generate(w, cx);
                            }
                        })),
                    )
                    .when(!ai.status.is_empty(), |d| {
                        d.child(div().text_color(c(GREEN)).child(ai.status.clone()))
                    })
                    .when(!ai.error.is_empty(), |d| {
                        d.child(div().text_color(c(RED)).child(ai.error.clone()))
                    }),
            )
            .child(
                div()
                    .id("ai-settings")
                    .test_support()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .py_1()
                    .text_color(c(MUTED))
                    .cursor_pointer()
                    .child(chevron(
                        if ai.settings_open { "down" } else { "right" },
                        c(MUTED),
                    ))
                    .child("Provider settings")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ai.settings_open = !this.ai.settings_open;
                        cx.notify();
                    })),
            )
            .children(settings)
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(c(MUTED))
                    .child("Key stays on this machine. Works with OpenAI, DeepSeek, Ollama."),
            )
            .into_any_element()
    }
}
