//! Shape painting (UML silhouettes per kind and tool).

use super::*;
use crate::model::*;
use crate::theme::{BLUE, c, canvas_font};
use gpui_kit::*;

pub(crate) struct Look {
    fill: Hsla,
    stroke: Hsla,
    text: Hsla,
    family: &'static str,
    fs: f32,
    sw: f32,
    dash: Option<(f32, f32)>,
    align: TAlign,
}

pub(crate) fn look(s: &Shape, xf: &Xf, selected: bool) -> Look {
    let st = s.style.clone().unwrap_or_default();
    let default_fill = if s.kind() == ShapeKind::Note {
        "#fefce8"
    } else {
        DEFAULT_FILL
    };
    let k = xf.k;
    Look {
        fill: hex(st.fill.as_deref().unwrap_or(default_fill), 0xffffff),
        stroke: if selected {
            c(BLUE)
        } else {
            hex(st.stroke.as_deref().unwrap_or(DEFAULT_STROKE), 0x777777)
        },
        text: hex(st.text.as_deref().unwrap_or(DEFAULT_TEXT), 0x111111),
        family: canvas_font(st.font.as_deref().unwrap_or(DEFAULT_FONT)),
        fs: st.font_size.unwrap_or(DEFAULT_FONT_SIZE) * k,
        sw: (if selected { 2.0 } else { 1.5 }) * k.max(0.5),
        dash: match st.dash {
            Some(Dash::Dashed) => Some((6.0 * k, 4.0 * k)),
            Some(Dash::Dotted) => Some((2.0 * k, 3.0 * k)),
            _ => None,
        },
        align: match st.align {
            Some(Align::Left) => TAlign::Left,
            Some(Align::Right) => TAlign::Right,
            _ => TAlign::Center,
        },
    }
}

/// Text helper in shape-local world units; `baseline` mimics the SVG layout.
pub(crate) fn label_lines(
    window: &mut Window,
    cx: &mut App,
    s: &Shape,
    xf: &Xf,
    lk: &Look,
    x: f32,
    baseline0: f32,
    step: f32,
    align: TAlign,
    fs_scale: f32,
) {
    let fs = lk.fs * fs_scale;
    for (i, ln) in s.label.split('\n').enumerate() {
        let base = baseline0 + i as f32 * step;
        // SVG y is the baseline; convert to the top of the line box.
        let top = xf.sy(s.y + base) - fs * 0.95;
        text_line(
            window,
            cx,
            ln,
            xf.sx(s.x + x),
            top,
            fs,
            lk.text,
            lk.family,
            false,
            align,
        );
    }
}

pub fn paint_shape(
    s: &Shape,
    xf: &Xf,
    selected: bool,
    editing: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let lk = look(s, xf, selected);
    let k = xf.k;
    let (w, h) = (s.w, s.h);
    let n = s.label.split('\n').count() as f32;
    let anchor_x = |lk: &Look, pad: f32| match lk.align {
        TAlign::Left => pad,
        TAlign::Right => w - pad,
        TAlign::Center => w / 2.0,
    };
    let lp = |x: f32, y: f32| xf.p(s.x + x, s.y + y);
    let show_text = !editing;
    match s.kind() {
        ShapeKind::Actor => {
            let cx_ = w / 2.0;
            ellipse(window, lp(cx_, 26.0), 9.0 * k, 9.0 * k, lk.fill, None);
            ellipse(
                window,
                lp(cx_, 26.0),
                9.0 * k,
                9.0 * k,
                lk.stroke,
                Some(px(lk.sw)),
            );
            for (a, b) in [
                ((cx_, 35.0), (cx_, 58.0)),
                ((cx_ - 14.0, 43.0), (cx_ + 14.0, 43.0)),
                ((cx_, 58.0), (cx_ - 11.0, 78.0)),
                ((cx_, 58.0), (cx_ + 11.0, 78.0)),
            ] {
                stroke_path(
                    window,
                    &[lp(a.0, a.1), lp(b.0, b.1)],
                    lk.sw,
                    lk.stroke,
                    None,
                    false,
                );
            }
            if show_text {
                label_lines(window, cx, s, xf, &lk, cx_, 92.0, 15.0, TAlign::Center, 1.0);
            }
        }
        ShapeKind::Database => {
            let cx_ = w / 2.0;
            let rx = w / 2.0 - 10.0;
            let body = h - 42.0;
            let ry = 10.0;
            // Body: sides + bottom arc, filled.
            let mut pts = vec![lp(cx_ - rx, 18.0), lp(cx_ - rx, 18.0 + body)];
            for i in 0..=16 {
                let t = std::f32::consts::PI * (i as f32 / 16.0);
                pts.push(lp(cx_ - rx * t.cos(), 18.0 + body + ry * t.sin()));
            }
            pts.push(lp(cx_ + rx, 18.0));
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, false);
            ellipse(window, lp(cx_, 18.0), rx * k, ry * k, lk.fill, None);
            ellipse(
                window,
                lp(cx_, 18.0),
                rx * k,
                ry * k,
                lk.stroke,
                Some(px(lk.sw)),
            );
            if show_text {
                label_lines(window, cx, s, xf, &lk, cx_, 48.0, 15.0, TAlign::Center, 1.0);
            }
        }
        ShapeKind::Package => {
            let tab = xf.rect(s.x + 8.0, s.y + 4.0, 56.0, 16.0);
            rect_shape(window, tab, lk.fill, lk.stroke, lk.sw, None);
            let body = xf.rect(s.x + 8.0, s.y + 14.0, w - 16.0, h - 22.0);
            rect_shape(window, body, lk.fill, lk.stroke, lk.sw, lk.dash);
            if show_text {
                label_lines(
                    window,
                    cx,
                    s,
                    xf,
                    &lk,
                    anchor_x(&lk, 18.0),
                    44.0,
                    15.0,
                    lk.align,
                    1.0,
                );
            }
        }
        ShapeKind::Note => {
            let fold = 14.0;
            let pts = [
                lp(0.0, 0.0),
                lp(w - fold, 0.0),
                lp(w, fold),
                lp(w, h),
                lp(0.0, h),
            ];
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, true);
            stroke_path(
                window,
                &[lp(w - fold, 0.0), lp(w - fold, fold), lp(w, fold)],
                k.max(0.5),
                lk.stroke,
                None,
                false,
            );
            if show_text {
                label_lines(
                    window,
                    cx,
                    s,
                    xf,
                    &lk,
                    10.0,
                    22.0,
                    15.0,
                    TAlign::Left,
                    (lk.fs - k) / lk.fs.max(0.01),
                );
            }
        }
        ShapeKind::Component => {
            rect_shape(
                window,
                xf.rect(s.x, s.y, w, h),
                lk.fill,
                lk.stroke,
                lk.sw,
                lk.dash,
            );
            for y in [14.0, 28.0] {
                rect_shape(
                    window,
                    xf.rect(s.x + 6.0, s.y + y, 12.0, 8.0),
                    lk.fill,
                    lk.stroke,
                    1.2 * k.max(0.5),
                    None,
                );
            }
            if show_text {
                let x = if lk.align == TAlign::Center {
                    w / 2.0 + 6.0
                } else {
                    anchor_x(&lk, 24.0)
                };
                label_lines(window, cx, s, xf, &lk, x, 22.0, 15.0, lk.align, 1.0);
            }
        }
        ShapeKind::Box => {
            paint_box_variant(s, xf, &lk, window);
            if show_text {
                let base = h / 2.0 - ((n - 1.0) * 15.0) / 2.0 + 5.0;
                label_lines(
                    window,
                    cx,
                    s,
                    xf,
                    &lk,
                    anchor_x(&lk, 10.0),
                    base,
                    15.0,
                    lk.align,
                    1.0,
                );
            }
        }
        ShapeKind::Usecase => {
            let center = lp(w / 2.0, h / 2.0);
            ellipse(
                window,
                center,
                (w / 2.0 - 2.0) * k,
                (h / 2.0 - 2.0) * k,
                lk.fill,
                None,
            );
            ellipse(
                window,
                center,
                (w / 2.0 - 2.0) * k,
                (h / 2.0 - 2.0) * k,
                lk.stroke,
                Some(px(lk.sw)),
            );
            if show_text {
                let base = h / 2.0 - ((n - 1.0) * 15.0) / 2.0 + 5.0;
                label_lines(
                    window,
                    cx,
                    s,
                    xf,
                    &lk,
                    w / 2.0,
                    base,
                    15.0,
                    TAlign::Center,
                    1.0,
                );
            }
        }
        ShapeKind::Class | ShapeKind::Interface => {
            rect_shape(
                window,
                xf.rect(s.x, s.y, w, h),
                lk.fill,
                lk.stroke,
                lk.sw,
                lk.dash,
            );
            let line = |y: f32, window: &mut Window| {
                window.paint_quad(fill(
                    Bounds::new(lp(0.0, y), size(px(w * k), px(k.max(0.6)))),
                    lk.stroke,
                ));
            };
            line(30.0, window);
            line(h - 28.0, window);
            if show_text {
                label_lines(
                    window,
                    cx,
                    s,
                    xf,
                    &lk,
                    anchor_x(&lk, 8.0),
                    20.0,
                    15.0,
                    lk.align,
                    1.0,
                );
            }
        }
    }
}

/// Rectangle-family shapes keep their PlantUML silhouette by tool.
pub(crate) fn paint_box_variant(s: &Shape, xf: &Xf, lk: &Look, window: &mut Window) {
    let (w, h, k) = (s.w, s.h, xf.k);
    let lp = |x: f32, y: f32| xf.p(s.x + x, s.y + y);
    match s.tool.as_deref() {
        Some("Queue") => {
            let r = (h / 2.0).min(14.0);
            let mut pts = vec![];
            for i in 0..=12 {
                let t = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (i as f32 / 12.0);
                pts.push(lp(r + r * 0.6 * t.cos(), h / 2.0 + (h / 2.0) * t.sin()));
            }
            for i in 0..=12 {
                let t = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * (i as f32 / 12.0);
                pts.push(lp(w - r + r * 0.6 * t.cos(), h / 2.0 + (h / 2.0) * t.sin()));
            }
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, true);
            ellipse(
                window,
                lp(w - r, h / 2.0),
                r * 0.6 * k,
                h / 2.0 * k,
                lk.stroke,
                Some(px(lk.sw)),
            );
        }
        Some("Node") => {
            let d = 10.0;
            rect_shape(
                window,
                xf.rect(s.x, s.y + d, w - d, h - d),
                lk.fill,
                lk.stroke,
                lk.sw,
                lk.dash,
            );
            let top = [lp(0.0, d), lp(d, 0.0), lp(w, 0.0), lp(w - d, d)];
            fill_path(window, &top, lk.fill);
            stroke_path(window, &top, lk.sw, lk.stroke, None, true);
            let side = [lp(w - d, d), lp(w, 0.0), lp(w, h - d), lp(w - d, h)];
            fill_path(window, &side, lk.fill);
            stroke_path(window, &side, lk.sw, lk.stroke, None, true);
        }
        Some("Cloud") => {
            let mut pts = vec![];
            for i in 0..=48 {
                let t = std::f32::consts::TAU * (i as f32 / 48.0);
                let bump = 1.0 + 0.07 * (t * 7.0).cos().abs();
                pts.push(lp(
                    w / 2.0 + (w / 2.0 - 2.0) * bump.min(1.06) * t.cos() / 1.06,
                    h / 2.0 + (h / 2.0 - 2.0) * bump.min(1.06) * t.sin() / 1.06,
                ));
            }
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, true);
        }
        Some("Artifact") | Some("File") => {
            let f = 12.0;
            let pts = [lp(0.0, 0.0), lp(w - f, 0.0), lp(w, f), lp(w, h), lp(0.0, h)];
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, true);
            stroke_path(
                window,
                &[lp(w - f, 0.0), lp(w - f, f), lp(w, f)],
                k.max(0.5),
                lk.stroke,
                None,
                false,
            );
        }
        Some("Hexagon") => {
            let d = (h / 2.0).min(18.0);
            let pts = [
                lp(d, 0.0),
                lp(w - d, 0.0),
                lp(w, h / 2.0),
                lp(w - d, h),
                lp(d, h),
                lp(0.0, h / 2.0),
            ];
            fill_path(window, &pts, lk.fill);
            stroke_path(window, &pts, lk.sw, lk.stroke, lk.dash, true);
        }
        Some("Card") | Some("Storage") => {
            let b = xf.rect(s.x, s.y, w, h);
            window.paint_quad(quad(
                b,
                px(10.0 * k),
                lk.fill,
                px(lk.sw),
                lk.stroke,
                BorderStyle::Solid,
            ));
        }
        _ => rect_shape(
            window,
            xf.rect(s.x, s.y, w, h),
            lk.fill,
            lk.stroke,
            lk.sw,
            lk.dash,
        ),
    }
}
