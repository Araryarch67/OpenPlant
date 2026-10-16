//! Diagram thumbnails tab.

use crate::app::*;
use crate::canvas::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn thumbs_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_wrap()
            .gap_3()
            .children(
                self.wb
                    .diagrams_sorted()
                    .into_iter()
                    .filter_map(|(id, name)| {
                        let doc = self.wb.project.diagrams.get(&id)?.clone();
                        let a = id.clone();
                        Some(
                            div()
                                .id(eid(format!("thumb-{id}")))
                                .test_support()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(6.))
                                .p_2()
                                .rounded(px(8.))
                                .border_1()
                                .border_color(c(LINE))
                                .bg(c(RAISED))
                                .cursor_pointer()
                                .hover(|s| s.border_color(c(BLUE)))
                                .child(
                                    canvas(|_, _, _| {}, move |b, _, w, _| paint_thumb(&doc, b, w))
                                        .w(px(150.))
                                        .h(px(92.)),
                                )
                                .child(name)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.wb.activate_diagram(&a);
                                    this.pending_fit = true;
                                    cx.notify();
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}
