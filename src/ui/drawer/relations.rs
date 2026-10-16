//! Relationships table tab.

use crate::app::*;
use crate::theme::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn relations_tab(&self) -> AnyElement {
        let doc = self.wb.doc();
        if doc.relations.is_empty() {
            return div()
                .text_color(c(MUTED))
                .italic()
                .child("No relationships in this diagram.")
                .into_any_element();
        }
        let name = |id: &str| {
            doc.shape(id)
                .map(|s| s.label.replace('\n', " "))
                .unwrap_or(id.to_string())
        };
        let cell = |t: String, head: bool| {
            div()
                .flex_1()
                .px(px(10.))
                .py(px(5.))
                .border_b_1()
                .border_color(c(LINE))
                .truncate()
                .when(head, |d| {
                    d.text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(c(MUTED))
                })
                .child(t)
        };
        div()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .children(["From", "Type", "To", "Label"].map(|h| cell(h.into(), true))),
            )
            .children(doc.relations.iter().map(|r| {
                div()
                    .flex()
                    .child(cell(name(&r.from), false))
                    .child(cell(r.kind.as_str().into(), false))
                    .child(cell(name(&r.to), false))
                    .child(cell(r.label.clone().unwrap_or("-".into()), false))
            }))
            .into_any_element()
    }
}
