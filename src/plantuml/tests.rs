//! Parser/generator tests (sample round-trip and syntax forms).

use super::*;
use crate::model::*;

#[test]
fn parses_sample_and_round_trips() {
    let src = include_str!("../../sample-puml/enterprise-library-management.puml");
    let p = parse_plantuml(src);
    assert!(p.shapes.len() > 50, "shapes: {}", p.shapes.len());
    assert!(p.relations.len() > 60, "relations: {}", p.relations.len());
    assert!(p.title_text.as_deref().unwrap().contains('\n'));
    let doc = DiagramDoc {
        id: "d".into(),
        name: p.title.clone().unwrap(),
        kind: "Class".into(),
        shapes: p.shapes,
        relations: p.relations,
        preamble: Some(p.preamble),
        legend: Some(p.legend),
        title: p.title_text,
    };
    let again = parse_plantuml(&generate_plantuml(&doc));
    assert_eq!(again.shapes.len(), doc.shapes.len());
    assert_eq!(again.relations.len(), doc.relations.len());
}

#[test]
fn simple_class_relation() {
    let p = parse_plantuml("@startuml\nclass Shop\nclass Item\nShop --> Item : stocks\n@enduml");
    assert_eq!(p.shapes.len(), 2);
    assert_eq!(p.relations[0].label.as_deref(), Some("stocks"));
    assert_eq!(p.relations[0].kind, RelationKind::DirectedAssociation);
}

#[test]
fn docs_short_forms_and_arrows() {
    let p = parse_plantuml(
        "@startuml\n:User: as U\n(Login) as L\n[API] as A\n() \"Port\" as P\nabstract class Base\nenum Color\nU --> L : uses\nA <|-- Base\nBase *-- Color\nA ..> P\nL .> A\n@enduml",
    );
    let kind = |l: &str| {
        p.shapes
            .iter()
            .find(|s| s.label == l)
            .map(|s| s.kind())
            .unwrap()
    };
    assert_eq!(kind("User"), ShapeKind::Actor);
    assert_eq!(kind("Login"), ShapeKind::Usecase);
    assert_eq!(kind("API"), ShapeKind::Component);
    assert_eq!(kind("Port"), ShapeKind::Interface);
    let kinds: Vec<RelationKind> = p.relations.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        vec![
            RelationKind::DirectedAssociation,
            RelationKind::Generalization,
            RelationKind::Composition,
            RelationKind::Dependency,
            RelationKind::Dependency
        ]
    );
    // `A <|-- Base` points from Base to A.
    let g = &p.relations[1];
    let label = |id: &str| p.shapes.iter().find(|s| s.id == id).unwrap().label.clone();
    assert_eq!(
        (label(&g.from), label(&g.to)),
        ("Base".into(), "API".into())
    );
}

#[test]
fn notes_attach_and_positions_round_trip() {
    let src = "@startuml\npackage G {\n rectangle \"R\" as R\n}\nnote right of R\n  hello\nend note\n' pos R: 400,300\n@enduml";
    let p = parse_plantuml(src);
    let r = p.shapes.iter().find(|s| s.label == "R").unwrap();
    assert_eq!((r.x, r.y), (400.0, 300.0));
    let n = p
        .shapes
        .iter()
        .find(|s| s.kind() == ShapeKind::Note)
        .unwrap();
    assert_eq!(n.note_of.as_deref(), Some(r.id.as_str()));
    assert_eq!(n.label, "hello");
}
