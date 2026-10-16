//! Tool strip and bottom drawer tabs.

use super::*;
use crate::plantuml::generate_plantuml;
use gpui_kit::{MouseButton, TestAppContext};

#[gpui_kit::test]
fn every_tab_opens_and_closes(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    for (tab, probe) in [
        ("ext", "ext-markdown"),
        ("diagrams", "thumb-diag1"),
        ("model", "drawer-body"),
        ("nav", "md-mode-0"),
        ("share", "drawer-body"),
        ("puml", "puml-load"),
        ("ai", "ai-generate"),
    ] {
        ui.click(id(&format!("strip-{tab}")));
        assert_eq!(ui.read(|a| a.wb.bottom_tab.clone()).as_deref(), Some(tab));
        assert!(
            ui.exists(id(probe)) || ui.exists(probe),
            "{tab} tab renders {probe}"
        );
    }
    ui.click(id("strip-ai"));
    assert!(
        ui.read(|a| a.wb.bottom_tab.is_none()),
        "clicking the open tab closes it"
    );
    ui.click(id("strip-ext"));
    ui.click(id("drawer-close"));
    assert!(ui.read(|a| a.wb.bottom_tab.is_none()));
}

#[gpui_kit::test]
fn extensions_gate_their_tabs(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("strip-ext"));
    ui.click(id("ext-thumbs"));
    assert!(!ui.read(|a| a.wb.ext_on("thumbs")));
    ui.click(id("strip-diagrams"));
    assert!(ui.exists(id("enable-thumbs")));
    assert!(!ui.exists(id("thumb-diag1")));
    ui.click(id("enable-thumbs"));
    assert!(ui.exists(id("thumb-diag1")));

    ui.update(|a, _, _| a.wb.add_diagram("Class"));
    ui.click(id("thumb-diag1"));
    assert_eq!(
        ui.read(|a| a.wb.project.active_diagram_id.clone()),
        "diag1",
        "thumbnail switches diagram"
    );
}

#[gpui_kit::test]
fn plantuml_tab_syncs_both_ways(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("strip-puml"));
    let text = |ui: &mut Ui| {
        let inp = ui.read(|a| a.puml_input.clone());
        inp.read_with(ui.cx, |s, _| s.value().to_string())
    };
    let generated = ui.doc(generate_plantuml);
    assert_eq!(text(&mut ui), generated, "script follows the canvas");

    // Canvas -> text while the script is not focused.
    let c1 = ui.center_of("c1");
    ui.click_at(c1, MouseButton::Left, Default::default());
    ui.press("shift-right");
    assert!(
        text(&mut ui).contains("' pos Customer: 136,100"),
        "{}",
        text(&mut ui)
    );

    // Text -> canvas after the debounce.
    let inp = ui.read(|a| a.puml_input.clone());
    ui.update(|_, w, cx| inp.update(cx, |s, cx| s.focus(w, cx)));
    ui.press("ctrl-a");
    ui.type_text("@startuml\nclass Shop\nclass Item\nShop --> Item : stocks\n@enduml");
    ui.advance(800);
    let labels: Vec<String> = ui.doc(|d| d.shapes.iter().map(|s| s.label.clone()).collect());
    assert_eq!(labels, vec!["Shop".to_string(), "Item".to_string()]);
    assert_eq!(
        ui.doc(|d| d.relations[0].label.clone()).as_deref(),
        Some("stocks")
    );
    assert!(
        ui.read(|a| a.puml_status.clone())
            .starts_with("Rendered 2 classes")
    );
    ui.press("ctrl-z");
    // ctrl-z goes to the textarea while it is focused; canvas undo is separate.
    assert_eq!(ui.doc(|d| d.shapes.len()), 2);

    ui.click("puml-auto");
    assert!(!ui.read(|a| a.puml_auto));
    ui.click("puml-follow");
    assert!(!ui.read(|a| a.puml_follow));
    ui.click(id("puml-load"));
    assert_eq!(text(&mut ui), ui.doc(generate_plantuml));
}

#[gpui_kit::test]
fn markdown_tab_edits_and_previews(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("strip-nav"));
    let md = ui.read(|a| a.md_input.clone());
    ui.update(|_, w, cx| md.update(cx, |s, cx| s.focus(w, cx)));
    ui.type_text("# Notes\n\nCustomers place *orders*.");
    assert_eq!(
        ui.read(|a| a.wb.node_docs.get("diag1").cloned()).as_deref(),
        Some("# Notes\n\nCustomers place *orders*.")
    );
    ui.click(id("md-mode-1"));
    assert!(
        ui.read(|a| a.md_preview),
        "preview renders the markdown view"
    );
    ui.click(id("md-mode-0"));
    assert!(!ui.read(|a| a.md_preview));
}

#[gpui_kit::test]
fn ai_tab_controls(cx: &mut TestAppContext) {
    let mut ui = Ui::open(cx);
    ui.click(id("strip-ai"));
    ui.click(id("ai-prov-1"));
    assert!(!ui.read(|a| a.ai.local), "Cloud API selected");
    ui.click("ai-settings");
    assert!(ui.read(|a| a.ai.settings_open));
    ui.click(("ai-ex", 1usize));
    let prompt = ui.read(|a| a.ai.prompt.clone());
    assert_eq!(
        prompt.read_with(ui.cx, |s, _| s.value().to_string()),
        crate::services::ai::EXAMPLES[1]
    );
}
