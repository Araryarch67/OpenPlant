//! Region rendering: menu bar, sidebar, canvas, tool strip, navigator,
//! property editor, bottom drawer, status bar and overlays. `shots/` is the
//! reference look; keep regions pixel-close to it.

mod actions;
mod canvas_view;
mod drawer;
pub(crate) mod icons;
mod menubar;
mod navigator;
mod overlays;
mod properties;
mod sidebar;
mod statusbar;
mod styles;
mod toolstrip;
mod widgets;

pub(crate) use properties::*;
pub(crate) use widgets::*;

use crate::app::*;
use crate::commands::*;
use crate::theme::*;
use gpui_kit::*;

impl Render for OpenPlant {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.keep_fitted(window);
        self.ensure_prop_inputs(window, cx);
        self.sync_puml_from_canvas(window, cx);
        if self.wb.bottom_tab.as_deref() == Some("nav") {
            self.sync_md(window, cx);
        }
        let menubar = self.menubar(cx);
        let sidebar = self.wb.show_sidebar.then(|| self.sidebar(cx));
        let canvas_view = self.canvas_view(cx);
        let strip = self.toolstrip(cx);
        let navigator = self.wb.show_navigator.then(|| self.navigator(window, cx));
        let drawer = self
            .wb
            .bottom_tab
            .is_some()
            .then(|| self.drawer(window, cx));
        let status = self.statusbar(cx);
        let backdrop = self.any_popup().then(|| {
            div()
                .id("backdrop")
                .test_support()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .on_any_mouse_down(cx.listener(|this, _, _, cx| {
                    this.close_popups();
                    cx.notify();
                }))
        });
        let ctx_menu = self
            .ctx_menu
            .clone()
            .map(|m| self.context_menu(&m, window, cx));
        let toasts = self.toasts(cx);
        let palette = self
            .palette
            .is_some()
            .then(|| self.palette_view(window, cx));
        let shortcuts = self.shortcuts_open.then(|| self.shortcuts_view(window, cx));
        let picker = self.picker_for.map(|k| self.picker_view(k, window, cx));

        let root = div()
            .id("root")
            .test_support()
            .key_context(ROOT)
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(c(DEEP))
            .text_color(c(FG))
            .font_family(UI_FONT)
            .text_size(px(13.))
            .line_height(relative(1.45));
        self.bind_actions(root, cx)
            .child(menubar)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .children(sidebar)
                    .child(canvas_view)
                    .child(strip)
                    .children(navigator),
            )
            .children(drawer)
            .child(status)
            .children(backdrop)
            .children(ctx_menu)
            .child(toasts)
            .children(palette)
            .children(shortcuts)
            .children(picker)
    }
}
