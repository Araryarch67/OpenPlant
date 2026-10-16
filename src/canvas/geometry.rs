//! Routing, hit testing, resize handles and connection arrows.

use super::*;
use crate::model::*;

pub struct Route {
    pub pts: Vec<(f32, f32)>,
    pub label: (f32, f32),
}

/// Manhattan path (PlantUML "ortho") between the facing borders of two shapes.
pub fn ortho_route(a: &Shape, b: &Shape) -> Route {
    let (ax, ay) = a.center();
    let (bx, by) = b.center();
    if a.id == b.id {
        let x = a.x + a.w;
        let y = a.y - 26.0;
        return Route {
            pts: vec![
                (x - 30.0, a.y),
                (x - 30.0, y),
                (x + 6.0, y),
                (x + 6.0, a.y + 24.0),
            ],
            label: (x - 12.0, y - 6.0),
        };
    }
    if (bx - ax).abs() >= (by - ay).abs() {
        let p1 = (if bx >= ax { a.x + a.w } else { a.x }, ay);
        let p2 = (if bx >= ax { b.x } else { b.x + b.w }, by);
        if (p2.1 - p1.1).abs() < 2.0 {
            return Route {
                pts: vec![p1, p2],
                label: ((p1.0 + p2.0) / 2.0, p1.1),
            };
        }
        let mx = ((p1.0 + p2.0) / 2.0).round();
        return Route {
            pts: vec![p1, (mx, p1.1), (mx, p2.1), p2],
            label: (mx, (p1.1 + p2.1) / 2.0),
        };
    }
    let p1 = (ax, if by >= ay { a.y + a.h } else { a.y });
    let p2 = (bx, if by >= ay { b.y } else { b.y + b.h });
    if (p2.0 - p1.0).abs() < 2.0 {
        return Route {
            pts: vec![p1, p2],
            label: (p1.0, (p1.1 + p2.1) / 2.0),
        };
    }
    let my = ((p1.1 + p2.1) / 2.0).round();
    Route {
        pts: vec![p1, (p1.0, my), (p2.0, my), p2],
        label: ((p1.0 + p2.0) / 2.0, my),
    }
}

/// Straight line from the border of `a` toward the border of `b` (note links).
pub fn straight_route(a: &Shape, b: &Shape) -> Vec<(f32, f32)> {
    let (ax, ay) = a.center();
    let (bx, by) = b.center();
    vec![clip_rect(a, bx, by), clip_rect(b, ax, ay)]
}

pub(crate) fn clip_rect(s: &Shape, tx: f32, ty: f32) -> (f32, f32) {
    let (cx, cy) = s.center();
    let (dx, dy) = (tx - cx, ty - cy);
    if dx == 0.0 && dy == 0.0 {
        return (cx, cy);
    }
    let sx = if dx != 0.0 {
        (s.w / 2.0) / dx.abs()
    } else {
        f32::MAX
    };
    let sy = if dy != 0.0 {
        (s.h / 2.0) / dy.abs()
    } else {
        f32::MAX
    };
    let t = sx.min(sy);
    (cx + dx * t, cy + dy * t)
}

pub(crate) fn seg_dist(px_: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((px_ - a.0) * dx + (py - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((px_ - qx).powi(2) + (py - qy).powi(2)).sqrt()
}

pub fn relation_at(doc: &DiagramDoc, wx: f32, wy: f32, tol: f32) -> Option<String> {
    for r in doc.relations.iter().rev() {
        let (Some(a), Some(b)) = (doc.shape(&r.from), doc.shape(&r.to)) else {
            continue;
        };
        let route = ortho_route(a, b);
        if route
            .pts
            .windows(2)
            .any(|w| seg_dist(wx, wy, w[0], w[1]) <= tol)
        {
            return Some(r.id.clone());
        }
    }
    None
}

pub fn is_container(doc: &DiagramDoc, id: &str) -> bool {
    doc.shapes
        .iter()
        .any(|s| s.parent_id.as_deref() == Some(id))
}

/// Topmost shape under the point: leaves before containers (containers are
/// painted underneath), later shapes before earlier ones.
pub fn shape_at(doc: &DiagramDoc, wx: f32, wy: f32) -> Option<String> {
    let hit = |leaf: bool| {
        doc.shapes
            .iter()
            .rev()
            .filter(|s| is_container(doc, &s.id) != leaf)
            .find(|s| s.contains(wx, wy))
            .map(|s| s.id.clone())
    };
    hit(true).or_else(|| hit(false))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Handle {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

pub const HANDLES: [Handle; 8] = [
    Handle::NW,
    Handle::N,
    Handle::NE,
    Handle::E,
    Handle::SE,
    Handle::S,
    Handle::SW,
    Handle::W,
];

pub fn handle_pos(s: &Shape, h: Handle) -> (f32, f32) {
    let (x0, y0, x1, y1) = (s.x, s.y, s.x + s.w, s.y + s.h);
    let (mx, my) = s.center();
    match h {
        Handle::NW => (x0, y0),
        Handle::N => (mx, y0),
        Handle::NE => (x1, y0),
        Handle::E => (x1, my),
        Handle::SE => (x1, y1),
        Handle::S => (mx, y1),
        Handle::SW => (x0, y1),
        Handle::W => (x0, my),
    }
}

/// Resize handle under a window point (handles are a fixed 8px on screen).
pub fn handle_at(s: &Shape, xf: &Xf, mx: f32, my: f32) -> Option<Handle> {
    HANDLES.into_iter().find(|h| {
        let (hx, hy) = handle_pos(s, *h);
        (xf.sx(hx) - mx).abs() <= 6.0 && (xf.sy(hy) - my).abs() <= 6.0
    })
}

/// draw.io-style hover arrows: one per side, just outside the shape.
pub fn connect_arrows(s: &Shape, xf: &Xf) -> [(f32, f32, f32, f32); 4] {
    let (mx, my) = s.center();
    let gap = 16.0;
    [
        (xf.sx(mx), xf.sy(s.y) - gap, 0.0, -1.0),
        (xf.sx(s.x + s.w) + gap, xf.sy(my), 1.0, 0.0),
        (xf.sx(mx), xf.sy(s.y + s.h) + gap, 0.0, 1.0),
        (xf.sx(s.x) - gap, xf.sy(my), -1.0, 0.0),
    ]
}

pub fn connect_arrow_at(s: &Shape, xf: &Xf, mx: f32, my: f32) -> bool {
    connect_arrows(s, xf)
        .iter()
        .any(|(x, y, _, _)| (x - mx).abs() <= 9.0 && (y - my).abs() <= 9.0)
}

pub fn bbox(doc: &DiagramDoc) -> Option<(f32, f32, f32, f32)> {
    let mut it = doc.shapes.iter();
    let f = it.next()?;
    let mut b = (f.x, f.y, f.x + f.w, f.y + f.h);
    for s in it {
        b = (
            b.0.min(s.x),
            b.1.min(s.y),
            b.2.max(s.x + s.w),
            b.3.max(s.y + s.h),
        );
    }
    Some(b)
}
