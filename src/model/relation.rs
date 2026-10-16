//! Relations between shapes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationKind {
    Association,
    DirectedAssociation,
    Aggregation,
    Composition,
    Generalization,
    Dependency,
    Realization,
    TemplateBinding,
    Containment,
    Connector,
    Link,
    DirectedLink,
    NoteLink,
    FreeLine,
    RoleBinding,
}

impl RelationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Association => "association",
            Self::DirectedAssociation => "directed-association",
            Self::Aggregation => "aggregation",
            Self::Composition => "composition",
            Self::Generalization => "generalization",
            Self::Dependency => "dependency",
            Self::Realization => "realization",
            Self::TemplateBinding => "template-binding",
            Self::Containment => "containment",
            Self::Connector => "connector",
            Self::Link => "link",
            Self::DirectedLink => "directed-link",
            Self::NoteLink => "note-link",
            Self::FreeLine => "free-line",
            Self::RoleBinding => "role-binding",
        }
    }

    pub fn is_dashed(self) -> bool {
        matches!(
            self,
            Self::Dependency
                | Self::Realization
                | Self::TemplateBinding
                | Self::RoleBinding
                | Self::NoteLink
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: RelationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
