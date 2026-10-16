//! Workbench behaviour tests.

use super::*;

#[test]
fn undo_redo_add_shape() {
    let mut wb = Workbench::new();
    let before = wb.doc().shapes.len();
    wb.add_shape("Class", 10.0, 10.0, None);
    assert_eq!(wb.doc().shapes.len(), before + 1);
    assert!(wb.undo());
    assert_eq!(wb.doc().shapes.len(), before);
    assert!(wb.redo());
    assert_eq!(wb.doc().shapes.len(), before + 1);
}

#[test]
fn duplicate_keeps_internal_relations() {
    let mut wb = Workbench::new();
    wb.select_shapes(vec!["c1".into(), "c2".into()]);
    wb.duplicate_selection();
    assert_eq!(wb.doc().shapes.len(), 4);
    assert_eq!(wb.doc().relations.len(), 2);
}

#[test]
fn delete_relation_only() {
    let mut wb = Workbench::new();
    wb.select_relation("r1", false);
    wb.delete_selection();
    assert_eq!(wb.doc().relations.len(), 0);
    assert_eq!(wb.doc().shapes.len(), 2);
}

#[test]
fn cut_paste_and_undo_chain() {
    let mut wb = Workbench::new();
    wb.select_shapes(vec!["c1".into(), "c2".into()]);
    wb.cut_selection();
    assert!(wb.doc().shapes.is_empty());
    assert!(wb.doc().relations.is_empty());
    wb.paste();
    assert_eq!(wb.doc().shapes.len(), 2);
    assert_eq!(
        wb.doc().relations.len(),
        1,
        "internal relation is pasted too"
    );
    wb.undo();
    assert!(wb.doc().shapes.is_empty());
    wb.undo();
    assert_eq!(wb.doc().shapes.len(), 2);
    assert!(!wb.can_undo() || wb.doc().shapes.len() == 2);
}

#[test]
fn align_distribute_and_sizes() {
    let mut wb = Workbench::new();
    wb.add_shape("Class", 300.0, 500.0, None);
    wb.select_all();
    wb.align(AlignMode::Left);
    let xs: Vec<f32> = wb.doc().shapes.iter().map(|s| s.x).collect();
    assert!(xs.iter().all(|x| *x == xs[0]));
    wb.align(AlignMode::SameW);
    let w0 = wb.doc().shapes[0].w;
    assert!(wb.doc().shapes.iter().all(|s| s.w == w0));
    wb.align(AlignMode::DistV);
    let mut ys: Vec<(f32, f32)> = wb.doc().shapes.iter().map(|s| (s.y, s.h)).collect();
    ys.sort_by(|a, b| a.0.total_cmp(&b.0));
    let gaps: Vec<f32> = ys.windows(2).map(|w| w[1].0 - (w[0].0 + w[0].1)).collect();
    assert!((gaps[0] - gaps[1]).abs() < 1.5, "even gaps {gaps:?}");
}

#[test]
fn reorder_nudge_resize_size_copy() {
    let mut wb = Workbench::new();
    wb.select_shapes(vec!["c1".into()]);
    wb.reorder(true, false);
    assert_eq!(wb.doc().shapes.last().unwrap().id, "c1");
    wb.reorder(false, true);
    assert_eq!(wb.doc().shapes[0].id, "c1");
    let (x, y) = (
        wb.doc().shape("c1").unwrap().x,
        wb.doc().shape("c1").unwrap().y,
    );
    wb.nudge(16.0, -1.0);
    let s = wb.doc().shape("c1").unwrap();
    assert_eq!((s.x, s.y), (x + 16.0, y - 1.0));
    wb.resize_selected(10.0, 5.0);
    let s = wb.doc().shape("c1").unwrap().clone();
    wb.copy_size();
    wb.select_shapes(vec!["c2".into()]);
    wb.paste_size();
    let t = wb.doc().shape("c2").unwrap();
    assert_eq!((t.w, t.h), (s.w, s.h));
}

#[test]
fn diagrams_add_switch_delete() {
    let mut wb = Workbench::new();
    wb.add_diagram("Sequence");
    assert_eq!(wb.project.diagrams.len(), 2);
    let new_id = wb.project.active_diagram_id.clone();
    assert!(wb.doc().shapes.is_empty());
    wb.add_shape("Actor", 0.0, 0.0, None);
    assert!(wb.project.root.find(&new_id).unwrap().children.len() == 1);
    wb.activate_diagram("diag1");
    assert_eq!(wb.doc().shapes.len(), 2);
    wb.delete_diagram(&new_id);
    assert_eq!(wb.project.diagrams.len(), 1);
    wb.delete_diagram("diag1");
    assert_eq!(wb.project.diagrams.len(), 1, "last diagram is kept");
}

#[test]
fn rename_and_styles() {
    let mut wb = Workbench::new();
    wb.rename_node("c1", "Client");
    assert_eq!(wb.doc().shape("c1").unwrap().label, "Client");
    assert_eq!(wb.project.root.find("c1").unwrap().name, "Client");
    wb.select_shapes(vec!["c1".into()]);
    wb.set_style(|s| s.fill = Some("#ff0000".into()));
    assert_eq!(
        wb.doc()
            .shape("c1")
            .unwrap()
            .style
            .as_ref()
            .unwrap()
            .fill
            .as_deref(),
        Some("#ff0000")
    );
    wb.clear_style();
    assert!(wb.doc().shape("c1").unwrap().style.is_none());
    wb.undo();
    assert!(wb.doc().shape("c1").unwrap().style.is_some());
}

#[test]
fn container_moves_and_deletes_with_children() {
    let mut wb = Workbench::new();
    let p = crate::plantuml::parse_plantuml(
        "@startuml\npackage P {\n class A\n class B\n}\nclass C\nA --> C\n@enduml",
    );
    wb.replace_from_plantuml(p);
    let pkg = wb
        .doc()
        .shapes
        .iter()
        .find(|s| s.label == "P")
        .unwrap()
        .id
        .clone();
    let set = wb.drag_set(&[pkg.clone()]);
    assert_eq!(set.len(), 3);
    wb.select_shapes(vec![pkg]);
    wb.delete_selection();
    assert_eq!(wb.doc().shapes.len(), 1);
    assert!(wb.doc().relations.is_empty());
}

#[test]
fn tab_cycles_selection() {
    let mut wb = Workbench::new();
    wb.select_step(true);
    assert_eq!(wb.selected_shapes, vec!["c1".to_string()]);
    wb.select_step(true);
    assert_eq!(wb.selected_shapes, vec!["c2".to_string()]);
    wb.select_step(true);
    assert_eq!(wb.selected_shapes, vec!["c1".to_string()]);
    wb.select_step(false);
    assert_eq!(wb.selected_shapes, vec!["c2".to_string()]);
}
