//! Canvas pointer state machine (select, move, resize, connect, pan, rubber band).

use super::*;
use crate::canvas::*;
use crate::model::toolbox::{self, is_edge_tool};
use crate::workbench::*;
use gpui_kit::*;

impl OpenPlant {
    pub fn canvas_down(
        &mut self,
        ev: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.canvas_focus, cx);
        // We manage focus here; stop GPUI's focus-on-click from pulling it
        // back from an inline label editor opened by this press.
        window.prevent_default();
        let had_popup = self.any_popup();
        self.close_popups();
        if self.editing.is_some() {
            self.commit_edit(window, cx);
        }
        let (mx, my) = (f32::from(ev.position.x), f32::from(ev.position.y));
        let xf = self.xf();
        let (wx, wy) = xf.world(mx, my);
        cx.notify();
        match ev.button {
            MouseButton::Middle => {
                self.drag = Drag::Pan {
                    sx: mx,
                    sy: my,
                    px: self.wb.pan.0,
                    py: self.wb.pan.1,
                    moved: false,
                    right: false,
                };
                return;
            }
            MouseButton::Right => {
                self.drag = Drag::Pan {
                    sx: mx,
                    sy: my,
                    px: self.wb.pan.0,
                    py: self.wb.pan.1,
                    moved: false,
                    right: true,
                };
                return;
            }
            MouseButton::Left => {}
            _ => return,
        }
        if had_popup {
            return;
        }
        if self.wb.show_minimap {
            let mm = minimap_geom(self.wb.doc(), &xf, self.canvas_rect());
            if mm.contains(mx, my) {
                self.drag = Drag::MiniMap;
                self.center_on(mm.to_world(mx, my));
                return;
            }
        }
        if self.space_down {
            self.drag = Drag::Pan {
                sx: mx,
                sy: my,
                px: self.wb.pan.0,
                py: self.wb.pan.1,
                moved: false,
                right: false,
            };
            return;
        }
        let tool = self.wb.active_tool.clone();
        let doc = self.wb.doc();
        let hit = shape_at(doc, wx, wy);
        if tool != "Select" && !is_edge_tool(&tool) {
            let (w, h) = toolbox::default_size(toolbox::shape_kind(&tool));
            self.wb.add_shape(&tool, wx - w / 2.0, wy - h / 2.0, None);
            return;
        }
        if is_edge_tool(&tool) {
            match (hit, self.pending_from.clone()) {
                (Some(id), None) => {
                    self.pending_from = Some(id.clone());
                    self.wb.select_shapes(vec![id]);
                }
                (Some(id), Some(from)) if id == from => self.pending_from = None,
                (Some(id), Some(from)) => {
                    self.wb.add_relation(&from, &id, &tool);
                    let name = |i: &str| {
                        self.wb
                            .doc()
                            .shape(i)
                            .map(|s| s.label.clone())
                            .unwrap_or_default()
                    };
                    let msg = format!("{tool}: {} → {}.", name(&from), name(&id));
                    self.pending_from = None;
                    self.toast(ToastKind::Success, msg, window, cx);
                }
                (None, _) => self.pending_from = None,
            }
            return;
        }
        if ev.click_count >= 2 {
            if let Some(id) = hit {
                self.begin_edit(EditTarget::Shape(id), window, cx);
            } else if let Some(rid) = relation_at(doc, wx, wy, 6.0 / xf.k) {
                self.begin_edit(EditTarget::Relation(rid), window, cx);
            } else {
                // draw.io: double-click on blank opens the quick insert menu.
                self.ctx_menu = Some(CtxMenu {
                    x: mx,
                    y: my,
                    wx,
                    wy,
                    shape: None,
                    sub: Some(0),
                });
            }
            return;
        }
        if self.wb.selected_shapes.len() == 1
            && let Some(s) = doc.shape(&self.wb.selected_shapes[0])
            && let Some(h) = handle_at(s, &xf, mx, my)
        {
            self.drag = Drag::Resize {
                id: s.id.clone(),
                handle: h,
                orig: (s.x, s.y, s.w, s.h),
                sx: mx,
                sy: my,
                started: false,
            };
            return;
        }
        if let Some(hid) = self.hover.clone()
            && let Some(s) = doc.shape(&hid)
            && connect_arrow_at(s, &xf, mx, my)
        {
            self.drag = Drag::Connect {
                from: hid,
                cx: mx,
                cy: my,
                target: None,
            };
            return;
        }
        let shift = ev.modifiers.shift;
        if ev.modifiers.alt || hit.is_none() {
            if let Some(rid) = relation_at(doc, wx, wy, 6.0 / xf.k).filter(|_| !ev.modifiers.alt) {
                self.wb.select_relation(&rid, shift);
                return;
            }
            if !shift {
                self.wb.select_none();
                self.wb.selected_node = Some(self.wb.project.active_diagram_id.clone());
            }
            self.drag = Drag::Rubber {
                sx: mx,
                sy: my,
                cx: mx,
                cy: my,
                additive: shift,
            };
            return;
        }
        let id = hit.unwrap();
        let was_selected = self.wb.selected_shapes.contains(&id);
        if shift {
            self.wb.toggle_shape(&id);
        } else if !was_selected {
            self.wb.select_shapes(vec![id.clone()]);
        } else {
            self.wb.selected_node = Some(id.clone());
        }
        let origins = self.wb.drag_set(&self.wb.selected_shapes.clone());
        self.drag = Drag::Move {
            origins,
            sx: mx,
            sy: my,
            moved: false,
            clone: ev.modifiers.control,
            click: id,
            was_selected,
        };
    }

    pub fn root_move(&mut self, ev: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let (mx, my) = (f32::from(ev.position.x), f32::from(ev.position.y));
        let xf = self.xf();
        let k = xf.k;
        let snap = !ev.modifiers.alt;
        let mut notify = true;
        match &mut self.drag {
            Drag::None => {
                notify = self.update_hover(mx, my);
            }
            Drag::Pan {
                sx,
                sy,
                px,
                py,
                moved,
                ..
            } => {
                if (mx - *sx).abs() + (my - *sy).abs() > 3.0 {
                    *moved = true;
                }
                self.auto_fit = None;
                self.wb.pan = (*px + mx - *sx, *py + my - *sy);
            }
            Drag::Move {
                origins,
                sx,
                sy,
                moved,
                clone,
                ..
            } => {
                let (mut dx, mut dy) = ((mx - *sx) / k, (my - *sy) / k);
                if !*moved {
                    if (mx - *sx).abs() + (my - *sy).abs() < 3.0 {
                        return;
                    }
                    *moved = true;
                    if *clone {
                        // Ctrl+drag (draw.io): move a copy, leave the original.
                        self.wb.duplicate_in_place();
                        *origins = self.wb.drag_set(&self.wb.selected_shapes.clone());
                    } else {
                        self.wb.checkpoint();
                    }
                }
                // Shift locks the minor axis; the locked axis never snaps.
                let lock = ev.modifiers.shift.then(|| dx.abs() > dy.abs());
                if snap && let Some((_, ox, oy)) = origins.first() {
                    dx = ((ox + dx) / GRID).round() * GRID - ox;
                    dy = ((oy + dy) / GRID).round() * GRID - oy;
                }
                match lock {
                    Some(true) => dy = 0.0,
                    Some(false) => dx = 0.0,
                    None => {}
                }
                let o = origins.clone();
                self.wb.move_shapes_to(&o, dx, dy);
            }
            Drag::Rubber { cx: x, cy: y, .. } => {
                *x = mx;
                *y = my;
            }
            Drag::Resize {
                id,
                handle,
                orig,
                sx,
                sy,
                started,
            } => {
                if !*started {
                    *started = true;
                    self.wb.checkpoint();
                }
                let (dx, dy) = ((mx - *sx) / k, (my - *sy) / k);
                let (x, y, w, h) = *orig;
                let (mut x0, mut y0, mut x1, mut y1) = (x, y, x + w, y + h);
                // Only the edges being dragged move (and snap).
                let g = |v: f32| if snap { (v / GRID).round() * GRID } else { v };
                match handle {
                    Handle::N | Handle::NE | Handle::NW => y0 = g(y0 + dy),
                    Handle::S | Handle::SE | Handle::SW => y1 = g(y1 + dy),
                    _ => {}
                }
                match handle {
                    Handle::W | Handle::NW | Handle::SW => x0 = g(x0 + dx),
                    Handle::E | Handle::NE | Handle::SE => x1 = g(x1 + dx),
                    _ => {}
                }
                let id = id.clone();
                self.wb.set_bounds(
                    &id,
                    x0.min(x1 - 20.0),
                    y0.min(y1 - 20.0),
                    (x1 - x0).abs(),
                    (y1 - y0).abs(),
                );
            }
            Drag::Connect {
                from,
                cx: x,
                cy: y,
                target,
            } => {
                *x = mx;
                *y = my;
                let (wx, wy) = xf.world(mx, my);
                *target = shape_at(self.wb.doc(), wx, wy).filter(|t| t != from);
            }
            Drag::MiniMap => {
                let mm = minimap_geom(self.wb.doc(), &xf, self.canvas_rect());
                self.center_on(mm.to_world(mx, my));
            }
            Drag::Splitter { left, sx, w0 } => {
                let dx = mx - *sx;
                if *left {
                    self.wb.sidebar_w = (*w0 + dx).clamp(180.0, 420.0);
                } else {
                    self.wb.navigator_w = (*w0 - dx).clamp(220.0, 480.0);
                }
            }
        }
        if notify {
            cx.notify();
        }
    }

    /// Hover + cursor feedback; returns whether anything changed.
    pub(crate) fn update_hover(&mut self, mx: f32, my: f32) -> bool {
        let (x, y, w, h) = self.canvas_bounds.get();
        let inside = mx >= x && mx <= x + w && my >= y && my <= y + h;
        let xf = self.xf();
        let (wx, wy) = xf.world(mx, my);
        let doc = self.wb.doc();
        let mut cursor = if self.wb.active_tool == "Select" {
            CursorStyle::Arrow
        } else {
            CursorStyle::Crosshair
        };
        let mut hover = None;
        if inside && self.wb.active_tool == "Select" {
            // Keep the hovered shape while the pointer is on its connect arrows.
            if let Some(prev) = self.hover.as_ref().and_then(|h| doc.shape(h))
                && connect_arrow_at(prev, &xf, mx, my)
            {
                hover = Some(prev.id.clone());
                cursor = CursorStyle::PointingHand;
            } else {
                hover = shape_at(doc, wx, wy);
            }
            if let Some(s) = self.wb.selected_shapes.first().and_then(|id| doc.shape(id))
                && self.wb.selected_shapes.len() == 1
                && let Some(hd) = handle_at(s, &xf, mx, my)
            {
                cursor = match hd {
                    Handle::N | Handle::S => CursorStyle::ResizeUpDown,
                    Handle::E | Handle::W => CursorStyle::ResizeLeftRight,
                    Handle::NE | Handle::SW => CursorStyle::ResizeUpRightDownLeft,
                    Handle::NW | Handle::SE => CursorStyle::ResizeUpLeftDownRight,
                };
            } else if hover.is_some() && cursor == CursorStyle::Arrow {
                cursor = CursorStyle::OpenHand;
            }
        }
        if self.space_down && inside {
            cursor = CursorStyle::OpenHand;
        }
        let changed = hover != self.hover || cursor != self.cursor;
        self.hover = hover;
        self.cursor = cursor;
        changed
    }

    pub fn root_up(&mut self, ev: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        let drag = std::mem::replace(&mut self.drag, Drag::None);
        let xf = self.xf();
        match drag {
            Drag::Move {
                moved: false,
                click,
                was_selected: true,
                ..
            } if !ev.modifiers.shift => {
                self.wb.select_shapes(vec![click]);
            }
            Drag::Rubber {
                sx,
                sy,
                cx: ex,
                cy: ey,
                additive,
            } => {
                let (ax, ay) = xf.world(sx.min(ex), sy.min(ey));
                let (bx, by) = xf.world(sx.max(ex), sy.max(ey));
                if (ex - sx).abs() + (ey - sy).abs() > 3.0 {
                    let doc = self.wb.doc();
                    let mut ids: Vec<String> = doc
                        .shapes
                        .iter()
                        .filter(|s| s.x >= ax && s.y >= ay && s.x + s.w <= bx && s.y + s.h <= by)
                        .map(|s| s.id.clone())
                        .collect();
                    if additive {
                        for id in self.wb.selected_shapes.clone() {
                            if !ids.contains(&id) {
                                ids.push(id);
                            }
                        }
                    }
                    let rels: Vec<String> = doc
                        .relations
                        .iter()
                        .filter(|r| ids.contains(&r.from) && ids.contains(&r.to))
                        .map(|r| r.id.clone())
                        .collect();
                    self.wb.select_shapes(ids);
                    self.wb.selected_relations = rels;
                }
            }
            Drag::Connect {
                from,
                target: Some(t),
                ..
            } => {
                let tool = if is_edge_tool(&self.wb.active_tool) {
                    self.wb.active_tool.clone()
                } else {
                    "Directed Association".into()
                };
                self.wb.add_relation(&from, &t, &tool);
            }
            Drag::Pan {
                right: true,
                moved: false,
                sx,
                sy,
                ..
            } => {
                let (wx, wy) = xf.world(sx, sy);
                let shape = shape_at(self.wb.doc(), wx, wy);
                if let Some(id) = &shape
                    && !self.wb.selected_shapes.contains(id)
                {
                    self.wb.select_shapes(vec![id.clone()]);
                }
                self.ctx_menu = Some(CtxMenu {
                    x: sx,
                    y: sy,
                    wx,
                    wy,
                    shape,
                    sub: None,
                });
            }
            Drag::Splitter { .. } => {
                self.settings.sidebar_w = self.wb.sidebar_w;
                self.settings.navigator_w = self.wb.navigator_w;
                self.settings.save();
            }
            _ => {}
        }
        let _ = window;
        cx.notify();
    }

    pub fn canvas_wheel(&mut self, ev: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let (x, y, _, _) = self.canvas_bounds.get();
        let (lx, ly) = (f32::from(ev.position.x) - x, f32::from(ev.position.y) - y);
        let m = ev.modifiers;
        if m.control || m.alt {
            let f = match ev.delta {
                ScrollDelta::Lines(l) => (l.y * 0.2).exp(),
                ScrollDelta::Pixels(p) => (f32::from(p.y) * 0.01).exp(),
            };
            self.zoom_at(self.wb.zoom * f, lx, ly);
        } else {
            let d = ev.delta.pixel_delta(px(40.));
            let (dx, dy) = (f32::from(d.x), f32::from(d.y));
            self.auto_fit = None;
            if m.shift && dx == 0.0 {
                self.wb.pan.0 += dy;
            } else {
                self.wb.pan.0 += dx;
                self.wb.pan.1 += dy;
            }
        }
        cx.notify();
    }

    pub fn drop_tool(&mut self, tool: &str, window: &mut Window, cx: &mut Context<Self>) {
        let pos = window.mouse_position();
        let (wx, wy) = self.xf().world(f32::from(pos.x), f32::from(pos.y));
        if is_edge_tool(tool) {
            self.wb.active_tool = tool.to_string();
            self.pending_from = None;
            self.toast(
                ToastKind::Info,
                format!("{tool}: klik shape sumber, lalu klik shape tujuan."),
                window,
                cx,
            );
            return;
        }
        let (w, h) = toolbox::default_size(toolbox::shape_kind(tool));
        self.wb.add_shape(tool, wx - w / 2.0, wy - h / 2.0, None);
        window.focus(&self.canvas_focus, cx);
        cx.notify();
    }
}
