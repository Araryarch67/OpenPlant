//! Line-by-line parser: declarations, nesting, notes, relations.

use super::*;
use crate::model::{Relation, Shape, ShapeKind, Side, nid};
use std::collections::HashMap;

pub(crate) struct Parser {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) relations: Vec<Relation>,
    pub(crate) index: HashMap<String, String>,
    pub(crate) col: usize,
    pub(crate) pkg_stack: Vec<String>,
    pub(crate) parent_of: HashMap<String, String>,
    pub(crate) children_of: HashMap<String, Vec<String>>,
    pub(crate) note_targets: Vec<(String, Side, String)>,
}

impl Parser {
    fn ensure(
        &mut self,
        display: &str,
        kind: ShapeKind,
        alias: Option<&str>,
        tool: &str,
    ) -> String {
        let key = alias.unwrap_or(display).to_string();
        if let Some(id) = self.index.get(&key) {
            return id.clone();
        }
        let id = nid("p");
        self.index.insert(key, id.clone());
        let label = display.replace("\\n", "\n");
        let lines = label.split('\n').count() as f32;
        let w = 170.0;
        let h = match kind {
            ShapeKind::Actor => 140.0,
            ShapeKind::Note => (44.0 + lines * 16.0).max(96.0),
            _ => 96.0,
        };
        let x = 80.0 + (self.col % 4) as f32 * 240.0;
        let y = 80.0 + (self.col / 4) as f32 * 200.0;
        self.col += 1;
        self.shapes.push(Shape {
            id: id.clone(),
            label,
            x,
            y,
            w,
            h,
            kind: Some(kind),
            tool: Some(tool.to_string()),
            parent_id: None,
            note_side: None,
            note_of: None,
            style: None,
        });
        id
    }

    fn track_parent(&mut self, id: &str) {
        if let Some(top) = self.pkg_stack.last().cloned()
            && !self.parent_of.contains_key(id)
        {
            self.parent_of.insert(id.to_string(), top.clone());
            self.children_of
                .entry(top)
                .or_default()
                .push(id.to_string());
        }
    }
}

pub fn parse_plantuml(text: &str) -> ParsedDiagram {
    let mut p = Parser {
        shapes: vec![],
        relations: vec![],
        index: HashMap::new(),
        col: 0,
        pkg_stack: vec![],
        parent_of: HashMap::new(),
        children_of: HashMap::new(),
        note_targets: vec![],
    };
    let mut title: Option<String> = None;
    let mut title_text: Option<String> = None;
    let mut pos_map: HashMap<String, (f32, f32, Option<String>)> = HashMap::new();
    let mut preamble: Vec<String> = vec![];
    let mut legend: Vec<String> = vec![];
    let mut pkg_blocks_seen = false;

    let lines: Vec<&str> = text.split('\n').collect();
    // (closing line, keep into preamble?)
    let mut skip: Option<(&str, bool)> = None;
    let mut idx = 0;
    while idx < lines.len() {
        let raw = lines[idx];
        let line = raw.trim();
        idx += 1;
        if line.is_empty() {
            continue;
        }
        if let Some((close, to_preamble)) = skip {
            let done = line == close || line == "endlegend";
            let keep = if done {
                line.to_string()
            } else {
                raw.trim_end().to_string()
            };
            if to_preamble {
                preamble.push(keep)
            } else {
                legend.push(keep)
            }
            if done {
                skip = None;
            }
            continue;
        }
        if SKIN_BLOCK.is_match(line) {
            skip = Some(("}", true));
            preamble.push(line.to_string());
            continue;
        }
        if LEGEND_START.is_match(line) {
            skip = Some(("endlegend", false));
            legend.push(line.to_string());
            continue;
        }
        if line.starts_with('\'') {
            if let Some(c) = POS_COMMENT.captures(line) {
                pos_map.insert(
                    c[1].to_string(),
                    (
                        c[2].parse().unwrap_or(0.0),
                        c[3].parse().unwrap_or(0.0),
                        c.get(4)
                            .map(|m| m.as_str().trim().to_string())
                            .filter(|s| !s.is_empty()),
                    ),
                );
            }
            continue;
        }
        if line == "@enduml" {
            continue;
        }
        if line == "}" {
            p.pkg_stack.pop();
            continue;
        }
        if let Some(rest) = line.strip_prefix("@startuml") {
            let rest = rest.trim();
            if !rest.is_empty() && title.is_none() {
                title = Some(rest.replace('_', " "));
            }
            continue;
        }
        if let Some(c) = TITLE.captures(line) {
            // Explicit title wins over the @startuml name (last wins, like PlantUML).
            title = Some(c[1].replace("\\n", " ").trim().to_string());
            title_text = Some(c[1].replace("\\n", "\n").trim().to_string());
            continue;
        }
        if PREAMBLE.is_match(line) {
            preamble.push(line.to_string());
            continue;
        }

        if let Some(c) = NOTE_START.captures(line) {
            let mut body = vec![];
            while idx < lines.len() && lines[idx].trim() != "end note" {
                body.push(lines[idx].trim().to_string());
                idx += 1;
            }
            idx += 1;
            let label = body.join("\n");
            if !label.is_empty() {
                let key = format!("__note{}", p.shapes.len());
                let id = p.ensure(&label, ShapeKind::Note, Some(&key), "Note");
                let side = match &c[1] {
                    "left" => Side::Left,
                    "top" => Side::Top,
                    "bottom" => Side::Bottom,
                    _ => Side::Right,
                };
                p.note_targets.push((id.clone(), side, c[2].to_string()));
                p.track_parent(&id);
            }
            continue;
        }

        if let Some(c) = ABSTRACT.captures(line) {
            if let Some((name, alias)) = parse_decl_rest(&c[1]) {
                let key = alias.clone().unwrap_or(name.clone());
                if !p.index.contains_key(&key) {
                    let id = p.ensure(&name, ShapeKind::Class, alias.as_deref(), "Abstract Class");
                    p.track_parent(&id);
                }
            }
            continue;
        }

        if let Some(c) = IFACE_SHORT.captures(line) {
            if let Some((name, alias)) = parse_decl_rest(&c[1]) {
                let key = alias.clone().unwrap_or(name.clone());
                if !p.index.contains_key(&key) {
                    let id = p.ensure(&name, ShapeKind::Interface, alias.as_deref(), "Interface");
                    p.track_parent(&id);
                }
            }
            continue;
        }

        if let Some(c) = SHORT_DECL.captures(line)
            && !c[1].contains(',')
        {
            let wrap = &c[1];
            let inner = wrap[1..wrap.len() - 1].trim().to_string();
            let (kind, tool) = match wrap.as_bytes()[0] {
                b'(' => (ShapeKind::Usecase, "Usecase"),
                b':' => (ShapeKind::Actor, "Actor"),
                _ => (ShapeKind::Component, "Component"),
            };
            let alias = strip_alias(c.get(2));
            let key = alias.clone().unwrap_or(inner.clone());
            if !inner.is_empty() && !p.index.contains_key(&key) {
                let id = p.ensure(&inner, kind, alias.as_deref(), tool);
                p.track_parent(&id);
            }
            continue;
        }

        if let Some(c) = DECL_RE.captures(line) {
            let kw = c[1].to_string();
            let (kind, base_tool) = keyword(&kw).expect("DECL_RE keywords are mapped");
            let mut rest = c[2].to_string();
            if kw == "note"
                && let Some(s) = NOTE_SINGLE.captures(&rest)
            {
                let key = format!("__note{}", p.shapes.len());
                let id = p.ensure(s[2].trim(), ShapeKind::Note, Some(&key), "Note");
                p.track_parent(&id);
                continue;
            }
            let stereo = STEREO.captures(&rest).map(|s| s[1].trim().to_string());
            rest = STEREO.replace_all(&rest, "").trim().to_string();
            if let Some((name, alias)) = parse_decl_rest(&rest) {
                let tool = stereo.unwrap_or_else(|| base_tool.to_string());
                let nestable = matches!(
                    kind,
                    ShapeKind::Package
                        | ShapeKind::Box
                        | ShapeKind::Component
                        | ShapeKind::Database
                );
                let opens = OPEN_BRACE_END.is_match(line) && nestable;
                let key = alias.clone().unwrap_or(name.clone());
                let is_new = !p.index.contains_key(&key);
                let id = p.ensure(&name, kind, alias.as_deref(), &tool);
                if is_new {
                    p.track_parent(&id);
                    if opens {
                        pkg_blocks_seen = true;
                        p.children_of.entry(id.clone()).or_default();
                        p.pkg_stack.push(id);
                    }
                }
            }
            continue;
        }

        for arrow in REL_ARROWS {
            let Some(at) = line.find(arrow) else { continue };
            if at == 0 {
                continue;
            }
            let raw_left = &line[..at];
            let left = strip_wrapper(&strip_quotes(raw_left));
            let mut right_part = line[at + arrow.len()..].trim().to_string();
            let mut label = None;
            if let Some(colon) = right_part.find(':') {
                let l = right_part[colon + 1..].trim().to_string();
                label = (!l.is_empty()).then_some(l);
                right_part = right_part[..colon].trim().to_string();
            }
            let raw_right = right_part.clone();
            let right = strip_wrapper(&strip_quotes(&raw_right));
            if left.is_empty() || right.is_empty() {
                continue;
            }
            // Plain tokens must be one word; quoted/wrapped ones may hold spaces.
            if raw_left.trim() == left && left.contains(char::is_whitespace) {
                continue;
            }
            if raw_right.trim() == right && right.contains(char::is_whitespace) {
                continue;
            }
            let (gf, tf) = guess(raw_left);
            let (gt, tt) = guess(&raw_right);
            let from = match p.index.get(&left) {
                Some(id) => id.clone(),
                None => p.ensure(&left, gf, None, tf),
            };
            let to = match p.index.get(&right) {
                Some(id) => id.clone(),
                None => p.ensure(&right, gt, None, tt),
            };
            let (ff, tt) = if arrow.starts_with('<') {
                (to, from)
            } else {
                (from, to)
            };
            p.relations.push(Relation {
                id: nid("r"),
                from: ff,
                to: tt,
                kind: arrow_kind(arrow),
                tool: None,
                label,
            });
            break;
        }
    }

    if pkg_blocks_seen {
        layout_grouped(&mut p);
    }
    let rev: HashMap<String, String> = p
        .index
        .iter()
        .map(|(k, v)| (v.clone(), k.clone()))
        .collect();
    for sh in &mut p.shapes {
        if let Some(pid) = p.parent_of.get(&sh.id) {
            sh.parent_id = Some(pid.clone());
        }
        let saved = rev
            .get(&sh.id)
            .and_then(|k| pos_map.get(k))
            .or_else(|| pos_map.get(&sh.label));
        if let Some((x, y, tool)) = saved {
            sh.x = x.round();
            sh.y = y.round();
            if tool.is_some() && sh.tool.is_none() {
                sh.tool = tool.clone();
            }
        }
    }
    ParsedDiagram {
        shapes: p.shapes,
        relations: p.relations,
        title,
        title_text,
        preamble,
        legend,
    }
}
