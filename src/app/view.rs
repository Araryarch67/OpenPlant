//! Zoom, pan and fit.

use super::*;
use crate::canvas::*;

impl OpenPlant {
    pub fn zoom_at(&mut self, k2: f32, lx: f32, ly: f32) {
        self.auto_fit = None;
        let k = self.wb.zoom;
        let k2 = k2.clamp(MIN_ZOOM, MAX_ZOOM);
        let (px_, py_) = self.wb.pan;
        self.wb.pan = (lx - (lx - px_) * k2 / k, ly - (ly - py_) * k2 / k);
        self.wb.zoom = k2;
    }

    pub fn zoom_by(&mut self, factor: f32) {
        let (_, _, w, h) = self.canvas_bounds.get();
        self.zoom_at(self.wb.zoom * factor, w / 2.0, h / 2.0);
    }

    pub fn reset_view(&mut self) {
        self.auto_fit = None;
        self.wb.zoom = 1.0;
        self.wb.pan = (0.0, 0.0);
    }

    /// Fit after open/import, and keep fitting while the canvas is still
    /// settling (window tiling, maximising, panels resizing) until the user
    /// pans or zooms.
    pub(crate) fn keep_fitted(&mut self, window: &mut Window) {
        let (_, _, w, h) = self.canvas_bounds.get();
        if w <= 0.0 {
            if self.pending_fit {
                window.request_animation_frame();
            }
            return;
        }
        if std::mem::take(&mut self.pending_fit) || self.auto_fit.is_some_and(|s| s != (w, h)) {
            self.fit_window();
            self.auto_fit = Some((w, h));
        }
    }

    pub fn fit_window(&mut self) {
        let Some((x0, y0, x1, y1)) = bbox(self.wb.doc()) else {
            self.reset_view();
            return;
        };
        let (_, _, w, h) = self.canvas_bounds.get();
        let pad = 40.0;
        let title_room = if self.wb.doc().title.is_some() {
            60.0
        } else {
            0.0
        };
        let k = ((w - pad * 2.0) / (x1 - x0).max(1.0))
            .min((h - pad * 2.0) / (y1 - y0 + title_room).max(1.0));
        let k = k.clamp(MIN_ZOOM, 2.0);
        self.wb.zoom = k;
        self.wb.pan = (
            w / 2.0 - (x0 + x1) / 2.0 * k,
            h / 2.0 - ((y0 - title_room + y1) / 2.0) * k,
        );
    }

    pub(crate) fn center_on(&mut self, (wx, wy): (f32, f32)) {
        self.auto_fit = None;
        let (_, _, w, h) = self.canvas_bounds.get();
        let k = self.wb.zoom;
        self.wb.pan = (w / 2.0 - wx * k, h / 2.0 - wy * k);
    }
}
