//! Drawing primitives: paths, rects, ellipses, text lines.

use gpui_kit::*;

pub(crate) fn stroke_path(
    window: &mut Window,
    pts: &[Point<Pixels>],
    w: f32,
    color: Hsla,
    dash: Option<(f32, f32)>,
    close: bool,
) {
    if pts.len() < 2 {
        return;
    }
    let mut b = PathBuilder::stroke(px(w));
    if let Some((on, off)) = dash {
        b = b.dash_array(&[px(on), px(off)]);
    }
    b.move_to(pts[0]);
    for p in &pts[1..] {
        b.line_to(*p);
    }
    if close {
        b.close();
    }
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

pub(crate) fn fill_path(window: &mut Window, pts: &[Point<Pixels>], color: Hsla) {
    if pts.len() < 3 {
        return;
    }
    let mut b = PathBuilder::fill();
    b.move_to(pts[0]);
    for p in &pts[1..] {
        b.line_to(*p);
    }
    b.close();
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

pub(crate) fn rect_shape(
    window: &mut Window,
    b: Bounds<Pixels>,
    fill: Hsla,
    stroke: Hsla,
    sw: f32,
    dash: Option<(f32, f32)>,
) {
    window.paint_quad(fill_quad(b, fill));
    match dash {
        None => window.paint_quad(quad(
            b,
            px(0.),
            transparent_black(),
            px(sw),
            stroke,
            BorderStyle::Solid,
        )),
        Some(_) => {
            let (o, s) = (b.origin, b.size);
            let pts = [
                o,
                point(o.x + s.width, o.y),
                point(o.x + s.width, o.y + s.height),
                point(o.x, o.y + s.height),
            ];
            stroke_path(window, &pts, sw, stroke, dash, true);
        }
    }
}

pub(crate) fn fill_quad(b: Bounds<Pixels>, color: Hsla) -> PaintQuad {
    fill(b, color)
}

/// Ellipse as two arcs; `stroke: None` fills it.
pub fn ellipse(
    window: &mut Window,
    center: Point<Pixels>,
    rx: f32,
    ry: f32,
    color: impl Into<Background>,
    stroke: Option<Pixels>,
) {
    let mut b = match stroke {
        Some(w) => PathBuilder::stroke(w),
        None => PathBuilder::fill(),
    };
    let r = point(px(rx), px(ry));
    b.move_to(point(center.x + px(rx), center.y));
    b.arc_to(r, px(0.), false, true, point(center.x - px(rx), center.y));
    b.arc_to(r, px(0.), false, true, point(center.x + px(rx), center.y));
    b.close();
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TAlign {
    Left,
    Center,
    Right,
}

/// Paint one line of text. `x` is the anchor (left/center/right edge),
/// `top` the top of the line box.
#[allow(clippy::too_many_arguments)]
pub fn text_line(
    window: &mut Window,
    cx: &mut App,
    s: &str,
    x: f32,
    top: f32,
    size_px: f32,
    color: Hsla,
    family: &str,
    bold: bool,
    align: TAlign,
) {
    if s.is_empty() || size_px < 3.0 {
        return;
    }
    let mut f = font(SharedString::from(family.to_string()));
    if bold {
        f.weight = FontWeight::BOLD;
    }
    let run = TextRun {
        len: s.len(),
        font: f,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window.text_system().shape_line(
        SharedString::from(s.to_string()),
        px(size_px),
        &[run],
        None,
    );
    let w = f32::from(line.width);
    let left = match align {
        TAlign::Left => x,
        TAlign::Center => x - w / 2.0,
        TAlign::Right => x - w,
    };
    let _ = line.paint(
        point(px(left), px(top)),
        px(size_px * 1.2),
        TextAlign::Left,
        None,
        window,
        cx,
    );
}
