//! Workbench state: the single source of truth for project, selection,
//! view and panels, plus undo/redo and draw.io-style editing operations.

mod arrange;
mod diagrams;
mod selection;
mod shapes;

use crate::model::*;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub msg: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlignMode {
    Left,
    CenterX,
    Right,
    Top,
    Middle,
    Bottom,
    DistH,
    DistV,
    SameW,
    SameH,
}

#[derive(Clone, PartialEq)]
pub enum AttrValue {
    Text(String),
    Bool(bool),
}

pub(crate) const HISTORY_LIMIT: usize = 200;

pub struct Workbench {
    pub project: Project,
    pub selected_node: Option<String>,
    pub selected_shapes: Vec<String>,
    pub selected_relations: Vec<String>,
    pub active_tool: String,
    pub open_groups: Vec<bool>,
    pub zoom: f32,
    pub pan: (f32, f32),
    pub show_sidebar: bool,
    pub show_navigator: bool,
    pub show_grid: bool,
    pub show_minimap: bool,
    pub bottom_tab: Option<String>,
    pub strip_tab: String,
    pub sidebar_w: f32,
    pub navigator_w: f32,
    pub extensions: HashMap<String, bool>,
    pub node_attrs: HashMap<String, HashMap<String, AttrValue>>,
    pub node_docs: HashMap<String, String>,
    pub toasts: Vec<Toast>,
    pub file_path: Option<std::path::PathBuf>,
    clipboard: Vec<Shape>,
    clip_relations: Vec<Relation>,
    copied_size: Option<(f32, f32)>,
    undo: Vec<Project>,
    redo: Vec<Project>,
    toast_seq: u64,
}

impl Default for Workbench {
    fn default() -> Self {
        Self::new()
    }
}

impl Workbench {
    pub fn new() -> Self {
        Self {
            project: default_project(),
            selected_node: Some("diag1".into()),
            selected_shapes: vec![],
            selected_relations: vec![],
            active_tool: "Select".into(),
            open_groups: vec![true, false, false, false, false],
            zoom: 1.0,
            pan: (0.0, 0.0),
            show_sidebar: true,
            show_navigator: true,
            show_grid: true,
            show_minimap: true,
            bottom_tab: None,
            strip_tab: "diagrams".into(),
            sidebar_w: 264.0,
            navigator_w: 300.0,
            extensions: HashMap::new(),
            node_attrs: HashMap::new(),
            node_docs: HashMap::new(),
            toasts: vec![],
            file_path: None,
            clipboard: vec![],
            clip_relations: vec![],
            copied_size: None,
            undo: vec![],
            redo: vec![],
            toast_seq: 0,
        }
    }

    pub fn doc(&self) -> &DiagramDoc {
        self.project.active()
    }

    pub fn ext_on(&self, id: &str) -> bool {
        self.extensions.get(id).copied().unwrap_or(true)
    }

    /// Snapshot before a mutation. Call once per user gesture.
    pub fn checkpoint(&mut self) {
        self.undo.push(self.project.clone());
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.project, prev));
        self.prune_selection();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.project, next));
        self.prune_selection();
        true
    }

    pub(crate) fn prune_selection(&mut self) {
        let doc = self.project.active();
        let shapes: HashSet<&str> = doc.shapes.iter().map(|s| s.id.as_str()).collect();
        let rels: HashSet<&str> = doc.relations.iter().map(|r| r.id.as_str()).collect();
        self.selected_shapes
            .retain(|id| shapes.contains(id.as_str()));
        self.selected_relations
            .retain(|id| rels.contains(id.as_str()));
    }

    pub(crate) fn touch(&mut self) {
        self.project.modified = true;
    }

    pub fn toast(&mut self, kind: ToastKind, msg: impl Into<String>) -> u64 {
        self.toast_seq += 1;
        let id = self.toast_seq;
        if self.toasts.len() >= 4 {
            self.toasts.remove(0);
        }
        self.toasts.push(Toast {
            id,
            kind,
            msg: msg.into(),
        });
        id
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn set_attr(&mut self, id: &str, key: &str, v: AttrValue) {
        self.node_attrs
            .entry(id.into())
            .or_default()
            .insert(key.into(), v);
    }

    pub fn attr_text(&self, id: &str, key: &str) -> Option<String> {
        match self.node_attrs.get(id)?.get(key)? {
            AttrValue::Text(t) => Some(t.clone()),
            AttrValue::Bool(_) => None,
        }
    }

    pub fn attr_bool(&self, id: &str, key: &str) -> bool {
        matches!(
            self.node_attrs.get(id).and_then(|m| m.get(key)),
            Some(AttrValue::Bool(true))
        )
    }

    /// Rebuild the active diagram's explorer children from its shapes;
    /// `parent_id` nests them (PlantUML `{ }` blocks).
    pub fn sync_nodes(&mut self) {
        let doc = self.project.active().clone();
        let ids: HashSet<&str> = doc.shapes.iter().map(|s| s.id.as_str()).collect();
        fn kind_for(s: &Shape) -> NodeKind {
            match (s.kind(), s.tool.as_deref()) {
                (ShapeKind::Interface, _) => NodeKind::Interface,
                (ShapeKind::Package, _) => NodeKind::Package,
                (_, Some("Model")) => NodeKind::Model,
                _ => NodeKind::Class,
            }
        }
        fn node(s: &Shape, doc: &DiagramDoc) -> ModelNode {
            ModelNode {
                id: s.id.clone(),
                name: s.label.clone(),
                kind: kind_for(s),
                children: doc
                    .shapes
                    .iter()
                    .filter(|c| c.parent_id.as_deref() == Some(s.id.as_str()))
                    .map(|c| node(c, doc))
                    .collect(),
            }
        }
        let kids: Vec<ModelNode> = doc
            .shapes
            .iter()
            .filter(|s| s.parent_id.as_deref().is_none_or(|p| !ids.contains(p)))
            .map(|s| node(s, &doc))
            .collect();
        if let Some(n) = self.project.root.find_mut(&doc.id) {
            n.children = kids;
        }
    }

    pub fn load_project(&mut self, p: Project) {
        bump_ids(&p);
        self.project = p;
        self.undo.clear();
        self.redo.clear();
        self.selected_node = Some(self.project.active_diagram_id.clone());
        self.select_none();
    }

    pub fn reset(&mut self) {
        let fresh = Workbench::new();
        self.project = fresh.project;
        self.undo.clear();
        self.redo.clear();
        self.node_attrs.clear();
        self.node_docs.clear();
        self.file_path = None;
        self.selected_node = Some("diag1".into());
        self.select_none();
        self.pan = (0.0, 0.0);
        self.zoom = 1.0;
    }

    pub fn diagrams_sorted(&self) -> Vec<(String, String)> {
        // Keep explorer order (creation order) rather than BTreeMap key order.
        let mut order = vec![];
        fn walk(n: &ModelNode, out: &mut Vec<String>) {
            if n.kind == NodeKind::Diagram {
                out.push(n.id.clone());
            }
            n.children.iter().for_each(|c| walk(c, out));
        }
        walk(&self.project.root, &mut order);
        let d: &BTreeMap<String, DiagramDoc> = &self.project.diagrams;
        let mut out: Vec<(String, String)> = order
            .iter()
            .filter_map(|id| d.get(id).map(|x| (x.id.clone(), x.name.clone())))
            .collect();
        for (id, doc) in d {
            if !order.contains(id) {
                out.push((id.clone(), doc.name.clone()));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests;
