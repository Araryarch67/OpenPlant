//! draw.io keyboard shortcuts on the canvas.

use super::*;
use gpui_kit::{MouseButton, TestAppContext};

fn select_c1(ui: &mut Ui) {
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Left, Default::default());
}

#[gpui_kit::test]
fn duplicate_delete_and_selection_keys(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    select_c1(&mut ui);
    ui.press("ctrl-d");
    assert_eq!(ui.doc(|d| d.shapes.len()), 3);
    let dup = ui.read(|a| a.wb.selected_shapes.clone());
    assert_eq!(dup.len(), 1);
    assert_ne!(dup[0], "c1", "the copy is selected");
    ui.press("delete");
    assert_eq!(ui.doc(|d| d.shapes.len()), 2);
    ui.press("ctrl-a");
    assert_eq!(
        ui.read(|a| (a.wb.selected_shapes.len(), a.wb.selected_relations.len())),
        (2, 1)
    );
    ui.press("ctrl-shift-a");
    assert!(!ui.read(|a| a.wb.has_selection()));
    ui.press("ctrl-shift-i");
    assert_eq!(
        ui.read(|a| (a.wb.selected_shapes.len(), a.wb.selected_relations.len())),
        (2, 0)
    );
    ui.press("ctrl-shift-e");
    assert_eq!(
        ui.read(|a| (a.wb.selected_shapes.len(), a.wb.selected_relations.len())),
        (0, 1)
    );
    ui.press("escape tab");
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c1".to_string()]
    );
    ui.press("tab");
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c2".to_string()]
    );
    ui.press("shift-tab");
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c1".to_string()]
    );
    ui.press("ctrl-enter");
    assert_eq!(
        ui.doc(|d| d.shapes.len()),
        3,
        "ctrl+enter duplicates (draw.io)"
    );
    ui.press("backspace");
    assert_eq!(ui.doc(|d| d.shapes.len()), 2, "backspace deletes");
}

#[gpui_kit::test]
fn clipboard_keys(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.press("ctrl-a ctrl-c ctrl-v");
    assert_eq!(
        ui.doc(|d| (d.shapes.len(), d.relations.len())),
        (4, 2),
        "paste keeps the relation"
    );
    ui.press("ctrl-x");
    assert_eq!(ui.doc(|d| (d.shapes.len(), d.relations.len())), (2, 1));
    ui.press("ctrl-v");
    assert_eq!(ui.doc(|d| d.shapes.len()), 4);
    ui.press("ctrl-z ctrl-z ctrl-z");
    assert_eq!(
        ui.doc(|d| d.shapes.len()),
        2,
        "undo back past cut and paste"
    );
}

#[gpui_kit::test]
fn arrows_nudge_and_resize(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    select_c1(&mut ui);
    ui.press("right down");
    assert_eq!((ui.shape("c1").x, ui.shape("c1").y), (121.0, 101.0));
    ui.press("shift-left shift-up");
    assert_eq!((ui.shape("c1").x, ui.shape("c1").y), (105.0, 85.0));
    ui.press("ctrl-right ctrl-down");
    assert_eq!((ui.shape("c1").w, ui.shape("c1").h), (161.0, 97.0));
    ui.press("ctrl-shift-left ctrl-shift-up");
    assert_eq!((ui.shape("c1").w, ui.shape("c1").h), (145.0, 81.0));
    ui.press("ctrl-z");
    assert_eq!(ui.shape("c1").h, 97.0, "each key is one undo step");
}

#[gpui_kit::test]
fn z_order_size_copy_and_autosize(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    select_c1(&mut ui);
    ui.press("ctrl-shift-f");
    assert_eq!(ui.doc(|d| d.shapes.last().unwrap().id.clone()), "c1");
    ui.press("ctrl-shift-b");
    assert_eq!(ui.doc(|d| d.shapes[0].id.clone()), "c1");
    ui.press("ctrl-alt-shift-f");
    assert_eq!(ui.doc(|d| d.shapes[1].id.clone()), "c1");
    ui.press("ctrl-alt-shift-b");
    assert_eq!(ui.doc(|d| d.shapes[0].id.clone()), "c1");

    ui.press("ctrl-right ctrl-right alt-shift-f tab alt-shift-v");
    assert_eq!(ui.shape("c2").w, 162.0, "size copied from c1");
    ui.press("ctrl-shift-y");
    assert!(ui.shape("c2").w < 162.0, "autosize shrinks to the label");
}

#[gpui_kit::test]
fn view_toggles(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.press("ctrl-shift-g");
    assert!(!ui.read(|a| a.wb.show_grid));
    ui.press("ctrl-shift-o");
    assert!(!ui.read(|a| a.wb.show_minimap));
    ui.press("ctrl-shift-k");
    assert!(!ui.read(|a| a.wb.show_sidebar));
    assert!(!ui.exists("toolbox"));
    ui.press("ctrl-shift-p");
    assert!(!ui.read(|a| a.wb.show_navigator));
    assert!(!ui.exists("tree"));
    ui.press("ctrl-shift-k ctrl-shift-p");
    assert!(ui.exists("toolbox") && ui.exists("tree"));
    ui.press("ctrl-= ctrl-=");
    assert!((ui.read(|a| a.wb.zoom) - 1.21).abs() < 1e-3);
    ui.press("ctrl-- ctrl-0");
    assert_eq!(ui.read(|a| a.wb.zoom), 1.0);
}

#[gpui_kit::test]
fn escape_leaves_tools_and_typing_never_edits_diagram(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("tool-0-6"));
    assert_eq!(ui.read(|a| a.wb.active_tool.clone()), "Association");
    ui.press("escape");
    assert_eq!(ui.read(|a| a.wb.active_tool.clone()), "Select");

    // Delete inside the explorer search must not delete the selected shape.
    select_c1(&mut ui);
    ui.update(|a, w, cx| a.explorer_search.update(cx, |s, cx| s.focus(w, cx)));
    ui.type_text("cu");
    ui.press("backspace delete ctrl-a");
    assert!(ui.doc(|d| d.shape("c1").is_some()));
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c1".to_string()]
    );
}
