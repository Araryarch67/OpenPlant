//! Pointer interaction on the canvas: select, move, resize, connect, zoom.

use super::*;
use crate::canvas::{GRID, Handle, connect_arrows, handle_pos, minimap_geom};
use crate::model::RelationKind;
use gpui_kit::{Modifiers, MouseButton, TestAppContext, point, px};

fn snap(v: f32) -> f32 {
    (v / GRID).round() * GRID
}

#[gpui_kit::test]
fn click_selects_and_fills_property_editor(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Left, Default::default());
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c1".to_string()]
    );
    assert_eq!(ui.read(|a| a.prop.node.clone()).as_deref(), Some("c1"));
    assert!(ui.exists(id("sel-visibility")), "class schema shown");
    assert!(
        ui.find(id("swatch-fill")).is_some(),
        "styles section rendered (below the fold)"
    );
    let name = ui
        .read(|a| {
            a.prop
                .fields
                .iter()
                .find(|(k, _)| *k == "name")
                .map(|(_, e)| e.clone())
        })
        .unwrap();
    assert_eq!(
        name.read_with(ui.cx, |s, _| s.value().to_string()),
        "Customer"
    );

    let blank = ui.at(900.0, 600.0);
    ui.click_at(blank, MouseButton::Left, Default::default());
    assert!(
        ui.read(|a| a.wb.selected_shapes.is_empty()),
        "blank click clears selection"
    );
}

#[gpui_kit::test]
fn drag_moves_on_grid_and_undoes(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let from = ui.center_of("c1");
    ui.drag(from, point(from.x + px(50.), from.y + px(30.)));
    let s = ui.shape("c1");
    assert_eq!((s.x, s.y), (snap(170.0), snap(130.0)));
    ui.press("ctrl-z");
    let s = ui.shape("c1");
    assert_eq!((s.x, s.y), (120.0, 100.0), "one undo step per drag");
    ui.press("ctrl-y");
    assert_eq!(ui.shape("c1").x, snap(170.0));
    ui.press("ctrl-z ctrl-shift-z");
    assert_eq!(ui.shape("c1").x, snap(170.0), "ctrl-shift-z redoes too");
}

#[gpui_kit::test]
fn drag_modifiers_free_axis_and_copy(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let from = ui.center_of("c1");
    // Alt while moving: no grid snap.
    ui.drag_with(
        from,
        point(from.x + px(5.), from.y + px(7.)),
        Modifiers::default(),
        alt(),
    );
    assert_eq!((ui.shape("c1").x, ui.shape("c1").y), (125.0, 107.0));
    // Shift: constrain to the dominant axis.
    let from = ui.center_of("c1");
    ui.drag_with(
        from,
        point(from.x + px(64.), from.y + px(20.)),
        Modifiers::default(),
        shift(),
    );
    assert_eq!(ui.shape("c1").y, 107.0, "locked axis unchanged");
    assert_eq!(ui.shape("c1").x, snap(125.0 + 64.0), "free axis snaps");
    // Ctrl+drag: copy, original stays.
    let before = ui.shape("c1");
    let from = ui.center_of("c1");
    ui.drag_with(
        from,
        point(from.x + px(0.), from.y + px(160.)),
        ctrl(),
        Modifiers::default(),
    );
    assert_eq!(ui.doc(|d| d.shapes.len()), 3);
    assert_eq!(ui.shape("c1"), before);
}

#[gpui_kit::test]
fn resize_handle_resizes_selected_shape(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Left, Default::default());
    let s = ui.shape("c1");
    let (hx, hy) = handle_pos(&s, Handle::SE);
    let from = ui.at(hx, hy);
    ui.drag(from, point(from.x + px(40.), from.y + px(20.)));
    let r = ui.shape("c1");
    assert_eq!((r.x, r.y), (120.0, 100.0));
    assert_eq!((r.w, r.h), (snap(320.0) - 120.0, snap(216.0) - 100.0));
}

#[gpui_kit::test]
fn toolbox_click_places_until_escape(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("tool-0-1")); // Interface
    assert_eq!(ui.read(|a| a.wb.active_tool.clone()), "Interface");
    assert!(ui.exists("tool-hint"));
    let p = ui.at(700.0, 400.0);
    ui.click_at(p, MouseButton::Left, Default::default());
    let p2 = ui.at(700.0, 600.0);
    ui.click_at(p2, MouseButton::Left, Default::default());
    assert_eq!(ui.doc(|d| d.shapes.len()), 4, "tool stays active");
    ui.press("escape");
    assert_eq!(ui.read(|a| a.wb.active_tool.clone()), "Select");
    assert!(!ui.exists("tool-hint"));

    ui.click(id("tool-0-0"));
    ui.click("tool-hint-x");
    assert_eq!(
        ui.read(|a| a.wb.active_tool.clone()),
        "Select",
        "hint close button"
    );
}

#[gpui_kit::test]
fn toolbox_drag_drops_onto_canvas(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.drag_ids(id("tool-0-3"), "diagram-wrap");
    let s = ui.doc(|d| d.shapes.last().cloned()).unwrap();
    assert_eq!(s.tool.as_deref(), Some("Enumeration"));
}

#[gpui_kit::test]
fn edge_tool_connects_two_shapes(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("tool-0-11")); // Generalization
    let (c2, c1) = (ui.center_of("c2"), ui.center_of("c1"));
    ui.click_at(c2, MouseButton::Left, Default::default());
    assert_eq!(ui.read(|a| a.pending_from.clone()).as_deref(), Some("c2"));
    ui.click_at(c1, MouseButton::Left, Default::default());
    let r = ui.doc(|d| d.relations.last().cloned()).unwrap();
    assert_eq!(
        (r.from.as_str(), r.to.as_str(), r.kind),
        ("c2", "c1", RelationKind::Generalization)
    );
    assert!(ui.read(|a| a.pending_from.is_none()));
}

#[gpui_kit::test]
fn hover_arrow_drag_connects(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let c1 = ui.center_of("c1");
    ui.mv(c1, None, Default::default());
    assert_eq!(ui.read(|a| a.hover.clone()).as_deref(), Some("c1"));
    let s = ui.shape("c1");
    let xf = ui.read(|a| a.xf());
    let (ax, ay, _, _) = connect_arrows(&s, &xf)[1]; // right-hand arrow
    let to = ui.center_of("c2");
    ui.drag(point(px(ax), px(ay)), to);
    assert_eq!(ui.doc(|d| d.relations.len()), 2);
    let r = ui.doc(|d| d.relations.last().cloned()).unwrap();
    assert_eq!((r.from.as_str(), r.to.as_str()), ("c1", "c2"));
}

#[gpui_kit::test]
fn rubber_band_selects_enclosed(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let (a, b) = (ui.at(60.0, 60.0), ui.at(600.0, 400.0));
    ui.drag(a, b);
    let mut sel = ui.read(|a| a.wb.selected_shapes.clone());
    sel.sort();
    assert_eq!(sel, vec!["c1".to_string(), "c2".to_string()]);
    assert_eq!(
        ui.read(|a| a.wb.selected_relations.clone()),
        vec!["r1".to_string()]
    );
    // Shift+click toggles one out.
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Left, shift());
    assert_eq!(
        ui.read(|a| a.wb.selected_shapes.clone()),
        vec!["c2".to_string()]
    );
}

#[gpui_kit::test]
fn double_click_edits_label_inline(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let c1 = ui.center_of("c1");
    ui.double_click_at(c1);
    assert!(ui.exists("label-editor"));
    ui.press("ctrl-a");
    ui.type_text("Client");
    ui.press("enter");
    assert!(!ui.exists("label-editor"));
    assert_eq!(ui.shape("c1").label, "Client");
    assert_eq!(
        ui.read(|a| a.wb.project.root.find("c1").unwrap().name.clone()),
        "Client"
    );

    // Escape cancels without renaming.
    ui.double_click_at(c1);
    ui.press("ctrl-a");
    ui.type_text("Nope");
    ui.press("escape");
    assert_eq!(ui.shape("c1").label, "Client");
}

#[gpui_kit::test]
fn relation_click_selects_and_label_edits(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let (a, b) = (ui.shape("c1"), ui.shape("c2"));
    let route = crate::canvas::ortho_route(&a, &b);
    let (mx, my) = route.label;
    let p = ui.at(mx, my);
    ui.click_at(p, MouseButton::Left, Default::default());
    assert_eq!(
        ui.read(|a| a.wb.selected_relations.clone()),
        vec!["r1".to_string()]
    );
    ui.press("f2");
    ui.press("ctrl-a");
    ui.type_text("buys");
    ui.press("enter");
    assert_eq!(
        ui.doc(|d| d.relations[0].label.clone()).as_deref(),
        Some("buys")
    );
    ui.press("delete");
    assert!(ui.doc(|d| d.relations.is_empty()));
}

#[gpui_kit::test]
fn wheel_zooms_pans_and_fit_recentres(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let p = ui.at(400.0, 300.0);
    ui.wheel(p, 0.0, 20.0, ctrl());
    let z = ui.read(|a| a.wb.zoom);
    assert!(z > 1.0, "ctrl+wheel zooms ({z})");
    let world_before = (400.0f32, 300.0f32);
    let after = ui.at(world_before.0, world_before.1);
    assert!(
        (after.x - p.x).abs() < px(1.) && (after.y - p.y).abs() < px(1.),
        "zoom keeps the point under the cursor"
    );
    let pan = ui.read(|a| a.wb.pan);
    ui.wheel(p, 0.0, 40.0, Default::default());
    assert_eq!(ui.read(|a| a.wb.pan.1), pan.1 + 40.0, "plain wheel scrolls");
    ui.wheel(p, 0.0, 40.0, shift());
    assert_eq!(
        ui.read(|a| a.wb.pan.0),
        pan.0 + 40.0,
        "shift+wheel scrolls sideways"
    );

    ui.press("ctrl-shift-h");
    let (cx_, cy_, w, h) = ui.read(|a| a.canvas_bounds.get());
    let c = ui.read(|a| crate::canvas::bbox(a.wb.doc())).unwrap();
    let mid = ui.at((c.0 + c.2) / 2.0, (c.1 + c.3) / 2.0);
    assert!((f32::from(mid.x) - (cx_ + w / 2.0)).abs() < 2.0);
    assert!((f32::from(mid.y) - (cy_ + h / 2.0)).abs() < 2.0);
    ui.press("home");
    assert_eq!(ui.read(|a| (a.wb.zoom, a.wb.pan)), (1.0, (0.0, 0.0)));
}

#[gpui_kit::test]
fn middle_and_space_drag_pan(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let p = ui.at(700.0, 500.0);
    let to = point(p.x + px(30.), p.y + px(10.));
    ui.mv(p, None, Default::default());
    ui.down(p, MouseButton::Middle, Default::default(), 1);
    ui.mv(to, Some(MouseButton::Middle), Default::default());
    ui.up(to, MouseButton::Middle, Default::default(), 1);
    assert_eq!(ui.read(|a| a.wb.pan), (30.0, 10.0));
}

#[gpui_kit::test]
fn minimap_click_recentres(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    let mm = ui.read(|a| {
        minimap_geom(a.wb.doc(), &a.xf(), {
            let (x, y, w, h) = a.canvas_bounds.get();
            gpui_kit::Bounds::new(point(px(x), px(y)), gpui_kit::size(px(w), px(h)))
        })
    });
    let p = point(px(mm.x + 20.0), px(mm.y + 20.0));
    let target = mm.to_world(mm.x + 20.0, mm.y + 20.0);
    ui.click_at(p, MouseButton::Left, Default::default());
    let (_, _, w, h) = ui.read(|a| a.canvas_bounds.get());
    let centre = ui.read(|a| a.xf().world(a.xf().ox + w / 2.0, a.xf().oy + h / 2.0));
    assert!((centre.0 - target.0).abs() < 1.0 && (centre.1 - target.1).abs() < 1.0);
}
