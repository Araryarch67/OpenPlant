//! Tokens of the supported PlantUML subset: arrows, keywords, line patterns.

use crate::model::{RelationKind, ShapeKind};
use regex::Regex;
use std::sync::LazyLock;

pub(crate) const REL_ARROWS: &[&str] = &[
    "<|--", "--|>", "<|..", "..|>", "<--", "-->", "<-", "->", "o--", "--o", "*--", "--*", "<..",
    "..>", "--", "..", ".>",
];

pub(crate) fn arrow_kind(arrow: &str) -> RelationKind {
    if arrow == ".>" {
        return RelationKind::Dependency;
    }
    if arrow.contains("o--") || arrow.contains("--o") {
        return RelationKind::Aggregation;
    }
    if arrow.contains("*--") || arrow.contains("--*") {
        return RelationKind::Composition;
    }
    if arrow.contains("|>") || arrow.contains("<|") {
        return if arrow.contains("..") {
            RelationKind::Realization
        } else {
            RelationKind::Generalization
        };
    }
    if arrow.contains("..") {
        return RelationKind::Dependency;
    }
    if arrow == "--" {
        return RelationKind::Association;
    }
    RelationKind::DirectedAssociation
}

pub(crate) fn keyword(kw: &str) -> Option<(ShapeKind, &'static str)> {
    use ShapeKind::*;
    Some(match kw {
        "actor" => (Actor, "Actor"),
        "person" => (Actor, "Person"),
        "agent" => (Actor, "Agent"),
        "class" => (Class, "Class"),
        "interface" => (Interface, "Interface"),
        "enum" => (Class, "Enumeration"),
        "annotation" => (Class, "Annotation"),
        "database" => (Database, "Database"),
        "package" => (Package, "Package"),
        "folder" => (Package, "Folder"),
        "frame" => (Package, "Frame"),
        "note" => (Note, "Note"),
        "component" => (Component, "Component"),
        "rectangle" => (Box, "Rectangle"),
        "queue" => (Box, "Queue"),
        "cloud" => (Box, "Cloud"),
        "card" => (Box, "Card"),
        "file" => (Box, "File"),
        "storage" => (Box, "Storage"),
        "stack" => (Box, "Stack"),
        "object" => (Box, "Object"),
        "node" => (Box, "Node"),
        "artifact" => (Box, "Artifact"),
        "entity" => (Box, "Entity"),
        "collections" => (Box, "Collections"),
        "hexagon" => (Box, "Hexagon"),
        "boundary" => (Box, "Boundary"),
        "control" => (Box, "Control"),
        "usecase" => (Usecase, "Usecase"),
        _ => return None,
    })
}

pub(crate) const NATIVE_KEYWORDS: &[&str] = &[
    "actor",
    "agent",
    "annotation",
    "artifact",
    "boundary",
    "card",
    "class",
    "cloud",
    "collections",
    "component",
    "control",
    "database",
    "entity",
    "enum",
    "file",
    "folder",
    "frame",
    "hexagon",
    "interface",
    "node",
    "note",
    "object",
    "package",
    "person",
    "queue",
    "rectangle",
    "stack",
    "storage",
    "usecase",
];

macro_rules! re {
    ($name:ident, $pat:expr) => {
        pub(crate) static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pat).unwrap());
    };
}

re!(
    DECL_RE,
    r"^(actor|agent|annotation|artifact|boundary|card|class|cloud|collections|component|control|database|entity|enum|file|folder|frame|hexagon|interface|node|note|object|package|person|queue|rectangle|stack|storage|usecase)\b\s*(.*)$"
);

re!(QUOTED, r#"^"([^"]+)"(?:\s+as\s+(\S+))?$"#);

re!(COLON_D, r"^:([^:]+):(?:\s+as\s+(\S+))?$");

re!(PAREN_D, r"^\(([^()]+)\)(?:\s+as\s+(\S+))?$");

re!(BARE, r"^(\S+)(?:\s+as\s+(\S+))?$");

re!(SKIN_BLOCK, r"^skinparam\s+\w+\s*\{\s*$");

re!(LEGEND_START, r"^legend(?:\s+\w+)?\s*$");

re!(
    POS_COMMENT,
    r"^'\s*(?:@pos\s+|pos\s+)(\S+)\s*:\s*(-?\d+)\s*,\s*(-?\d+)(?:\s+tool=(.+))?$"
);

re!(TITLE, r"^(?:title|caption)\s+(.+)$");

re!(
    PREAMBLE,
    r"^(skinparam|header|footer|hide|allow_mixing|left to right direction|top to bottom direction)\b"
);

re!(
    NOTE_START,
    r"^note\s+(right|left|top|bottom)\s+of\s+(\S+)\s*$"
);

re!(ABSTRACT, r"^abstract(?:\s+class)?\s+(.+)$");

re!(IFACE_SHORT, r"^\(\)\s+(.+)$");

re!(
    SHORT_DECL,
    r"^(\([^()]+\)|:[^:]+:|\[[^\[\]]+\])(?:\s+as\s+(\S+))?$"
);

re!(NOTE_SINGLE, r"^(left|right|top|bottom)\s*:\s*(.+)$");

re!(STEREO, r"<<\s*([^<>]+?)\s*>>");

re!(OPEN_BRACE_END, r"\{\s*$");

pub(crate) fn strip_quotes(tok: &str) -> String {
    let t = tok.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

/// Docs aliases may be parenthesised: `as (UC2)` -> `UC2`.
pub(crate) fn strip_alias(a: Option<regex::Match>) -> Option<String> {
    let a = a?.as_str().trim_start_matches('(').trim_end_matches(')');
    (!a.is_empty()).then(|| a.to_string())
}

pub(crate) fn parse_decl_rest(rest: &str) -> Option<(String, Option<String>)> {
    let r = OPEN_BRACE_END.replace(rest.trim(), "").trim().to_string();
    if r.is_empty() {
        return None;
    }
    for re in [&*QUOTED, &*COLON_D, &*PAREN_D, &*BARE] {
        if let Some(c) = re.captures(&r) {
            return Some((c[1].trim().to_string(), strip_alias(c.get(2))));
        }
    }
    None
}

/// Strip docs wrappers on relation operands: `:A:` `(UC)` `[C]` -> plain name.
pub(crate) fn strip_wrapper(tok: &str) -> String {
    let t = tok.trim();
    if t.len() >= 2 {
        let f = t.chars().next().unwrap();
        let l = t.chars().last().unwrap();
        if (f == ':' && l == ':') || (f == '(' && l == ')') || (f == '[' && l == ']') {
            return t[1..t.len() - 1].trim().to_string();
        }
    }
    t.to_string()
}

pub(crate) fn guess(raw: &str) -> (ShapeKind, &'static str) {
    let t = raw.trim();
    if t.starts_with('(') && t.ends_with(')') {
        (ShapeKind::Usecase, "Usecase")
    } else if t.starts_with(':') && t.ends_with(':') {
        (ShapeKind::Actor, "Actor")
    } else if t.starts_with('[') && t.ends_with(']') {
        (ShapeKind::Component, "Component")
    } else {
        (ShapeKind::Class, "Class")
    }
}
