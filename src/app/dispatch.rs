//! Command dispatch by stable id.

use super::*;
use crate::model::*;
use crate::plantuml::generate_plantuml;
use crate::workbench::*;
use gpui_kit::*;

impl OpenPlant {
    pub fn run(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        // Key bindings run before key listeners, and a single-line input
        // propagates Enter/Tab, so while a label editor is open these canvas
        // commands mean "finish editing".
        if self.editing.is_some()
            && matches!(id, "edit.label" | "edit.select-next" | "edit.select-prev")
        {
            return self.commit_edit(window, cx);
        }
        self.close_popups();
        let wb = &mut self.wb;
        match id {
            "file.new" => {
                wb.reset();
                self.prop.node = None;
            }
            "file.open" => return self.open_file(false, window, cx),
            "file.import-puml" => return self.open_file(true, window, cx),
            "file.save" => return self.save(false, window, cx),
            "file.save-as" => return self.save(true, window, cx),
            "file.export-puml" => {
                let text = generate_plantuml(wb.doc());
                let name = format!("{}.puml", wb.doc().name);
                return self.save_dialog(
                    name,
                    text.into_bytes(),
                    "PlantUML exported.",
                    false,
                    window,
                    cx,
                );
            }
            "file.export-svg" => {
                let svg = crate::services::export::to_svg(wb.doc());
                let name = format!("{}.svg", wb.doc().name);
                return self.save_dialog(
                    name,
                    svg.into_bytes(),
                    "SVG exported.",
                    false,
                    window,
                    cx,
                );
            }
            "file.export-png" => {
                let name = format!("{}.png", wb.doc().name);
                match crate::services::export::to_png(wb.doc()) {
                    Ok(png) => {
                        return self.save_dialog(name, png, "PNG exported.", false, window, cx);
                    }
                    Err(e) => {
                        return self.toast(
                            ToastKind::Error,
                            format!("PNG export failed: {e}"),
                            window,
                            cx,
                        );
                    }
                }
            }
            "app.quit" => cx.quit(),
            "window.minimize" => window.minimize_window(),
            "edit.undo" => {
                if !wb.undo() {
                    return self.toast(ToastKind::Info, "Nothing to undo.", window, cx);
                }
            }
            "edit.redo" => {
                if !wb.redo() {
                    return self.toast(ToastKind::Info, "Nothing to redo.", window, cx);
                }
            }
            "edit.cut" => wb.cut_selection(),
            "edit.copy" => {
                wb.copy_selection();
            }
            "edit.paste" => wb.paste(),
            "edit.duplicate" => wb.duplicate_selection(),
            "edit.delete" | "edit.delete-all" => wb.delete_selection(),
            "edit.select-all" => wb.select_all(),
            "edit.select-none" => wb.select_none(),
            "edit.select-vertices" => wb.select_vertices(),
            "edit.select-edges" => wb.select_edges(),
            "edit.select-next" => wb.select_step(true),
            "edit.select-prev" => wb.select_step(false),
            "edit.label" => {
                let t = wb
                    .selected_shapes
                    .first()
                    .map(|s| EditTarget::Shape(s.clone()))
                    .or_else(|| {
                        wb.selected_relations
                            .first()
                            .map(|r| EditTarget::Relation(r.clone()))
                    });
                if let Some(t) = t {
                    return self.begin_edit(t, window, cx);
                }
            }
            "edit.copy-size" => wb.copy_size(),
            "edit.paste-size" => wb.paste_size(),
            "edit.cancel" => {
                self.pending_from = None;
                if wb.active_tool != "Select" {
                    wb.active_tool = "Select".into();
                } else {
                    wb.select_none();
                }
            }
            "format.autosize" => wb.autosize(),
            "format.line-solid" => wb.set_style(|s| s.dash = Some(Dash::Solid)),
            "format.line-dashed" => wb.set_style(|s| s.dash = Some(Dash::Dashed)),
            "format.line-dotted" => wb.set_style(|s| s.dash = Some(Dash::Dotted)),
            "format.clear" => wb.clear_style(),
            "arrange.front" => wb.reorder(true, false),
            "arrange.back" => wb.reorder(false, false),
            "arrange.forward" => wb.reorder(true, true),
            "arrange.backward" => wb.reorder(false, true),
            "align.left" => wb.align(AlignMode::Left),
            "align.center" => wb.align(AlignMode::CenterX),
            "align.right" => wb.align(AlignMode::Right),
            "align.top" => wb.align(AlignMode::Top),
            "align.middle" => wb.align(AlignMode::Middle),
            "align.bottom" => wb.align(AlignMode::Bottom),
            "align.dist-h" => wb.align(AlignMode::DistH),
            "align.dist-v" => wb.align(AlignMode::DistV),
            "align.same-w" => wb.align(AlignMode::SameW),
            "align.same-h" => wb.align(AlignMode::SameH),
            "diagram.add-class" => {
                wb.add_shape("Class", 240.0, 160.0, None);
            }
            "diagram.add-interface" => {
                wb.add_shape("Interface", 260.0, 180.0, None);
            }
            "diagram.add-package" => {
                wb.add_shape("Package", 280.0, 200.0, None);
            }
            "view.palette" => return self.open_palette(window, cx),
            "view.sidebar" => wb.show_sidebar = !wb.show_sidebar,
            "view.navigator" => wb.show_navigator = !wb.show_navigator,
            "view.outline" => wb.show_minimap = !wb.show_minimap,
            "view.grid" => wb.show_grid = !wb.show_grid,
            "view.zoom-in" => self.zoom_by(1.1),
            "view.zoom-out" => self.zoom_by(1.0 / 1.1),
            "view.reset" => self.reset_view(),
            "view.fit" => self.pending_fit = true,
            "help.shortcuts" => self.shortcuts_open = !self.shortcuts_open,
            "help.about" => {
                return self.toast(
                    ToastKind::Info,
                    "OpenPlant 0.1.0 — StarUML / draw.io alternative, native GPUI build.",
                    window,
                    cx,
                );
            }
            _ => {}
        }
        cx.notify();
    }

    pub fn nudge(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        self.wb.nudge(dx, dy);
        cx.notify();
    }

    pub fn grow(&mut self, dw: f32, dh: f32, cx: &mut Context<Self>) {
        self.wb.resize_selected(dw, dh);
        cx.notify();
    }
}
