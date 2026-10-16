//! Shared controls styled like the original web components.

use crate::app::*;
use crate::theme::*;
use crate::ui::icons::*;
use gpui_kit::component::input::Input;
use gpui_kit::prelude::*;
use gpui_kit::*;

/// An identified, test-observable element (plain `Stateful<Div>` unless the
/// `test-support` feature of gpui-kit is enabled for UI tests).
pub(crate) type Btn = gpui_kit::base::ObservedElement<Stateful<Div>>;

pub fn eid(s: impl Into<String>) -> ElementId {
    ElementId::Name(s.into().into())
}

pub(crate) const CHECK: &[Op] = &[Op::Path(&[(3.5, 8.2), (6.6, 11.2), (12.6, 4.8)])];

pub(crate) fn header(label: &'static str) -> Div {
    div()
        .px_3()
        .pt_2()
        .pb_1()
        .text_size(px(11.))
        .text_color(c(MUTED))
        .child(label)
}

pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Btn {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .px_3()
        .py(px(6.))
        .rounded(px(6.))
        .border_1()
        .border_color(c(LINE))
        .bg(c(RAISED))
        .text_color(c(FG))
        .cursor_pointer()
        .hover(|s| s.bg(c(HOVER)))
        .child(label.into())
}

pub fn button_sm(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Btn {
    button(id, label).px_2().py(px(3.)).text_xs()
}

pub(crate) fn disabled(el: Btn, off: bool) -> Btn {
    if off {
        el.opacity(0.45).cursor_default()
    } else {
        el
    }
}

pub fn icon_button(id: impl Into<ElementId>) -> Btn {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .px_2()
        .py_1()
        .rounded(px(6.))
        .border_1()
        .border_color(c(LINE))
        .bg(c(RAISED))
        .text_color(c(FG))
        .cursor_pointer()
        .hover(|s| s.bg(c(HOVER)))
}

pub(crate) fn checkbox(id: impl Into<ElementId>, checked: bool) -> Btn {
    let b = div()
        .id(id)
        .test_support()
        .size(px(16.))
        .flex_none()
        .rounded(px(3.))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center();
    if checked {
        b.bg(c(BLUE))
            .child(glyph_el(CHECK, (16.0, 16.0), c(0x10161a), 14.0, 14.0, 2.0))
    } else {
        b.bg(c(0xffffff)).border_1().border_color(c(0x767676))
    }
}

/// Text field matching the web `TextInput` (34px tall, 13px text).
pub(crate) fn field(inp: &Entity<gpui_kit::component::input::InputState>) -> Input {
    let i = Input::new(inp)
        .bg(c(INPUTBG))
        .border_color(c(LINE))
        .text_color(c(FG))
        .text_size(px(13.));
    Styled::h(i, px(34.))
}

pub(crate) fn label_cell(text: &'static str) -> Div {
    div()
        .w(px(110.))
        .flex_none()
        .text_right()
        .text_xs()
        .text_color(c(MUTED))
        .child(text)
}

pub(crate) fn segmented(id: &'static str, opts: &[(&'static str, bool)]) -> Vec<Btn> {
    let n = opts.len();
    opts.iter()
        .enumerate()
        .map(|(i, (label, on))| {
            div()
                .id(eid(format!("{id}-{i}")))
                .test_support()
                .px(px(14.))
                .py_1()
                .border_1()
                .border_color(c(LINE))
                .when(i > 0, |d| d.border_l_0())
                .when(i == 0, |d| d.rounded_l(px(6.)))
                .when(i == n - 1, |d| d.rounded_r(px(6.)))
                .cursor_pointer()
                .bg(if *on { c(HOVER) } else { c(RAISED) })
                .text_color(if *on { c(FG) } else { c(MUTED) })
                .child(*label)
        })
        .collect()
}

pub(crate) fn popup_panel() -> Div {
    div()
        .bg(c(RAISED))
        .border_1()
        .border_color(c(LINE))
        .rounded(px(8.))
        .shadow_lg()
        .occlude()
        .text_color(c(FG))
}

pub(crate) fn sep() -> Div {
    div().h(px(1.)).bg(c(LINE)).mx(px(2.)).my_1()
}

pub(crate) fn dash_glyph(dash: Option<(f32, f32)>, color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |b, _, window, _| {
            let y = b.origin.y + b.size.height / 2.0;
            let mut p = PathBuilder::stroke(px(1.6));
            if let Some((on, off)) = dash {
                p = p.dash_array(&[px(on), px(off)]);
            }
            p.move_to(point(b.origin.x + px(1.), y));
            p.line_to(point(b.origin.x + b.size.width - px(1.), y));
            if let Ok(path) = p.build() {
                window.paint_path(path, color);
            }
        },
    )
    .w(px(28.))
    .h(px(8.))
}

impl OpenPlant {
    pub(crate) fn check_label(
        &self,
        id: &'static str,
        on: bool,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> Btn {
        let _ = cx;
        div()
            .id(id)
            .test_support()
            .flex()
            .items_center()
            .gap(px(6.))
            .text_color(c(MUTED))
            .cursor_pointer()
            .child(checkbox(eid(format!("{id}-box")), on))
            .child(label)
    }
}
