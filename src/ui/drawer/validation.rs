//! Model validation tab.

use crate::app::*;
use crate::theme::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn validation_tab(&self) -> AnyElement {
        let doc = self.wb.doc();
        let mut rows: Vec<(bool, String)> = vec![];
        let mut seen: Vec<(String, usize)> = vec![];
        for s in &doc.shapes {
            if s.label.trim().is_empty() {
                rows.push((true, format!("Shape {} has no name.", s.id)));
            }
            match seen.iter_mut().find(|(l, _)| *l == s.label) {
                Some(e) => e.1 += 1,
                None => seen.push((s.label.clone(), 1)),
            }
        }
        for (label, n) in seen {
            if n > 1 && !label.trim().is_empty() {
                rows.push((
                    false,
                    format!("\"{}\" is used {n} times.", label.replace('\n', " ")),
                ));
            }
        }
        for r in &doc.relations {
            if doc.shape(&r.from).is_none() || doc.shape(&r.to).is_none() {
                rows.push((
                    true,
                    format!("Relation {} points to a missing shape.", r.id),
                ));
            }
        }
        if rows.is_empty() {
            return div()
                .text_color(c(MUTED))
                .italic()
                .child("No issues found. Model is valid.")
                .into_any_element();
        }
        div()
            .flex()
            .flex_col()
            .children(rows.into_iter().map(|(err, msg)| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_1()
                    .py(px(5.))
                    .child(
                        div()
                            .size(px(18.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(c(DEEP))
                            .bg(if err { c(RED) } else { c(YELLOW) })
                            .child(if err { "E" } else { "W" }),
                    )
                    .child(msg)
            }))
            .into_any_element()
    }
}
