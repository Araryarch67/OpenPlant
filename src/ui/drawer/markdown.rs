//! Markdown documentation tab.

use crate::app::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::text::TextView;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn markdown_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        let modes = segmented(
            "md-mode",
            &[("Edit", !self.md_preview), ("Preview", self.md_preview)],
        );
        let mut seg = div().flex().mb_2();
        for (i, b) in modes.into_iter().enumerate() {
            seg = seg.child(b.on_click(cx.listener(move |this, _, _, cx| {
                this.md_preview = i == 1;
                cx.notify();
            })));
        }
        let body = if self.md_preview {
            if self.md_doc.trim().is_empty() {
                div()
                    .text_color(c(MUTED))
                    .italic()
                    .child("Nothing to preview.")
                    .into_any_element()
            } else {
                TextView::markdown("md-preview", self.md_doc.clone()).into_any_element()
            }
        } else {
            div()
                .font_family(MONO_FONT)
                .text_size(px(16.))
                .child(
                    Textarea::new(&self.md_input)
                        .h(px(120.))
                        .bg(c(INPUTBG))
                        .border_color(c(LINE)),
                )
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .child(seg)
            .child(body)
            .into_any_element()
    }
}
