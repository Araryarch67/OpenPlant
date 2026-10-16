//! Shape and relation editing: add, move, resize, rename, delete, clipboard.

use super::*;
use crate::model::toolbox;
use crate::model::*;
use std::collections::HashMap;

impl Workbench {
    pub fn add_shape(&mut self, tool: &str, x: f32, y: f32, label: Option<String>) -> String {
        self.checkpoint();
        let kind = toolbox::shape_kind(tool);
        let (w, h) = toolbox::default_size(kind);
        let n = self.doc().shapes.len() + 1;
        let shape = Shape {
            id: nid("c"),
            label: label.unwrap_or_else(|| toolbox::default_label(tool, n)),
            x: x.round(),
            y: y.round(),
            w,
            h,
            kind: Some(kind),
            tool: Some(tool.to_string()),
            parent_id: None,
            note_side: None,
            note_of: None,
            style: None,
        };
        let id = shape.id.clone();
        self.project.active_mut().shapes.push(shape);
        self.sync_nodes();
        self.touch();
        self.select_shapes(vec![id.clone()]);
        id
    }

    pub fn add_relation(&mut self, from: &str, to: &str, tool: &str) -> bool {
        let doc = self.doc();
        if doc.shape(from).is_none() || doc.shape(to).is_none() {
            return false;
        }
        self.checkpoint();
        let kind = toolbox::edge_kind(tool).unwrap_or(RelationKind::DirectedAssociation);
        let rel = Relation {
            id: nid("r"),
            from: from.into(),
            to: to.into(),
            kind,
            tool: Some(tool.into()),
            label: None,
        };
        let id = rel.id.clone();
        self.project.active_mut().relations.push(rel);
        self.touch();
        self.selected_shapes.clear();
        self.selected_relations = vec![id];
        true
    }

    /// Move a set of shapes (and their contained children) by a delta,
    /// starting from saved origins (drag gesture).
    pub fn move_shapes_to(&mut self, origins: &[(String, f32, f32)], dx: f32, dy: f32) {
        let doc = self.project.active_mut();
        for (id, ox, oy) in origins {
            if let Some(s) = doc.shapes.iter_mut().find(|s| &s.id == id) {
                s.x = ox + dx;
                s.y = oy + dy;
            }
        }
        self.touch();
    }

    /// Selected shapes plus every descendant (containers drag their content).
    pub fn drag_set(&self, ids: &[String]) -> Vec<(String, f32, f32)> {
        let doc = self.doc();
        let mut set: Vec<String> = ids.to_vec();
        let mut i = 0;
        while i < set.len() {
            let pid = set[i].clone();
            for s in &doc.shapes {
                if s.parent_id.as_deref() == Some(pid.as_str()) && !set.contains(&s.id) {
                    set.push(s.id.clone());
                }
            }
            i += 1;
        }
        set.iter()
            .filter_map(|id| doc.shape(id).map(|s| (s.id.clone(), s.x, s.y)))
            .collect()
    }

    pub fn nudge(&mut self, dx: f32, dy: f32) {
        if self.selected_shapes.is_empty() {
            return;
        }
        self.checkpoint();
        let origins = self.drag_set(&self.selected_shapes.clone());
        self.move_shapes_to(&origins, dx, dy);
    }

    pub fn resize_selected(&mut self, dw: f32, dh: f32) {
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
            s.w = (s.w + dw).max(20.0);
            s.h = (s.h + dh).max(20.0);
        }
        self.touch();
    }

    pub fn set_bounds(&mut self, id: &str, x: f32, y: f32, w: f32, h: f32) {
        if let Some(s) = self
            .project
            .active_mut()
            .shapes
            .iter_mut()
            .find(|s| s.id == id)
        {
            s.x = x;
            s.y = y;
            s.w = w.max(20.0);
            s.h = h.max(20.0);
        }
        self.touch();
    }

    pub fn rename_node(&mut self, id: &str, name: &str) {
        self.project.root.rename(id, name);
        for d in self.project.diagrams.values_mut() {
            if d.id == id {
                d.name = name.to_string();
            }
            for s in d.shapes.iter_mut().filter(|s| s.id == id) {
                s.label = name.to_string();
            }
        }
        if self.project.root.id == id {
            self.project.name = name.to_string();
        }
        self.touch();
    }

    pub fn set_relation_label(&mut self, id: &str, label: &str) {
        if let Some(r) = self
            .project
            .active_mut()
            .relations
            .iter_mut()
            .find(|r| r.id == id)
        {
            r.label = (!label.trim().is_empty()).then(|| label.to_string());
        }
        self.touch();
    }

    /// Delete selected shapes (with their relations) and selected relations.
    pub fn delete_selection(&mut self) {
        if !self.has_selection() {
            return;
        }
        self.checkpoint();
        let mut gone: Vec<String> = self.selected_shapes.clone();
        // Containers take their content with them.
        let all = self.drag_set(&gone);
        gone = all.into_iter().map(|(id, _, _)| id).collect();
        let rels = self.selected_relations.clone();
        let doc = self.project.active_mut();
        doc.shapes.retain(|s| !gone.contains(&s.id));
        doc.relations
            .retain(|r| !gone.contains(&r.from) && !gone.contains(&r.to) && !rels.contains(&r.id));
        self.project.root.drop_ids(&gone);
        self.selected_node = Some(self.project.active_diagram_id.clone());
        self.select_none();
        self.touch();
    }

    pub fn duplicate_selection(&mut self) {
        self.duplicate_by(10.0);
    }

    /// Ctrl+drag copy: the copy starts exactly on top of the original.
    pub fn duplicate_in_place(&mut self) {
        self.duplicate_by(0.0);
    }

    pub(crate) fn duplicate_by(&mut self, offset: f32) {
        if self.selected_shapes.is_empty() {
            return;
        }
        self.checkpoint();
        let sel = self.selected_shapes.clone();
        let ids = self.paste_shapes(
            self.doc()
                .shapes
                .iter()
                .filter(|s| sel.contains(&s.id))
                .cloned()
                .collect(),
            self.doc()
                .relations
                .iter()
                .filter(|r| sel.contains(&r.from) && sel.contains(&r.to))
                .cloned()
                .collect(),
            offset,
        );
        self.select_shapes(ids);
    }

    pub(crate) fn paste_shapes(
        &mut self,
        shapes: Vec<Shape>,
        rels: Vec<Relation>,
        offset: f32,
    ) -> Vec<String> {
        let mut map: HashMap<String, String> = HashMap::new();
        let moved: Vec<Shape> = shapes
            .into_iter()
            .map(|mut s| {
                let id = nid("c");
                map.insert(s.id.clone(), id.clone());
                s.id = id;
                s.x += offset;
                s.y += offset;
                s
            })
            .collect();
        let copies: Vec<Shape> = moved
            .into_iter()
            .map(|mut s| {
                s.parent_id = s.parent_id.and_then(|p| map.get(&p).cloned());
                s.note_of = s.note_of.map(|n| map.get(&n).cloned().unwrap_or(n));
                s
            })
            .collect();
        let new_rels: Vec<Relation> = rels
            .into_iter()
            .filter_map(|mut r| {
                r.from = map.get(&r.from)?.clone();
                r.to = map.get(&r.to)?.clone();
                r.id = nid("r");
                Some(r)
            })
            .collect();
        let ids = copies.iter().map(|s| s.id.clone()).collect();
        let doc = self.project.active_mut();
        doc.shapes.extend(copies);
        doc.relations.extend(new_rels);
        self.sync_nodes();
        self.touch();
        ids
    }

    pub fn copy_selection(&mut self) -> usize {
        let sel = self.selected_shapes.clone();
        let doc = self.doc();
        let shapes: Vec<Shape> = doc
            .shapes
            .iter()
            .filter(|s| sel.contains(&s.id))
            .cloned()
            .collect();
        let rels: Vec<Relation> = doc
            .relations
            .iter()
            .filter(|r| sel.contains(&r.from) && sel.contains(&r.to))
            .cloned()
            .collect();
        self.clipboard = shapes;
        self.clip_relations = rels;
        self.clipboard.len()
    }

    pub fn cut_selection(&mut self) {
        if self.copy_selection() > 0 {
            self.delete_selection();
        }
    }

    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        self.checkpoint();
        let ids = self.paste_shapes(self.clipboard.clone(), self.clip_relations.clone(), 24.0);
        // Next paste cascades further.
        for s in &mut self.clipboard {
            s.x += 24.0;
            s.y += 24.0;
        }
        self.select_shapes(ids);
    }

    pub fn copy_size(&mut self) {
        if let Some(s) = self
            .selected_shapes
            .first()
            .and_then(|id| self.doc().shape(id))
        {
            self.copied_size = Some((s.w, s.h));
        }
    }

    pub fn paste_size(&mut self) {
        let Some((w, h)) = self.copied_size else {
            return;
        };
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
            s.w = w;
            s.h = h;
        }
        self.touch();
    }

    /// Ctrl+Shift+Y: fit selected shapes to their label.
    pub fn autosize(&mut self) {
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
            let lines: Vec<&str> = s.label.split('\n').collect();
            let wide = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as f32;
            let fs = s
                .style
                .as_ref()
                .and_then(|st| st.font_size)
                .unwrap_or(DEFAULT_FONT_SIZE);
            let n = lines.len() as f32;
            let (min_w, min_h) = toolbox::default_size(s.kind());
            match s.kind() {
                ShapeKind::Package => {}
                _ => {
                    s.w = (wide * fs * 0.58 + 36.0).max(min_w.min(100.0));
                    s.h = (n * fs * 1.15 + 30.0).max(min_h.min(56.0));
                }
            }
        }
        self.touch();
    }
}
