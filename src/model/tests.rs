//! Model serialisation tests.

use super::*;

#[test]
fn project_json_round_trip() {
    let p = default_project();
    let json = serde_json::to_string_pretty(&p).unwrap();
    assert!(json.contains("\"activeDiagramId\""));
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back, p);
}

#[test]
fn opens_web_build_project_json() {
    // Shape written by the old React build (kebab-case relation kinds,
    // optional fields missing).
    let json = r##"{"name":"Untitled","modified":true,"root":{"id":"proj","name":"Untitled","kind":"project","children":[]},
      "diagrams":{"diag1":{"id":"diag1","name":"Main","kind":"Class",
        "shapes":[{"id":"c1","label":"A","x":1,"y":2,"w":160,"h":96,"kind":"box","tool":"Queue","style":{"fill":"#ff0000","fontSize":16,"dash":"dashed"}}],
        "relations":[{"id":"r1","from":"c1","to":"c1","kind":"directed-association"}]}},
      "activeDiagramId":"diag1"}"##;
    let p: Project = serde_json::from_str(json).unwrap();
    let s = &p.active().shapes[0];
    assert_eq!(s.kind(), ShapeKind::Box);
    assert_eq!(s.style.as_ref().unwrap().dash, Some(Dash::Dashed));
    assert_eq!(
        p.active().relations[0].kind,
        RelationKind::DirectedAssociation
    );
}

#[test]
fn colors_parse() {
    assert_eq!(parse_color("#fff"), Some(0xffffff));
    assert_eq!(parse_color("#1d2529"), Some(0x1d2529));
    assert_eq!(parse_color("#11223344"), Some(0x112233));
    assert_eq!(parse_color("white"), Some(0xffffff));
    assert_eq!(parse_color("nope"), None);
    assert_eq!(color_hex(0x0a0b0c), "#0a0b0c");
}
