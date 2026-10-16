//! Toolbox catalogue: tool groups, diagram kinds, default sizes.

use super::{RelationKind, ShapeKind};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Node,
    Edge,
}

pub struct ToolDef {
    pub label: &'static str,
    pub kind: ToolKind,
}

pub struct ToolGroup {
    pub name: &'static str,
    pub tools: &'static [ToolDef],
}

const fn n(label: &'static str) -> ToolDef {
    ToolDef {
        label,
        kind: ToolKind::Node,
    }
}
const fn e(label: &'static str) -> ToolDef {
    ToolDef {
        label,
        kind: ToolKind::Edge,
    }
}

pub const TOOL_GROUPS: &[ToolGroup] = &[
    ToolGroup {
        name: "Class",
        tools: &[
            n("Class"),
            n("Interface"),
            n("Abstract Class"),
            n("Enumeration"),
            n("Annotation"),
            n("Object"),
            e("Association"),
            e("Directed Association"),
            e("Aggregation"),
            e("Composition"),
            e("Dependency"),
            e("Generalization"),
            e("Realization"),
        ],
    },
    ToolGroup {
        name: "Use Case",
        tools: &[
            n("Actor"),
            n("Usecase"),
            e("Association"),
            e("Generalization"),
            e("Dependency"),
        ],
    },
    ToolGroup {
        name: "Container",
        tools: &[
            n("Package"),
            n("Folder"),
            n("Frame"),
            n("Rectangle"),
            n("Queue"),
            n("Cloud"),
            e("Dependency"),
        ],
    },
    ToolGroup {
        name: "Deployment",
        tools: &[
            n("Node"),
            n("Artifact"),
            n("Component"),
            n("Database"),
            n("Storage"),
            e("Dependency"),
        ],
    },
    ToolGroup {
        name: "Note",
        tools: &[n("Note"), e("Note Link")],
    },
];

pub const DIAGRAM_KINDS: &[&str] = &[
    "Class Diagram",
    "Package Diagram",
    "Object Diagram",
    "Composite Structure Diagram",
    "Component Diagram",
    "Deployment Diagram",
    "Use Case Diagram",
    "Sequence Diagram",
    "Communication Diagram",
    "Timing Diagram",
    "Interaction Overview Diagram",
    "Statechart Diagram",
    "Activity Diagram",
    "Profile Diagram",
    "ER Diagram",
    "Flowchart Diagram",
    "Mindmap Diagram",
];

pub fn edge_kind(label: &str) -> Option<RelationKind> {
    Some(match label {
        "Association" => RelationKind::Association,
        "Directed Association" => RelationKind::DirectedAssociation,
        "Aggregation" => RelationKind::Aggregation,
        "Composition" => RelationKind::Composition,
        "Dependency" => RelationKind::Dependency,
        "Generalization" => RelationKind::Generalization,
        "Realization" => RelationKind::Realization,
        "Note Link" => RelationKind::NoteLink,
        _ => return None,
    })
}

pub fn is_edge_tool(label: &str) -> bool {
    edge_kind(label).is_some()
}

pub fn shape_kind(label: &str) -> ShapeKind {
    match label {
        "Interface" => ShapeKind::Interface,
        "Object" | "Rectangle" | "Queue" | "Cloud" | "Node" | "Artifact" => ShapeKind::Box,
        "Actor" => ShapeKind::Actor,
        "Usecase" => ShapeKind::Usecase,
        "Package" | "Folder" | "Frame" => ShapeKind::Package,
        "Component" => ShapeKind::Component,
        "Database" | "Storage" => ShapeKind::Database,
        "Note" => ShapeKind::Note,
        _ => ShapeKind::Class,
    }
}

pub fn default_size(kind: ShapeKind) -> (f32, f32) {
    match kind {
        ShapeKind::Class | ShapeKind::Interface => (160.0, 96.0),
        ShapeKind::Actor => (120.0, 130.0),
        ShapeKind::Database => (160.0, 110.0),
        ShapeKind::Package => (180.0, 110.0),
        ShapeKind::Note => (160.0, 96.0),
        ShapeKind::Component => (170.0, 110.0),
        ShapeKind::Box => (180.0, 72.0),
        ShapeKind::Usecase => (170.0, 64.0),
    }
}

pub fn default_label(tool: &str, n: usize) -> String {
    let base: String = tool
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let ok = base.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && base.chars().all(|c| c.is_ascii_alphanumeric());
    format!("{}{n}", if ok { base.as_str() } else { "Element" })
}
