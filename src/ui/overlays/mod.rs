//! Overlays: toasts, palette, shortcut sheet, element picker, context menu.

use crate::app::*;
use crate::theme::*;
use crate::ui::widgets::Btn;
use gpui_kit::*;

impl OpenPlant {
    pub(crate) fn modal_backdrop(
        &self,
        id: &'static str,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Btn {
        let top = f32::from(window.viewport_size().height) * 0.12;
        div()
            .id(id)
            .test_support()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(ca(0x000000, 0.45))
            .occlude()
            .flex()
            .justify_center()
            .items_start()
            .pt(px(top))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, w, cx| {
                    this.palette = None;
                    this.shortcuts_open = false;
                    this.picker_for = None;
                    this.focus_canvas(w, cx);
                    cx.notify();
                }),
            )
    }
}

mod context_menu;
mod palette;
mod picker;
mod shortcuts;
mod toasts;
