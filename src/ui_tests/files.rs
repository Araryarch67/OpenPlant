//! Opening, importing and saving.

use super::*;
use crate::model::Project;
use gpui_kit::TestAppContext;

fn sample() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sample-puml/enterprise-library-management.puml")
}

#[gpui_kit::test]
fn open_plantuml_imports_and_fits(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.update(|a, w, cx| a.open_path(sample(), w, cx));
    assert_eq!(ui.doc(|d| d.shapes.len()), 72);
    assert!(
        ui.doc(|d| d.title.clone()).unwrap().contains('\n'),
        "multi-line title kept"
    );
    assert!(ui.read(|a| a.wb.zoom) < 1.0, "large diagram fitted");
    assert!(ui.read(|a| a.auto_fit.is_some()));
    assert_eq!(ui.read(|a| a.wb.toasts.len()), 1);
    ui.advance(5000);
    assert_eq!(ui.read(|a| a.wb.toasts.len()), 0, "toast dismisses itself");

    // Panning ends auto-fit; the explorer lists the imported model.
    let p = ui.at(200.0, 200.0);
    ui.wheel(p, 0.0, 30.0, Default::default());
    assert!(ui.read(|a| a.auto_fit.is_none()));
    assert!(
        ui.read(|a| a.wb.project.root.find("model1").unwrap().children[0]
            .children
            .len())
            > 5
    );
}

#[gpui_kit::test]
fn save_writes_project_and_reopens(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let dir = std::env::temp_dir().join(format!("openplant-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("project.json");
    ui.update(|a, _, _| {
        a.wb.add_shape("Note", 500.0, 80.0, Some("Remember".into()));
        a.wb.file_path = Some(path.clone());
    });
    ui.press("ctrl-s");
    let saved: Project = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved.active().shapes.len(), 3);
    assert!(
        !ui.read(|a| a.wb.project.modified),
        "save clears the modified flag"
    );

    ui.press("ctrl-n");
    assert_eq!(ui.doc(|d| d.shapes.len()), 2, "new project");
    ui.update(|a, w, cx| a.open_path(path.clone(), w, cx));
    assert_eq!(
        ui.doc(|d| d.shapes.iter().any(|s| s.label == "Remember")),
        true
    );
    assert_eq!(ui.read(|a| a.wb.file_path.clone()), Some(path.clone()));
    std::fs::remove_dir_all(dir).ok();
}

#[gpui_kit::test]
fn unreadable_file_reports_error(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.update(|a, w, cx| a.open_path("/nonexistent/x.puml".into(), w, cx));
    let t = ui.read(|a| a.wb.toasts.last().cloned()).unwrap();
    assert_eq!(t.kind, crate::workbench::ToastKind::Error);
    assert_eq!(ui.doc(|d| d.shapes.len()), 2, "project untouched");
}
