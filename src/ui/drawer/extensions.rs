//! Extensions tab.

use crate::app::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

pub(crate) const EXTENSIONS: &[(&str, &str, &str)] = &[
    ("markdown", "Markdown Documentation", "1.2.0"),
    ("puml", "PlantUML Script", "1.0.0"),
    ("ai", "AI Diagram Generator", "0.9.0"),
    ("thumbs", "Diagram Thumbnails", "0.9.1"),
    ("validation", "Model Validation", "1.1.0"),
];

impl OpenPlant {
    pub(crate) fn extensions_tab(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .children(EXTENSIONS.iter().map(|(id, name, ver)| {
                let id = *id;
                let on = self.wb.ext_on(id);
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .border_b_1()
                    .border_color(c(LINE))
                    .px_1()
                    .py(px(6.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(*name))
                    .child(div().text_color(c(MUTED)).child(format!("v{ver}")))
                    .child(div().flex_1())
                    .child(
                        div()
                            .id(eid(format!("ext-{id}")))
                            .test_support()
                            .px_3()
                            .py(px(3.))
                            .rounded(px(12.))
                            .border_1()
                            .cursor_pointer()
                            .when(on, |d| d.border_color(c(GREEN)).text_color(c(GREEN)))
                            .when(!on, |d| {
                                d.border_color(c(LINE)).bg(c(RAISED)).text_color(c(MUTED))
                            })
                            .child(if on { "On" } else { "Off" })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let v = this.wb.ext_on(id);
                                this.wb.extensions.insert(id.into(), !v);
                                cx.notify();
                            })),
                    )
            }))
            .into_any_element()
    }
}
