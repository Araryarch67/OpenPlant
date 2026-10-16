//! Sidebar, model explorer, property editor, styles and status bar.

use super::*;
use crate::model::Dash;
use crate::workbench::AttrValue;
use gpui_kit::{MouseButton, TestAppContext, point, px};

fn select(ui: &mut Ui, shape: &str) {
    let p = ui.center_of(shape);
    ui.click_at(p, MouseButton::Left, Default::default());
}

fn field(ui: &mut Ui, key: &str) -> gpui_kit::Entity<gpui_kit::component::input::InputState> {
    ui.read(|a| {
        a.prop
            .fields
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, e)| e.clone())
    })
    .unwrap_or_else(|| panic!("no field {key}"))
}

#[gpui_kit::test]
fn explorer_search_sort_collapse_select(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    assert!(ui.exists(id("tree-c1")) && ui.exists(id("tree-c2")));
    ui.update(|a, w, cx| a.explorer_search.update(cx, |s, cx| s.focus(w, cx)));
    ui.type_text("ord");
    assert!(ui.exists(id("tree-c2")));
    assert!(!ui.exists(id("tree-c1")), "search filters the tree");
    ui.update(|a, w, cx| a.explorer_search.update(cx, |s, cx| s.set_value("", w, cx)));

    ui.click(id("sort-az"));
    assert!(ui.read(|a| a.explorer_sorted));
    ui.click(id("twist-diag1"));
    assert!(!ui.exists(id("tree-c1")), "collapse hides children");
    ui.click(id("twist-diag1"));
    ui.click(id("tree-c2"));
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c2".to_string()]
    );
    ui.click(id("tree-proj"));
    assert!(
        ui.read(|a| a.prop.fields.iter().any(|(k, _)| *k == "author")),
        "project schema"
    );
}

#[gpui_kit::test]
fn property_fields_edit_the_model(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    select(&mut ui, "c1");
    let name = field(&mut ui, "name");
    ui.update(|_, w, cx| name.update(cx, |s, cx| s.focus(w, cx)));
    ui.press("ctrl-a");
    ui.type_text("Client");
    assert_eq!(
        ui.shape("c1").label,
        "Client",
        "name edits rename the shape live"
    );
    let shapes = ui.doc(|d| d.shapes.len());
    ui.press("ctrl-z");
    let text = name.read_with(ui.cx, |s, _| s.value().to_string());
    assert_ne!(text, "Client", "ctrl-z in the field undoes typing");
    assert_eq!(ui.shape("c1").label, text, "label follows the field");
    assert_eq!(
        ui.doc(|d| d.shapes.len()),
        shapes,
        "diagram undo did not run"
    );

    ui.click(id("sel-visibility"));
    ui.click(id("opt-visibility-1"));
    assert!(ui.read(|a| a.wb.attr_text("c1", "visibility")).as_deref() == Some("private"));

    ui.click(id("chk-isAbstract"));
    assert!(ui.read(|a| a.wb.attr_bool("c1", "isAbstract")));

    ui.click(id("pick-stereotype"));
    assert!(ui.exists("picker"));
    ui.click(("pick", 3usize));
    assert!(!ui.exists("picker"));
    let st = ui.read(|a| {
        a.wb.node_attrs
            .get("c1")
            .and_then(|m| m.get("stereotype"))
            .cloned()
    });
    assert!(matches!(st, Some(AttrValue::Text(t)) if !t.is_empty()));

    let doc = ui.read(|a| a.prop.doc.clone()).unwrap();
    ui.update(|_, w, cx| doc.update(cx, |s, cx| s.focus(w, cx)));
    ui.type_text("Pays invoices");
    assert_eq!(
        ui.read(|a| a.wb.node_docs.get("c1").cloned()).as_deref(),
        Some("Pays invoices")
    );

    ui.click(id("sec-props"));
    assert!(!ui.exists(id("sel-visibility")), "section collapses");
}

#[gpui_kit::test]
fn styles_section_controls(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    select(&mut ui, "c1");
    // Bring Styles above the fold of the editors column.
    ui.click(id("sec-docs"));
    ui.click(id("sec-puml"));
    ui.click(id("swatch-fill"));
    ui.click(id("preset-fill-6"));
    let st = ui.shape("c1").style.unwrap();
    assert_eq!(st.fill.as_deref(), Some("#fee2e2"));
    ui.click(id("swatch-stroke"));
    ui.click(id("preset-stroke-14"));
    ui.click(id("swatch-text"));
    ui.click(id("preset-text-17"));
    let st = ui.shape("c1").style.unwrap();
    assert_eq!(
        (st.stroke.as_deref(), st.text.as_deref()),
        (Some("#e57474"), Some("#67b0e8"))
    );

    ui.click(("dash", 1usize));
    assert_eq!(ui.shape("c1").style.unwrap().dash, Some(Dash::Dashed));
    ui.click(id("sel-font"));
    ui.click(id("opt-font-1"));
    assert_eq!(
        ui.shape("c1").style.unwrap().font.as_deref(),
        Some("Georgia, serif")
    );

    let size = ui.read(|a| a.prop.font_size.clone()).unwrap();
    ui.update(|_, w, cx| size.update(cx, |s, cx| s.focus(w, cx)));
    ui.press("ctrl-a");
    ui.type_text("20");
    assert_eq!(ui.shape("c1").style.unwrap().font_size, Some(20.0));

    ui.click("inherit");
    assert!(ui.shape("c1").style.is_none(), "inherit clears overrides");

    ui.click(id("to-front"));
    assert_eq!(ui.doc(|d| d.shapes.last().unwrap().id.clone()), "c1");
    ui.click(id("to-back"));
    assert_eq!(ui.doc(|d| d.shapes[0].id.clone()), "c1");

    // Alignment needs two shapes.
    let c2x = ui.shape("c2").x;
    ui.click(id("al-align.left"));
    assert_eq!(ui.shape("c2").x, c2x, "disabled with one shape");
    // Give the canvas focus back (the font size field still has it).
    ui.update(|a, w, cx| a.focus_canvas(w, cx));
    ui.press("ctrl-a");
    println!("DBG sel={:?} btn={:?} editors={:?}", ui.read(|a| a.wb.selected_shapes.clone()), ui.bounds(id("al-align.left")), ui.bounds("editors"));
    ui.click(id("al-align.left"));
    assert_eq!(ui.shape("c2").x, ui.shape("c1").x);
}

#[gpui_kit::test]
fn sidebar_diagrams_groups_and_resizer(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.update(|a, _, _| a.wb.add_diagram("Sequence"));
    let new_id = ui.read(|a| a.wb.project.active_diagram_id.clone());
    ui.click(id("wd-diag1"));
    assert_eq!(ui.read(|a| a.wb.project.active_diagram_id.clone()), "diag1");
    ui.click(id(&format!("wdx-{new_id}")));
    assert_eq!(ui.read(|a| a.wb.project.diagrams.len()), 1);
    assert!(
        !ui.exists(id("wdx-diag1")),
        "the last diagram cannot be deleted"
    );

    assert!(!ui.exists(id("tool-1-0")));
    ui.click(("grp", 1usize));
    assert!(ui.exists(id("tool-1-0")), "Use Case group opens");
    ui.click(("grp", 0usize));
    assert!(!ui.exists(id("tool-0-0")), "Class group closes");

    let r = ui.bounds("resize-left").center();
    ui.drag(r, point(r.x + px(40.), r.y));
    assert_eq!(ui.read(|a| a.wb.sidebar_w), 304.0);
    let r = ui.bounds("resize-right").center();
    ui.drag(r, point(r.x - px(30.), r.y));
    assert_eq!(ui.read(|a| a.wb.navigator_w), 330.0);
}

#[gpui_kit::test]
fn statusbar_and_select_tool_buttons(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("st-zoom-in"));
    assert!((ui.read(|a| a.wb.zoom) - 1.1).abs() < 1e-3);
    ui.click(id("st-zoom-out"));
    assert!((ui.read(|a| a.wb.zoom) - 1.0).abs() < 1e-3);
    ui.click(id("tool-0-0"));
    ui.click(id("tool-select"));
    assert_eq!(ui.read(|a| a.wb.active_tool.clone()), "Select");
}
