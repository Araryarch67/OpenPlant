//! PlantUML text generation (positions round-trip as `' pos` comments).

use super::*;
use crate::model::{DiagramDoc, RelationKind, Shape, ShapeKind, Side};
use std::collections::HashMap;

pub(crate) fn gen_arrow(kind: RelationKind) -> &'static str {
    use RelationKind::*;
    match kind {
        Association | Link | FreeLine => "--",
        DirectedAssociation | Containment | Connector | DirectedLink => "-->",
        Aggregation => "o--",
        Composition => "*--",
        Generalization => "--|>",
        Dependency | TemplateBinding | RoleBinding => "..>",
        Realization => "..|>",
        NoteLink => "..",
    }
}

pub(crate) fn keyword_for_kind(kind: ShapeKind) -> &'static str {
    match kind {
        ShapeKind::Actor => "actor",
        ShapeKind::Database => "database",
        ShapeKind::Package => "package",
        ShapeKind::Note => "note",
        ShapeKind::Component => "component",
        ShapeKind::Interface => "interface",
        ShapeKind::Box => "rectangle",
        ShapeKind::Usecase => "usecase",
        ShapeKind::Class => "class",
    }
}

/// Most faithful keyword: the original tool when it is a native PlantUML
/// keyword, otherwise derived from the visual kind.
pub fn keyword_for(shape: &Shape) -> String {
    let t = shape.tool.as_deref().map(str::to_lowercase);
    match t.as_deref() {
        Some("enumeration") => "enum".into(),
        Some("abstract class") | Some("abstract") => "abstract class".into(),
        Some(t) if NATIVE_KEYWORDS.contains(&t) => t.into(),
        _ => keyword_for_kind(shape.kind()).into(),
    }
}

pub(crate) fn puml_safe(s: &str) -> String {
    let t: String = s
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if t.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        t
    } else {
        let mut h: i32 = 0;
        for c in s.encode_utf16() {
            h = h.wrapping_mul(31).wrapping_add(c as i32);
        }
        format!("e_{}", h.unsigned_abs())
    }
}

pub(crate) fn decl(sh: &Shape, alias: &str) -> String {
    let kw = keyword_for(sh);
    let flat = sh.label.replace('\n', "\\n").replace('"', "'");
    let stereo = match &sh.tool {
        Some(t) if t.to_lowercase() != kw => format!(" <<{t}>>"),
        _ => String::new(),
    };
    format!("{kw} \"{flat}\" as {alias}{stereo}")
}

pub fn describe_shape(sh: &Shape) -> String {
    decl(sh, &puml_safe(sh.label.split('\n').next().unwrap_or("e")))
}

pub fn generate_plantuml(d: &DiagramDoc) -> String {
    let mut lines = vec!["@startuml".to_string(), format!("title {}", d.name)];
    if let Some(p) = &d.preamble {
        lines.extend(p.iter().cloned());
    }
    let ids: Vec<&str> = d.shapes.iter().map(|s| s.id.as_str()).collect();
    let mut alias_of: HashMap<&str, String> = HashMap::new();
    let mut used = std::collections::HashSet::new();
    for s in &d.shapes {
        let first = s
            .label
            .split('\n')
            .next()
            .filter(|l| !l.is_empty())
            .unwrap_or(&s.id);
        let mut alias = puml_safe(first);
        if used.contains(&alias) {
            let suffix: String = s.id.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            alias = format!("{alias}_{suffix}");
        }
        used.insert(alias.clone());
        alias_of.insert(&s.id, alias);
    }
    fn parent<'a>(s: &'a Shape, ids: &[&str]) -> Option<&'a str> {
        s.parent_id.as_deref().filter(|p| ids.contains(p))
    }
    fn emit(
        pid: Option<&str>,
        d: &DiagramDoc,
        alias_of: &HashMap<&str, String>,
        ids: &[&str],
        lines: &mut Vec<String>,
    ) {
        for s in d.shapes.iter().filter(|s| parent(s, ids) == pid) {
            let alias = &alias_of[s.id.as_str()];
            if s.kind() == ShapeKind::Note
                && let Some(of) = s.note_of.as_deref().filter(|o| ids.contains(o))
            {
                let target = alias_of.get(of).cloned().unwrap_or(of.to_string());
                let side = s.note_side.unwrap_or(Side::Right).as_str();
                lines.push(format!("note {side} of {target}"));
                for ln in s.label.split('\n') {
                    lines.push(if ln.is_empty() {
                        String::new()
                    } else {
                        format!("  {ln}")
                    });
                }
                lines.push("end note".into());
                continue;
            }
            let has_kids = d
                .shapes
                .iter()
                .any(|k| parent(k, ids) == Some(s.id.as_str()));
            if has_kids {
                lines.push(format!("{} {{", decl(s, alias)));
                emit(Some(&s.id), d, alias_of, ids, lines);
                lines.push("}".into());
            } else {
                lines.push(decl(s, alias));
            }
            let tool = s
                .tool
                .as_deref()
                .map(|t| format!(" tool={t}"))
                .unwrap_or_default();
            lines.push(format!(
                "' pos {alias}: {},{}{tool}",
                s.x.round(),
                s.y.round()
            ));
        }
    }
    emit(None, d, &alias_of, &ids, &mut lines);
    for r in &d.relations {
        let from = alias_of
            .get(r.from.as_str())
            .cloned()
            .unwrap_or(r.from.clone());
        let to = alias_of.get(r.to.as_str()).cloned().unwrap_or(r.to.clone());
        let suffix = r
            .label
            .as_deref()
            .map(|l| format!(" : {l}"))
            .unwrap_or_default();
        lines.push(format!("{from} {} {to}{suffix}", gen_arrow(r.kind)));
    }
    if let Some(l) = &d.legend {
        lines.extend(l.iter().cloned());
    }
    lines.push("@enduml".into());
    lines.join("\n") + "\n"
}
