//! Diagram canvas: geometry, hit testing and painting with GPUI paths
//! (hand-drawn, no canvas library).

mod draw;
mod edges;
mod geometry;
mod minimap;
mod scene;
mod shapes;

pub(crate) use draw::*;
pub(crate) use edges::*;
pub(crate) use geometry::*;
pub(crate) use minimap::*;
pub(crate) use scene::*;
pub(crate) use shapes::*;

use crate::model::*;
use crate::theme::c;
use gpui_kit::*;

pub const GRID: f32 = 16.0;

pub const MIN_ZOOM: f32 = 0.1;

pub const MAX_ZOOM: f32 = 4.0;

pub const MM_W: f32 = 130.0;

pub const MM_H: f32 = 110.0;

pub(crate) const EDGE: u32 = 0x555555;

/// World <-> window transform for the canvas element.
#[derive(Clone, Copy, Debug, Default)]
pub struct Xf {
    pub ox: f32,
    pub oy: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub k: f32,
}

impl Xf {
    pub fn sx(&self, x: f32) -> f32 {
        self.ox + self.pan_x + x * self.k
    }
    pub fn sy(&self, y: f32) -> f32 {
        self.oy + self.pan_y + y * self.k
    }
    pub fn p(&self, x: f32, y: f32) -> Point<Pixels> {
        point(px(self.sx(x)), px(self.sy(y)))
    }
    pub fn world(&self, wx: f32, wy: f32) -> (f32, f32) {
        (
            (wx - self.ox - self.pan_x) / self.k,
            (wy - self.oy - self.pan_y) / self.k,
        )
    }
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(self.p(x, y), size(px(w * self.k), px(h * self.k)))
    }
}

pub fn hex(s: &str, fallback: u32) -> Hsla {
    c(parse_color(s).unwrap_or(fallback))
}
