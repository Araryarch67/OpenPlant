//! Grouped layout for diagrams with package blocks.

use super::*;
use crate::model::{Shape, ShapeKind, Side};
use std::collections::HashMap;

/// Grouped layout like PlantUML: members gridded inside their package
/// (recursive), top-level blocks flowed into two balanced columns, notes
/// beside their target. Deterministic by declaration order.
pub(crate) fn layout_grouped(p: &mut Parser) {
    const PAD: f32 = 20.0;
    const GAP: f32 = 18.0;
    const HEADER: f32 = 36.0;
    const COLS: usize = 3;
    let pos: HashMap<String, usize> = p
        .shapes
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();
    let clamp = |v: f32, lo: f32, hi: f32| v.max(lo).min(hi);
    for sh in &mut p.shapes {
        let parts: Vec<&str> = sh.label.split('\n').collect();
        let wide = parts.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f32;
        let n = parts.len() as f32;
        match sh.kind() {
            ShapeKind::Box => {
                sh.w = clamp(wide * 8.0 + 36.0, 140.0, 240.0);
                sh.h = (30.0 + n * 18.0).max(62.0);
            }
            ShapeKind::Usecase => {
                sh.w = clamp(wide * 8.0 + 44.0, 130.0, 220.0);
                sh.h = (28.0 + n * 18.0).max(56.0);
            }
            ShapeKind::Actor => {
                sh.w = 120.0;
                sh.h = 130.0;
            }
            ShapeKind::Database | ShapeKind::Component => {
                sh.w = 190.0;
                sh.h = (52.0 + n * 18.0).max(100.0);
            }
            ShapeKind::Note => {
                sh.w = 215.0;
                sh.h = (34.0 + n * 16.0).max(72.0);
            }
            _ => {}
        }
    }
    let shapes = &mut p.shapes;
    let children_of = &p.children_of;
    let is_note = |shapes: &Vec<Shape>, id: &str| shapes[pos[id]].kind() == ShapeKind::Note;
    let has_kids = |id: &str| children_of.get(id).is_some_and(|k| !k.is_empty());

    fn layout_group(
        members: &[String],
        ox: f32,
        oy: f32,
        shapes: &mut Vec<Shape>,
        pos: &HashMap<String, usize>,
        children_of: &HashMap<String, Vec<String>>,
    ) -> (f32, f32) {
        let has_kids = |id: &str| children_of.get(id).is_some_and(|k| !k.is_empty());
        let subs: Vec<&String> = members.iter().filter(|id| has_kids(id)).collect();
        let leaves: Vec<&String> = members
            .iter()
            .filter(|id| !has_kids(id) && shapes[pos[*id]].kind() != ShapeKind::Note)
            .collect();
        let (mut x, mut y) = (ox + PAD, oy + HEADER + PAD);
        let (mut row_h, mut col, mut right) = (0.0f32, 0usize, ox + PAD);
        for id in &leaves {
            let s = &mut shapes[pos[*id]];
            if col >= COLS {
                col = 0;
                x = ox + PAD;
                y += row_h + GAP;
                row_h = 0.0;
            }
            s.x = x.round();
            s.y = y.round();
            x += s.w + GAP;
            row_h = row_h.max(s.h);
            col += 1;
            right = right.max(x - GAP);
        }
        if !leaves.is_empty() {
            y += row_h + GAP;
        }
        for sub in &subs {
            let kids = children_of.get(*sub).cloned().unwrap_or_default();
            let (w, h) = layout_group(&kids, ox + PAD, y, shapes, pos, children_of);
            let ps = &mut shapes[pos[*sub]];
            ps.x = ox + PAD;
            ps.y = y;
            ps.w = w;
            ps.h = h;
            y += h + GAP;
            right = right.max(ox + PAD + w);
        }
        let w = (right - ox + PAD).max(200.0);
        let content_h = y - oy - HEADER - PAD;
        let gap = if !leaves.is_empty() || !subs.is_empty() {
            GAP
        } else {
            0.0
        };
        (w, HEADER + PAD + (content_h - gap).max(0.0) + PAD)
    }

    fn shift_tree(
        id: &str,
        dx: f32,
        dy: f32,
        shapes: &mut Vec<Shape>,
        pos: &HashMap<String, usize>,
        children_of: &HashMap<String, Vec<String>>,
    ) {
        for kid in children_of.get(id).cloned().unwrap_or_default() {
            let k = &mut shapes[pos[&kid]];
            k.x += dx;
            k.y += dy;
            shift_tree(&kid, dx, dy, shapes, pos, children_of);
        }
    }

    let top: Vec<String> = shapes
        .iter()
        .filter(|s| !p.parent_of.contains_key(&s.id) && s.kind() != ShapeKind::Note)
        .map(|s| s.id.clone())
        .collect();
    let mut blocks = vec![];
    for id in &top {
        if has_kids(id) {
            let kids = children_of[id].clone();
            let (w, h) = layout_group(&kids, 0.0, 0.0, shapes, &pos, children_of);
            blocks.push((id.clone(), w, h));
        } else {
            let s = &shapes[pos[id]];
            blocks.push((id.clone(), s.w, s.h));
        }
    }
    // Two balanced columns keep wide diagrams from getting too tall.
    let mut col_h = [0.0f32; 2];
    let mut col_w = [0.0f32; 2];
    let col_of: Vec<usize> = blocks
        .iter()
        .map(|(_, w, h)| {
            let c = if col_h[0] <= col_h[1] { 0 } else { 1 };
            col_h[c] += h + 60.0;
            col_w[c] = col_w[c].max(*w);
            c
        })
        .collect();
    let col_x = [80.0, 80.0 + col_w[0] + 80.0];
    let mut col_y = [60.0f32, 60.0];
    for (i, (id, w, h)) in blocks.iter().enumerate() {
        let c = col_of[i];
        if has_kids(id) {
            let s = &mut shapes[pos[id]];
            s.w = *w;
            s.h = *h;
            shift_tree(id, col_x[c], col_y[c], shapes, &pos, children_of);
        }
        let s = &mut shapes[pos[id]];
        s.x = col_x[c].round();
        s.y = col_y[c].round();
        col_y[c] += h + 60.0;
    }

    // Notes sit beside their target; first side that hits nothing else wins
    // (target and its ancestor containers are ignored).
    for (note_id, _, _) in &p.note_targets {
        let n = &mut shapes[pos[note_id]];
        n.x = -100000.0;
        n.y = -100000.0;
    }
    let mut spill_x = 80.0f32;
    for sh in shapes.iter() {
        if sh.kind() != ShapeKind::Note {
            spill_x = spill_x.max(sh.x + sh.w + 40.0);
        }
    }
    for (note_id, side, alias) in &p.note_targets {
        let ni = pos[note_id];
        let (nw, nh) = (shapes[ni].w, shapes[ni].h);
        let Some(ti) = p.index.get(alias).and_then(|tid| pos.get(tid)).copied() else {
            shapes[ni].x = spill_x;
            shapes[ni].y = 60.0;
            spill_x += nw + 40.0;
            continue;
        };
        let t = shapes[ti].clone();
        let mut ignore = vec![t.id.clone()];
        let mut cur = p.parent_of.get(&t.id);
        while let Some(c) = cur {
            ignore.push(c.clone());
            cur = p.parent_of.get(c);
        }
        let at = |s: Side| match s {
            Side::Right => (t.x + t.w + 24.0, t.y),
            Side::Left => (t.x - nw - 24.0, t.y),
            Side::Top => (t.x, t.y - nh - 24.0),
            Side::Bottom => (t.x, t.y + t.h + 24.0),
        };
        let mut order = vec![*side];
        for s in [Side::Right, Side::Bottom, Side::Left, Side::Top] {
            if !order.contains(&s) {
                order.push(s);
            }
        }
        let hits = |x: f32, y: f32, shapes: &Vec<Shape>| {
            shapes.iter().any(|s| {
                !ignore.contains(&s.id)
                    && s.id != *note_id
                    && x < s.x + s.w + 10.0
                    && s.x - 10.0 < x + nw
                    && y < s.y + s.h + 10.0
                    && s.y - 10.0 < y + nh
            })
        };
        let spot = order
            .iter()
            .map(|s| at(*s))
            .find(|(x, y)| !hits(*x, *y, shapes));
        let n = &mut shapes[ni];
        match spot {
            Some((x, y)) => {
                n.x = x;
                n.y = y;
            }
            None => {
                n.x = spill_x;
                n.y = 60.0;
                spill_x += nw + 40.0;
            }
        }
        n.note_side = Some(*side);
        n.note_of = Some(t.id.clone());
    }
    let _ = is_note;
}
