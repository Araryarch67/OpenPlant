//! Diagram management and PlantUML import.

use super::*;
use crate::model::*;
use crate::plantuml::ParsedDiagram;

impl Workbench {
    pub fn activate_diagram(&mut self, id: &str) {
        if self.project.diagrams.contains_key(id) {
            self.project.active_diagram_id = id.to_string();
            self.select_none();
        }
    }

    pub fn add_diagram(&mut self, kind: &str) {
        self.checkpoint();
        let id = nid("diag");
        let n = self.project.diagrams.len() + 1;
        let name = if kind == "Class" {
            format!("Main {n}")
        } else {
            format!("{kind} {n}")
        };
        if let Some(model) = self.project.root.children.first_mut() {
            model.children.push(ModelNode {
                id: id.clone(),
                name: name.clone(),
                kind: NodeKind::Diagram,
                children: vec![],
            });
        }
        self.project.diagrams.insert(
            id.clone(),
            DiagramDoc {
                id: id.clone(),
                name,
                kind: kind.into(),
                shapes: vec![],
                relations: vec![],
                preamble: None,
                legend: None,
                title: None,
            },
        );
        self.project.active_diagram_id = id.clone();
        self.selected_node = Some(id);
        self.select_none();
        self.touch();
    }

    pub fn delete_diagram(&mut self, id: &str) {
        if self.project.diagrams.len() <= 1 || !self.project.diagrams.contains_key(id) {
            return;
        }
        self.checkpoint();
        self.project.diagrams.remove(id);
        self.project.root.drop_ids(&[id.to_string()]);
        if self.project.active_diagram_id == id {
            self.project.active_diagram_id = self.project.diagrams.keys().next().unwrap().clone();
        }
        self.selected_node = Some(self.project.active_diagram_id.clone());
        self.select_none();
        self.touch();
    }

    pub fn replace_from_plantuml(&mut self, parsed: ParsedDiagram) {
        self.checkpoint();
        let doc = self.project.active_mut();
        doc.shapes = parsed.shapes;
        doc.relations = parsed.relations;
        doc.preamble = Some(parsed.preamble);
        doc.legend = Some(parsed.legend);
        doc.title = parsed.title_text;
        let did = doc.id.clone();
        if let Some(t) = parsed.title
            && self.project.active().name != t
        {
            self.rename_node(&did, &t);
        }
        self.sync_nodes();
        self.select_none();
        self.touch();
    }
}
