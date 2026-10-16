//! Styles section: font, colours, line style, z-order, alignment.

use crate::app::*;
use crate::model::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;

pub(crate) const PRESETS: &[u32] = &[
    0xffffff, 0xf4f1e8, 0xfefce8, 0xfef3c7, 0xdbeafe, 0xdcfce7, 0xfee2e2, 0xf3e8ff, 0xe5e7eb,
    0x94a3b8, 0x777777, 0x475569, 0x334155, 0x111111, 0xe57474, 0x8ccf7e, 0xe5c76b, 0x67b0e8,
    0xc47fd5, 0x6cbfbf, 0x2563eb, 0xd97706, 0xca8a04, 0x16a34a,
];

impl OpenPlant {
    pub(crate) fn styles_section(&self, s: &Shape, cx: &mut Context<Self>) -> AnyElement {
        let st = s.style.clone().unwrap_or_default();
        let inherited = s.style.is_none();
        let n_sel = self.wb.selected_shapes.len();
        let swatch = |which: &'static str,
                      value: &str,
                      title: &'static str,
                      this: &Self,
                      cx: &mut Context<Self>| {
            let col = parse_color(value).unwrap_or(0xffffff);
            let open = this.color_pop == Some(which);
            let pop = open.then(|| {
                let mut grid = popup_panel()
                    .p(px(8.))
                    .w(px(196.))
                    .flex()
                    .flex_wrap()
                    .gap(px(6.));
                for (i, p) in PRESETS.iter().enumerate() {
                    let p = *p;
                    grid = grid.child(
                        div()
                            .id(eid(format!("preset-{which}-{i}")))
                            .test_support()
                            .size(px(22.))
                            .rounded(px(4.))
                            .border_1()
                            .border_color(if p == col { c(BLUE) } else { c(LINE) })
                            .bg(c(p))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let hexv = color_hex(p);
                                this.wb.set_style(|s| match which {
                                    "fill" => s.fill = Some(hexv.clone()),
                                    "stroke" => s.stroke = Some(hexv.clone()),
                                    _ => s.text = Some(hexv.clone()),
                                });
                                this.color_pop = None;
                                cx.notify();
                            })),
                    );
                }
                div()
                    .absolute()
                    .top(px(28.))
                    .left_0()
                    .child(deferred(anchored().snap_to_window().child(grid)).with_priority(2))
            });
            div()
                .relative()
                .child(
                    div()
                        .id(eid(format!("swatch-{which}")))
                        .test_support()
                        .w(px(34.))
                        .h(px(24.))
                        .p(px(3.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(if open { c(BLUE) } else { c(LINE) })
                        .bg(c(INPUTBG))
                        .cursor_pointer()
                        .child(div().size_full().rounded(px(3.)).bg(c(col)))
                        .tooltip(move |w, cx| Tooltip::new(title).build(w, cx))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.color_pop = if this.color_pop == Some(which) {
                                None
                            } else {
                                Some(which)
                            };
                            cx.notify();
                        })),
                )
                .children(pop)
        };
        let fill_sw = swatch(
            "fill",
            st.fill.as_deref().unwrap_or(DEFAULT_FILL),
            "Fill color",
            self,
            cx,
        );
        let line_sw = swatch(
            "stroke",
            st.stroke.as_deref().unwrap_or(DEFAULT_STROKE),
            "Line color",
            self,
            cx,
        );
        let text_sw = swatch(
            "text",
            st.text.as_deref().unwrap_or(DEFAULT_TEXT),
            "Text color",
            self,
            cx,
        );
        let font = st.font.clone().unwrap_or_else(|| DEFAULT_FONT.into());
        let cur_dash = st.dash.unwrap_or_default();
        let dashes = [
            (Dash::Solid, None),
            (Dash::Dashed, Some((6.0, 4.0))),
            (Dash::Dotted, Some((2.0, 3.0))),
        ];
        let mut line_row = div().flex().flex_1();
        for (i, (d, pat)) in dashes.into_iter().enumerate() {
            let on = cur_dash == d;
            line_row = line_row.child(
                div()
                    .id(("dash", i))
                    .test_support()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .px_1()
                    .py(px(5.))
                    .border_1()
                    .border_color(if on { c(BLUE) } else { c(LINE) })
                    .when(i == 0, |x| x.rounded_l(px(6.)))
                    .when(i == 2, |x| x.rounded_r(px(6.)))
                    .bg(c(RAISED))
                    .cursor_pointer()
                    .child(dash_glyph(pat, if on { c(BLUE) } else { c(MUTED) }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.wb.set_style(|s| s.dash = Some(d));
                        cx.notify();
                    })),
            );
        }
        let aligns: [(&str, &str); 10] = [
            ("align.left", "Left"),
            ("align.center", "Center"),
            ("align.right", "Right"),
            ("align.top", "Top"),
            ("align.middle", "Middle"),
            ("align.bottom", "Bottom"),
            ("align.dist-h", "Dist H"),
            ("align.dist-v", "Dist V"),
            ("align.same-w", "Same W"),
            ("align.same-h", "Same H"),
        ];
        let mut grid = div().flex_1().grid().grid_cols(3).gap(px(6.));
        for (cmd_id, label) in aligns {
            let off = n_sel < 2;
            grid = grid.child(
                disabled(button_sm(eid(format!("al-{cmd_id}")), label), off).on_click(cx.listener(
                    move |this, _, w, cx| {
                        if this.wb.selected_shapes.len() >= 2 {
                            this.run(cmd_id, w, cx);
                        }
                    },
                )),
            );
        }
        let font_size = self.prop.font_size.clone();
        let body = div()
            .flex()
            .flex_col()
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(label_cell("Font"))
                    .child(self.select_box("font", font, FONTS, cx))
                    .children(font_size.map(|e| div().w(px(56.)).flex_none().child(field(&e)))),
            )
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(label_cell("Color"))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .gap_2()
                            .child(fill_sw)
                            .child(line_sw)
                            .child(text_sw)
                            .child(
                                div()
                                    .id("inherit")
                                    .test_support()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_color(c(MUTED))
                                    .cursor_pointer()
                                    .child(checkbox("inherit-chk", inherited))
                                    .child("Inherit")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if inherited {
                                            this.wb.set_style(|_| {});
                                        } else {
                                            this.wb.clear_style();
                                        }
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(label_cell("Line Style"))
                    .child(line_row),
            )
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(label_cell("Format"))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .gap(px(6.))
                            .child(button("to-front", "To Front").on_click(
                                cx.listener(|this, _, w, cx| this.run("arrange.front", w, cx)),
                            ))
                            .child(button("to-back", "To Back").on_click(
                                cx.listener(|this, _, w, cx| this.run("arrange.back", w, cx)),
                            )),
                    ),
            )
            .child(
                div()
                    .mb_2()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(label_cell("Alignment"))
                    .child(grid),
            )
            .into_any_element();
        self.section("styles", "Styles", body, cx)
    }
}
