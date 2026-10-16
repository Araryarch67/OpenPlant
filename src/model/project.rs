//! Project, model tree and diagrams.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Project,
    Model,
    Package,
    Class,
    Interface,
    Diagram,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramDoc {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub shapes: Vec<Shape>,
    pub relations: Vec<Relation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preamble: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legend: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl DiagramDoc {
    pub fn shape(&self, id: &str) -> Option<&Shape> {
        self.shapes.iter().find(|s| s.id == id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelNode {
    pub id: String,
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<ModelNode>,
}

impl ModelNode {
    pub fn find(&self, id: &str) -> Option<&ModelNode> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut ModelNode> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }

    pub fn rename(&mut self, id: &str, name: &str) {
        if let Some(n) = self.find_mut(id) {
            n.name = name.to_string();
        }
    }

    pub fn drop_ids(&mut self, ids: &[String]) {
        self.children.retain(|c| !ids.contains(&c.id));
        for c in &mut self.children {
            c.drop_ids(ids);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub modified: bool,
    pub root: ModelNode,
    pub diagrams: BTreeMap<String, DiagramDoc>,
    pub active_diagram_id: String,
}

impl Project {
    pub fn active(&self) -> &DiagramDoc {
        &self.diagrams[&self.active_diagram_id]
    }

    pub fn active_mut(&mut self) -> &mut DiagramDoc {
        self.diagrams
            .get_mut(&self.active_diagram_id)
            .expect("active diagram exists")
    }
}

pub(crate) fn class_node(id: &str, name: &str) -> ModelNode {
    ModelNode {
        id: id.into(),
        name: name.into(),
        kind: NodeKind::Class,
        children: vec![],
    }
}

pub fn default_project() -> Project {
    let shape = |id: &str, label: &str, x: f32, y: f32| Shape {
        id: id.into(),
        label: label.into(),
        x,
        y,
        w: 160.0,
        h: 96.0,
        kind: None,
        tool: None,
        parent_id: None,
        note_side: None,
        note_of: None,
        style: None,
    };
    let mut diagrams = BTreeMap::new();
    diagrams.insert(
        "diag1".to_string(),
        DiagramDoc {
            id: "diag1".into(),
            name: "Main".into(),
            kind: "Class".into(),
            shapes: vec![
                shape("c1", "Customer", 120.0, 100.0),
                shape("c2", "Order", 360.0, 220.0),
            ],
            relations: vec![Relation {
                id: "r1".into(),
                from: "c1".into(),
                to: "c2".into(),
                kind: RelationKind::Association,
                tool: None,
                label: Some("places".into()),
            }],
            preamble: None,
            legend: None,
            title: None,
        },
    );
    Project {
        name: "Untitled".into(),
        modified: false,
        root: ModelNode {
            id: "proj".into(),
            name: "Untitled".into(),
            kind: NodeKind::Project,
            children: vec![ModelNode {
                id: "model1".into(),
                name: "Model".into(),
                kind: NodeKind::Model,
                children: vec![ModelNode {
                    id: "diag1".into(),
                    name: "Main".into(),
                    kind: NodeKind::Diagram,
                    children: vec![class_node("c1", "Customer"), class_node("c2", "Order")],
                }],
            }],
        },
        diagrams,
        active_diagram_id: "diag1".into(),
    }
}
