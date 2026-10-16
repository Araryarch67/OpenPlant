//! Arrange and format: alignment, z-order, styles.

use super::*;
use crate::model::*;
use std::collections::HashMap;

impl Workbench {
    pub fn align(&mut self, mode: AlignMode) {
        let sel: Vec<Shape> = self
            .doc()
            .shapes
            .iter()
            .filter(|s| self.selected_shapes.contains(&s.id))
            .cloned()
            .collect();
        if sel.len() < 2 {
            return;
        }
        self.checkpoint();
        let min_x = sel.iter().map(|s| s.x).fold(f32::MAX, f32::min);
        let max_r = sel.iter().map(|s| s.x + s.w).fold(f32::MIN, f32::max);
        let min_y = sel.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let max_b = sel.iter().map(|s| s.y + s.h).fold(f32::MIN, f32::max);
        let mut out: HashMap<String, (f32, f32, f32, f32)> = sel
            .iter()
            .map(|s| (s.id.clone(), (s.x, s.y, s.w, s.h)))
            .collect();
        for s in &sel {
            let e = out.get_mut(&s.id).unwrap();
            match mode {
                AlignMode::Left => e.0 = min_x,
                AlignMode::Right => e.0 = max_r - s.w,
                AlignMode::CenterX => e.0 = (min_x + max_r) / 2.0 - s.w / 2.0,
                AlignMode::Top => e.1 = min_y,
                AlignMode::Bottom => e.1 = max_b - s.h,
                AlignMode::Middle => e.1 = (min_y + max_b) / 2.0 - s.h / 2.0,
                AlignMode::SameW => e.2 = sel[0].w,
                AlignMode::SameH => e.3 = sel[0].h,
                _ => {}
            }
        }
        if matches!(mode, AlignMode::DistH | AlignMode::DistV) {
            let horiz = mode == AlignMode::DistH;
            let mut ordered = sel.clone();
            ordered.sort_by(|a, b| {
                let (ka, kb) = if horiz { (a.x, b.x) } else { (a.y, b.y) };
                ka.total_cmp(&kb)
            });
            let first = if horiz { ordered[0].x } else { ordered[0].y };
            let last_s = ordered.last().unwrap();
            let last_end = if horiz {
                last_s.x + last_s.w
            } else {
                last_s.y + last_s.h
            };
            let total: f32 = ordered.iter().map(|s| if horiz { s.w } else { s.h }).sum();
            let gap = ((last_end - first - total) / (ordered.len() as f32 - 1.0)).max(0.0);
            let mut cursor = first;
            for s in &ordered {
                let e = out.get_mut(&s.id).unwrap();
                if horiz {
                    e.0 = cursor;
                    cursor += s.w + gap;
                } else {
                    e.1 = cursor;
                    cursor += s.h + gap;
                }
            }
        }
        for s in self.project.active_mut().shapes.iter_mut() {
            if let Some((x, y, w, h)) = out.get(&s.id) {
                s.x = x.round();
                s.y = y.round();
                s.w = *w;
                s.h = *h;
            }
        }
        self.touch();
    }

    /// Z-order: front/back moves to the end/start; forward/backward one step.
    pub fn reorder(&mut self, front: bool, step: bool) {
        if self.selected_shapes.is_empty() {
            return;
        }
        self.checkpoint();
        let sel = self.selected_shapes.clone();
        let shapes = &mut self.project.active_mut().shapes;
        if step {
            if front {
                for i in (0..shapes.len().saturating_sub(1)).rev() {
                    if sel.contains(&shapes[i].id) && !sel.contains(&shapes[i + 1].id) {
                        shapes.swap(i, i + 1);
                    }
                }
            } else {
                for i in 1..shapes.len() {
                    if sel.contains(&shapes[i].id) && !sel.contains(&shapes[i - 1].id) {
                        shapes.swap(i, i - 1);
                    }
                }
            }
        } else {
            let (mut picked, rest): (Vec<Shape>, Vec<Shape>) =
                shapes.drain(..).partition(|s| sel.contains(&s.id));
            if front {
                *shapes = rest;
                shapes.append(&mut picked);
            } else {
                picked.extend(rest);
                *shapes = picked;
            }
        }
        self.touch();
    }

    pub fn set_style(&mut self, patch: impl Fn(&mut ShapeStyle)) {
        if self.selected_shapes.is_empty() {
            return;
        }
        self.checkpoint();
        let sel = self.selected_shapes.clone();
        for s in self
            .project
            .active_mut()
            .shapes
            .iter_mut()
            .filter(|s| sel.contains(&s.id))
        {
            let mut st = s.style.clone().unwrap_or_default();
            patch(&mut st);
            s.style = Some(st);
        }
        self.touch();
    }

    pub fn clear_style(&mut self) {
        self.checkpoint();
        let sel = self.selected_shapes.clone();
        for s in self
            .project
            .active_mut()
            .shapes
            .iter_mut()
            .filter(|s| sel.contains(&s.id))
        {
            s.style = None;
        }
        self.touch();
    }
}
