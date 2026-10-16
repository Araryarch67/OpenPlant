//! Menu bar and dropdown menus.

use crate::app::*;
use crate::commands::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn menubar(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut bar = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .px_2()
            .py_1()
            .bg(c(PANEL))
            .border_b_1()
            .border_color(c(LINE));
        for (i, m) in MENUS.iter().enumerate() {
            let open = self.open_menu == Some(i);
            let btn = div()
                .id(("menu", i))
                .test_support()
                .px_2()
                .py(px(3.))
                .border_1()
                .border_color(transparent_black())
                .line_height(px(16.))
                .rounded(px(6.))
                .text_xs()
                .cursor_pointer()
                .hover(|s| s.bg(c(HOVER)))
                .when(open, |d| d.bg(c(HOVER)))
                .child(m.label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.ctx_menu = None;
                    this.open_menu = if this.open_menu == Some(i) {
                        None
                    } else {
                        Some(i)
                    };
                    cx.notify();
                }));
            let mut wrap = div().relative().child(btn);
            if open {
                wrap = wrap.child(
                    div().absolute().top(px(26.)).left_0().child(
                        deferred(anchored().snap_to_window().child(self.menu_panel(i, cx)))
                            .with_priority(2),
                    ),
                );
            }
            bar = bar.child(wrap);
        }
        bar.child(div().flex_1())
            .child(
                div()
                    .id("palette-btn")
                    .test_support()
                    .px_2()
                    .py(px(3.))
                    .border_1()
                    .border_color(transparent_black())
                    .line_height(px(16.))
                    .rounded(px(6.))
                    .text_xs()
                    .cursor_pointer()
                    .hover(|s| s.bg(c(HOVER)))
                    .child("Palette")
                    .on_click(cx.listener(|this, _, w, cx| this.run("view.palette", w, cx))),
            )
            .into_any_element()
    }

    pub(crate) fn menu_panel(&self, i: usize, cx: &mut Context<Self>) -> Div {
        let mut panel = popup_panel().min_w(px(220.)).p(px(6.)).flex().flex_col();
        for (j, item) in MENUS[i].items.iter().enumerate() {
            let Some(cmd_id) = item.cmd else {
                panel = panel.child(sep());
                continue;
            };
            let off = match cmd_id {
                "edit.undo" => !self.wb.can_undo(),
                "edit.redo" => !self.wb.can_redo(),
                _ => false,
            };
            let row = div()
                .id(eid(format!("mi-{i}-{j}")))
                .test_support()
                .flex()
                .items_center()
                .gap_4()
                .px_2()
                .py(px(3.))
                .rounded(px(6.))
                .text_xs()
                .cursor_pointer()
                .hover(|s| s.bg(c(HOVER)))
                .when(off, |d| d.text_color(c(MUTED)))
                .child(div().flex_1().child(item.label))
                .children(
                    item.kbd
                        .map(|k| div().text_size(px(11.)).text_color(c(MUTED)).child(k)),
                )
                .on_click(cx.listener(move |this, _, w, cx| this.run(cmd_id, w, cx)));
            panel = panel.child(row);
        }
        panel
    }
}
