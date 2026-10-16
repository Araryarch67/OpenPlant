//! Selection: shapes, relations, keyboard cycling.

use super::*;

impl Workbench {
    pub fn select_node(&mut self, id: Option<String>) {
        let is_shape = id.as_ref().is_some_and(|i| self.doc().shape(i).is_some());
        self.selected_shapes = if is_shape {
            vec![id.clone().unwrap()]
        } else {
            vec![]
        };
        self.selected_relations.clear();
        self.selected_node = id;
    }

    pub fn select_shapes(&mut self, ids: Vec<String>) {
        if let Some(first) = ids.first() {
            self.selected_node = Some(first.clone());
        }
        self.selected_shapes = ids;
        self.selected_relations.clear();
    }

    pub fn toggle_shape(&mut self, id: &str) {
        if let Some(i) = self.selected_shapes.iter().position(|s| s == id) {
            self.selected_shapes.remove(i);
        } else {
            self.selected_shapes.push(id.to_string());
            self.selected_node = Some(id.to_string());
        }
    }

    pub fn select_relation(&mut self, id: &str, additive: bool) {
        if !additive {
            self.selected_shapes.clear();
            self.selected_relations.clear();
        }
        if !self.selected_relations.iter().any(|r| r == id) {
            self.selected_relations.push(id.to_string());
        }
    }

    pub fn select_all(&mut self) {
        self.selected_shapes = self.doc().shapes.iter().map(|s| s.id.clone()).collect();
        self.selected_relations = self.doc().relations.iter().map(|r| r.id.clone()).collect();
    }

    pub fn select_vertices(&mut self) {
        self.selected_shapes = self.doc().shapes.iter().map(|s| s.id.clone()).collect();
        self.selected_relations.clear();
    }

    pub fn select_edges(&mut self) {
        self.selected_relations = self.doc().relations.iter().map(|r| r.id.clone()).collect();
        self.selected_shapes.clear();
    }

    pub fn select_none(&mut self) {
        self.selected_shapes.clear();
        self.selected_relations.clear();
    }

    /// Tab / Shift+Tab: cycle the single selection through shapes.
    pub fn select_step(&mut self, forward: bool) {
        let ids: Vec<String> = self.doc().shapes.iter().map(|s| s.id.clone()).collect();
        if ids.is_empty() {
            return;
        }
        let cur = self
            .selected_shapes
            .first()
            .and_then(|c| ids.iter().position(|i| i == c));
        let next = match (cur, forward) {
            (None, true) => 0,
            (None, false) => ids.len() - 1,
            (Some(i), true) => (i + 1) % ids.len(),
            (Some(i), false) => (i + ids.len() - 1) % ids.len(),
        };
        self.select_shapes(vec![ids[next].clone()]);
    }

    pub fn has_selection(&self) -> bool {
        !self.selected_shapes.is_empty() || !self.selected_relations.is_empty()
    }
}
