//! Menu bar, command palette, shortcut sheet and canvas context menu.

use super::*;
use gpui_kit::{MouseButton, TestAppContext};

#[gpui_kit::test]
fn menu_opens_runs_command_and_closes(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(("menu", 0usize));
    assert!(ui.exists(id("mi-0-0")), "File > New is listed");
    ui.click("backdrop");
    assert!(!ui.exists(id("mi-0-0")), "clicking outside closes the menu");

    // View > Grid (index counts separators).
    assert!(ui.read(|a| a.wb.show_grid));
    ui.click(("menu", 5usize));
    ui.click(id("mi-5-3"));
    assert!(!ui.read(|a| a.wb.show_grid), "View > Grid toggles the grid");
    assert!(!ui.exists(id("mi-5-3")), "running an item closes the menu");

    // Edit > Undo is reachable from the menu.
    ui.click(("menu", 1usize));
    ui.click(id("mi-1-0"));
    assert_eq!(
        ui.read(|a| a.wb.toasts.len()),
        1,
        "nothing to undo -> info toast"
    );
}

#[gpui_kit::test]
fn palette_filters_and_runs_with_keyboard(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.press("f1");
    assert!(ui.exists("palette"));
    ui.type_text("zoom in");
    assert!(ui.exists(("pal", 0usize)));
    assert!(!ui.exists(("pal", 1usize)), "only one match for 'zoom in'");
    ui.press("enter");
    assert!(!ui.exists("palette"));
    assert!((ui.read(|a| a.wb.zoom) - 1.1).abs() < 1e-3);

    ui.click("palette-btn");
    assert!(ui.exists("palette"));
    ui.press("down down escape");
    assert!(!ui.exists("palette"), "escape closes the palette");
}

#[gpui_kit::test]
fn palette_click_runs_entry(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.press("f1");
    ui.type_text("add class");
    let before = ui.doc(|d| d.shapes.len());
    ui.click(("pal", 0usize));
    assert_eq!(ui.doc(|d| d.shapes.len()), before + 1);
}

#[gpui_kit::test]
fn shortcut_sheet_toggles(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.press("ctrl-/");
    assert!(ui.exists("shortcuts"));
    ui.press("escape");
    assert!(!ui.exists("shortcuts"));
    ui.click(("menu", 7usize));
    ui.click(id("mi-7-0"));
    assert!(ui.exists("shortcuts"), "Help > Keyboard Shortcuts");
    ui.click(id("shortcuts-x"));
    assert!(!ui.exists("shortcuts"));
}

#[gpui_kit::test]
fn context_menu_adds_shape_and_diagram(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let blank = ui.at(700.0, 500.0);
    ui.click_at(blank, MouseButton::Right, Default::default());
    assert!(ui.exists("cm-add"), "right click opens the context menu");
    ui.hover("cm-add");
    ui.click(("cm-add-item", 3usize)); // Rectangle
    let added = ui.doc(|d| d.shapes.last().cloned()).unwrap();
    assert_eq!(added.tool.as_deref(), Some("Rectangle"));
    assert!(
        (added.x + added.w / 2.0 - 700.0).abs() < 2.0,
        "placed at the click point"
    );
    assert!(!ui.exists("cm-add"));

    ui.click_at(blank, MouseButton::Right, Default::default());
    ui.hover("cm-add-diagram");
    ui.click(("cm-diag-item", 6usize)); // Use Case Diagram
    assert_eq!(ui.read(|a| a.wb.project.diagrams.len()), 2);
    assert_eq!(ui.doc(|d| d.kind.clone()), "Use Case");

    // Right click on a shape selects it, and Delete from the menu removes it.
    ui.update(|a, _, _| a.wb.activate_diagram("diag1"));
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Right, Default::default());
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c1".to_string()]
    );
    ui.click("cm-del");
    assert!(ui.doc(|d| d.shape("c1").is_none()));
}

#[gpui_kit::test]
fn double_click_blank_opens_quick_insert(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let blank = ui.at(800.0, 450.0);
    ui.double_click_at(blank);
    assert!(
        ui.exists(("cm-add-item", 0usize)),
        "Add submenu shown directly"
    );
    ui.click(("cm-add-item", 8usize)); // Note
    assert_eq!(
        ui.doc(|d| d.shapes.last().unwrap().tool.clone()).as_deref(),
        Some("Note")
    );
}
