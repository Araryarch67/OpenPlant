//! Whole-canvas paint: grid, z-ordered shapes and edges, selection overlay.

use super::*;
use crate::model::*;
use crate::theme::{BLUE, PAPER, PAPERGRID, c, ca};
use gpui_kit::*;

#[derive(Clone, Default)]
pub struct Overlay {
    pub selected: Vec<String>,
    pub selected_rel: Vec<String>,
    pub hover: Option<String>,
    pub pending_from: Option<String>,
    pub rubber: Option<(f32, f32, f32, f32)>,
    /// (from shape id, window x, window y, target id)
    pub connect: Option<(String, f32, f32, Option<String>)>,
    pub show_arrows: bool,
    pub grid: bool,
    pub minimap: bool,
    pub editing: Option<String>,
}

pub fn paint_canvas(
    doc: &DiagramDoc,
    xf: &Xf,
    bounds: Bounds<Pixels>,
    ov: &Overlay,
    window: &mut Window,
    cx: &mut App,
) {
    window.paint_quad(fill(bounds, c(PAPER)));
    if ov.grid {
        paint_grid(xf, bounds, window);
    }
    let containers: Vec<&Shape> = doc
        .shapes
        .iter()
        .filter(|s| is_container(doc, &s.id))
        .collect();
    // Containers first (outermost first), then edges, then leaves on top.
    let mut ordered = containers.clone();
    let depth = |s: &Shape| {
        let mut d = 0;
        let mut cur = s.parent_id.clone();
        while let Some(p) = cur {
            d += 1;
            cur = doc.shape(&p).and_then(|x| x.parent_id.clone());
        }
        d
    };
    ordered.sort_by_key(|s| depth(s));
    for s in &ordered {
        paint_shape(
            s,
            xf,
            ov.selected.contains(&s.id),
            ov.editing.as_deref() == Some(&s.id),
            window,
            cx,
        );
    }
    for r in &doc.relations {
        let (Some(a), Some(b)) = (doc.shape(&r.from), doc.shape(&r.to)) else {
            continue;
        };
        paint_relation(
            r,
            a,
            b,
            xf,
            ov.selected_rel.contains(&r.id),
            ov.editing.as_deref() == Some(&r.id),
            window,
            cx,
        );
    }
    for s in &doc.shapes {
        if let Some(t) = s.note_of.as_ref().and_then(|t| doc.shape(t)) {
            let pts: Vec<Point<Pixels>> = straight_route(s, t)
                .iter()
                .map(|(x, y)| xf.p(*x, *y))
                .collect();
            stroke_path(
                window,
                &pts,
                1.2 * xf.k.max(0.5),
                c(0x777777),
                Some((5.0 * xf.k, 4.0 * xf.k)),
                false,
            );
        }
    }
    for s in doc.shapes.iter().filter(|s| !is_container(doc, &s.id)) {
        let sel = ov.selected.contains(&s.id) || ov.pending_from.as_deref() == Some(&s.id);
        paint_shape(s, xf, sel, ov.editing.as_deref() == Some(&s.id), window, cx);
    }
    if let (Some(title), Some((x0, y0, x1, _))) = (&doc.title, bbox(doc)) {
        let lines: Vec<&str> = title.split('\n').collect();
        let fs = 15.0 * xf.k;
        let top = xf.sy(y0) - (lines.len() as f32) * fs * 1.25 - 14.0 * xf.k;
        for (i, ln) in lines.iter().enumerate() {
            text_line(
                window,
                cx,
                ln,
                xf.sx((x0 + x1) / 2.0),
                top + i as f32 * fs * 1.25,
                fs,
                c(0x111111),
                "Liberation Sans",
                true,
                TAlign::Center,
            );
        }
    }
    paint_selection(doc, xf, ov, window);
    if ov.minimap {
        paint_minimap(doc, xf, bounds, window);
    }
}

pub(crate) fn paint_grid(xf: &Xf, bounds: Bounds<Pixels>, window: &mut Window) {
    let step = GRID * xf.k;
    if step < 5.0 {
        return;
    }
    let (bx, by) = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let col = c(PAPERGRID);
    let start_x = bx + (xf.pan_x).rem_euclid(step);
    let start_y = by + (xf.pan_y).rem_euclid(step);
    let mut x = start_x;
    while x < bx + bw {
        window.paint_quad(fill(
            Bounds::new(point(px(x), px(by)), size(px(1.), px(bh))),
            col,
        ));
        x += step;
    }
    let mut y = start_y;
    while y < by + bh {
        window.paint_quad(fill(
            Bounds::new(point(px(bx), px(y)), size(px(bw), px(1.))),
            col,
        ));
        y += step;
    }
}

pub(crate) fn paint_selection(doc: &DiagramDoc, xf: &Xf, ov: &Overlay, window: &mut Window) {
    let blue = c(BLUE);
    // Resize handles on a single selection (draw.io: 8 squares).
    if ov.selected.len() == 1
        && ov.editing.is_none()
        && let Some(s) = doc.shape(&ov.selected[0])
    {
        for h in HANDLES {
            let (hx, hy) = handle_pos(s, h);
            let b = Bounds::new(
                point(px(xf.sx(hx) - 4.0), px(xf.sy(hy) - 4.0)),
                size(px(8.), px(8.)),
            );
            window.paint_quad(quad(
                b,
                px(1.),
                c(0xffffff),
                px(1.),
                blue,
                BorderStyle::Solid,
            ));
        }
    } else {
        for s in doc.shapes.iter().filter(|s| ov.selected.contains(&s.id)) {
            let b = xf.rect(
                s.x - 3.0 / xf.k,
                s.y - 3.0 / xf.k,
                s.w + 6.0 / xf.k,
                s.h + 6.0 / xf.k,
            );
            let (o, sz) = (b.origin, b.size);
            let pts = [
                o,
                point(o.x + sz.width, o.y),
                point(o.x + sz.width, o.y + sz.height),
                point(o.x, o.y + sz.height),
            ];
            stroke_path(window, &pts, 1.0, blue, Some((4.0, 3.0)), true);
        }
    }
    for r in doc
        .relations
        .iter()
        .filter(|r| ov.selected_rel.contains(&r.id))
    {
        let (Some(a), Some(b)) = (doc.shape(&r.from), doc.shape(&r.to)) else {
            continue;
        };
        let route = ortho_route(a, b);
        for (x, y) in [route.pts[0], *route.pts.last().unwrap()] {
            let bb = Bounds::new(
                point(px(xf.sx(x) - 4.0), px(xf.sy(y) - 4.0)),
                size(px(8.), px(8.)),
            );
            window.paint_quad(quad(
                bb,
                px(4.),
                blue,
                px(1.),
                c(0xffffff),
                BorderStyle::Solid,
            ));
        }
    }
    // Hover connection arrows.
    if ov.show_arrows
        && let Some(s) = ov.hover.as_ref().and_then(|h| doc.shape(h))
    {
        for (x, y, dx, dy) in connect_arrows(s, xf) {
            let (nx, ny) = (-dy, dx);
            let tip = point(px(x + dx * 7.0), px(y + dy * 7.0));
            let l = point(px(x - dx * 5.0 + nx * 6.0), px(y - dy * 5.0 + ny * 6.0));
            let r = point(px(x - dx * 5.0 - nx * 6.0), px(y - dy * 5.0 - ny * 6.0));
            fill_path(window, &[tip, l, r], ca(BLUE, 0.75));
        }
    }
    if let Some((x0, y0, x1, y1)) = ov.rubber {
        let b = Bounds::from_corners(
            point(px(x0.min(x1)), px(y0.min(y1))),
            point(px(x0.max(x1)), px(y0.max(y1))),
        );
        window.paint_quad(fill(b, ca(BLUE, 0.12)));
        let (o, sz) = (b.origin, b.size);
        let pts = [
            o,
            point(o.x + sz.width, o.y),
            point(o.x + sz.width, o.y + sz.height),
            point(o.x, o.y + sz.height),
        ];
        stroke_path(window, &pts, 1.0, blue, Some((4.0, 3.0)), true);
    }
    if let Some((from, mx, my, target)) = &ov.connect
        && let Some(a) = doc.shape(from)
    {
        let (ax, ay) = a.center();
        stroke_path(
            window,
            &[xf.p(ax, ay), point(px(*mx), px(*my))],
            1.6,
            blue,
            Some((6.0, 4.0)),
            false,
        );
        if let Some(t) = target.as_ref().and_then(|t| doc.shape(t)) {
            let b = xf.rect(
                t.x - 2.0 / xf.k,
                t.y - 2.0 / xf.k,
                t.w + 4.0 / xf.k,
                t.h + 4.0 / xf.k,
            );
            window.paint_quad(quad(
                b,
                px(2.),
                transparent_black(),
                px(2.),
                blue,
                BorderStyle::Solid,
            ));
        }
    }
}
