//! Vertical tool strip that opens drawer tabs.

use crate::app::*;
use crate::theme::*;
use crate::ui::icons::*;
use crate::ui::*;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;

pub(crate) const STRIP: &[(&str, &str)] = &[
    ("ext", "Extensions"),
    ("diagrams", "Diagrams"),
    ("model", "Model"),
    ("nav", "Navigate"),
    ("share", "Share"),
    ("puml", "PlantUML Script"),
    ("ai", "AI Generate"),
];

impl OpenPlant {
    pub(crate) fn toolstrip(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.wb.bottom_tab.clone();
        div()
            .flex()
            .flex_col()
            .flex_none()
            .items_center()
            .gap_1()
            .pt_2()
            .w(px(44.))
            .h_full()
            .bg(c(PANEL))
            .border_l_1()
            .border_color(c(LINE))
            .children(STRIP.iter().map(|(id, title)| {
                let (id, title) = (*id, *title);
                icon_button(eid(format!("strip-{id}")))
                    .when(open.as_deref() == Some(id), |d| d.bg(c(HOVER)))
                    .child(glyph_el(
                        strip_glyph(id),
                        (18.0, 18.0),
                        c(FG),
                        18.0,
                        18.0,
                        1.5,
                    ))
                    .tooltip(move |w, cx| Tooltip::new(title).build(w, cx))
                    .on_click(cx.listener(move |this, _, w, cx| {
                        this.wb.strip_tab = id.to_string();
                        if this.wb.bottom_tab.as_deref() == Some(id) {
                            this.wb.bottom_tab = None;
                        } else {
                            this.wb.bottom_tab = Some(id.to_string());
                            if id == "puml" {
                                this.puml_last_set.clear();
                            }
                            if id == "ai" && this.ai.local && this.ai.local_models.is_empty() {
                                this.ai_refresh_models(w, cx);
                            }
                        }
                        cx.notify();
                    }))
            }))
            .into_any_element()
    }
}
