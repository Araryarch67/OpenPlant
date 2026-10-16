//! Overview minimap and diagram thumbnails.

use super::*;
use crate::model::*;
use crate::theme::{BLUE, PAPER, c, ca};
use gpui_kit::*;

/// Overview map: shapes and the current viewport scaled into a corner box.
pub struct MiniMap {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub wx0: f32,
    pub wy0: f32,
}

pub fn minimap_geom(doc: &DiagramDoc, xf: &Xf, bounds: Bounds<Pixels>) -> MiniMap {
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let (vx0, vy0) = xf.world(xf.ox, xf.oy);
    let (vx1, vy1) = xf.world(xf.ox + bw, xf.oy + bh);
    let (mut x0, mut y0, mut x1, mut y1) = (vx0, vy0, vx1, vy1);
    if let Some(b) = bbox(doc) {
        x0 = x0.min(b.0 - 40.0);
        y0 = y0.min(b.1 - 40.0);
        x1 = x1.max(b.2 + 40.0);
        y1 = y1.max(b.3 + 40.0);
    }
    let scale = ((MM_W - 8.0) / (x1 - x0).max(1.0)).min((MM_H - 8.0) / (y1 - y0).max(1.0));
    MiniMap {
        x: f32::from(bounds.origin.x) + bw - MM_W - 16.0,
        y: f32::from(bounds.origin.y) + bh - MM_H - 16.0,
        scale,
        wx0: x0 - ((MM_W - 8.0) / scale - (x1 - x0)) / 2.0,
        wy0: y0 - ((MM_H - 8.0) / scale - (y1 - y0)) / 2.0,
    }
}

impl MiniMap {
    pub fn contains(&self, mx: f32, my: f32) -> bool {
        mx >= self.x && mx <= self.x + MM_W && my >= self.y && my <= self.y + MM_H
    }
    pub fn to_world(&self, mx: f32, my: f32) -> (f32, f32) {
        (
            self.wx0 + (mx - self.x - 4.0) / self.scale,
            self.wy0 + (my - self.y - 4.0) / self.scale,
        )
    }
    fn sx(&self, wx: f32) -> f32 {
        self.x + 4.0 + (wx - self.wx0) * self.scale
    }
    fn sy(&self, wy: f32) -> f32 {
        self.y + 4.0 + (wy - self.wy0) * self.scale
    }
}

pub(crate) fn paint_minimap(
    doc: &DiagramDoc,
    xf: &Xf,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let mm = minimap_geom(doc, xf, bounds);
    let frame = Bounds::new(point(px(mm.x), px(mm.y)), size(px(MM_W), px(MM_H)));
    window.paint_quad(quad(
        frame,
        px(4.),
        c(PAPER),
        px(1.),
        c(0xc9c2ae),
        BorderStyle::Solid,
    ));
    window.with_content_mask(Some(ContentMask { bounds: frame }), |window| {
        for s in &doc.shapes {
            let b = Bounds::new(
                point(px(mm.sx(s.x)), px(mm.sy(s.y))),
                size(px((s.w * mm.scale).max(2.0)), px((s.h * mm.scale).max(2.0))),
            );
            window.paint_quad(fill(
                b,
                ca(BLUE, if is_container(doc, &s.id) { 0.25 } else { 0.9 }),
            ));
        }
        let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let (vx0, vy0) = xf.world(xf.ox, xf.oy);
        let (vx1, vy1) = xf.world(xf.ox + bw, xf.oy + bh);
        let v = Bounds::from_corners(
            point(px(mm.sx(vx0)), px(mm.sy(vy0))),
            point(px(mm.sx(vx1)), px(mm.sy(vy1))),
        );
        window.paint_quad(quad(
            v,
            px(1.),
            transparent_black(),
            px(1.5),
            c(BLUE),
            BorderStyle::Solid,
        ));
    });
}

/// Thumbnail painter (bottom drawer "Diagrams" tab).
pub fn paint_thumb(doc: &DiagramDoc, bounds: Bounds<Pixels>, window: &mut Window) {
    window.paint_quad(fill(bounds, c(PAPER)));
    let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let (mx, my) = bbox(doc)
        .map(|b| (b.2.max(400.0), b.3.max(300.0)))
        .unwrap_or((400.0, 300.0));
    let k = (bw / mx).min(bh / my);
    for s in &doc.shapes {
        let b = Bounds::new(
            point(bounds.origin.x + px(s.x * k), bounds.origin.y + px(s.y * k)),
            size(px((s.w * k).max(2.0)), px((s.h * k).max(2.0))),
        );
        window.paint_quad(quad(
            b,
            px(0.),
            transparent_black(),
            px(1.),
            c(0x777777),
            BorderStyle::Solid,
        ));
    }
}
