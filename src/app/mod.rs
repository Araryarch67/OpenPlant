//! Root view state: owns the workbench, the canvas interaction state and
//! command dispatch. Region rendering lives in `ui`.

mod dispatch;
mod editing;
mod files;
mod palette;
mod pointer;
mod props;
mod sync;
mod view;

use crate::canvas::*;
use crate::services::settings::Settings;
use crate::workbench::*;
use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::*;
use std::cell::Cell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

pub enum Drag {
    None,
    Pan {
        sx: f32,
        sy: f32,
        px: f32,
        py: f32,
        moved: bool,
        right: bool,
    },
    Move {
        origins: Vec<(String, f32, f32)>,
        sx: f32,
        sy: f32,
        moved: bool,
        clone: bool,
        click: String,
        was_selected: bool,
    },
    Rubber {
        sx: f32,
        sy: f32,
        cx: f32,
        cy: f32,
        additive: bool,
    },
    Resize {
        id: String,
        handle: Handle,
        orig: (f32, f32, f32, f32),
        sx: f32,
        sy: f32,
        started: bool,
    },
    Connect {
        from: String,
        cx: f32,
        cy: f32,
        target: Option<String>,
    },
    MiniMap,
    Splitter {
        left: bool,
        sx: f32,
        w0: f32,
    },
}

#[derive(Clone, PartialEq)]
pub enum EditTarget {
    Shape(String),
    Relation(String),
}

pub struct Editing {
    pub target: EditTarget,
    pub input: Entity<InputState>,
    _sub: Subscription,
}

#[derive(Clone)]
pub struct CtxMenu {
    pub x: f32,
    pub y: f32,
    pub wx: f32,
    pub wy: f32,
    pub shape: Option<String>,
    pub sub: Option<u8>,
}

pub struct PaletteState {
    pub input: Entity<InputState>,
    pub idx: usize,
    _sub: Subscription,
}

/// Inputs bound to the property editor for the selected model node.
pub struct PropInputs {
    pub node: Option<String>,
    pub fields: Vec<(&'static str, Entity<InputState>)>,
    pub doc: Option<Entity<TextareaState>>,
    pub font_size: Option<Entity<InputState>>,
    _subs: Vec<Subscription>,
}

#[derive(Clone)]
pub struct ToolDrag(pub String);

pub struct DragGhost(pub String);

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(crate::theme::c(crate::theme::RAISED))
            .border_1()
            .border_color(crate::theme::c(crate::theme::BLUE))
            .text_color(crate::theme::c(crate::theme::FG))
            .text_size(px(12.))
            .child(self.0.clone())
    }
}

pub struct OpenPlant {
    pub wb: Workbench,
    pub settings: Settings,
    pub focus: FocusHandle,
    pub canvas_focus: FocusHandle,
    /// Canvas element bounds in window coordinates (x, y, w, h).
    pub canvas_bounds: Rc<Cell<(f32, f32, f32, f32)>>,
    pub drag: Drag,
    pub hover: Option<String>,
    pub pending_from: Option<String>,
    pub space_down: bool,
    pub cursor: CursorStyle,
    pub open_menu: Option<usize>,
    pub ctx_menu: Option<CtxMenu>,
    pub palette: Option<PaletteState>,
    pub shortcuts_open: bool,
    pub editing: Option<Editing>,
    pub explorer_search: Entity<InputState>,
    pub explorer_sorted: bool,
    pub collapsed: HashSet<String>,
    pub closed_sections: HashSet<&'static str>,
    pub prop: PropInputs,
    pub open_dropdown: Option<&'static str>,
    pub color_pop: Option<&'static str>,
    pub picker_for: Option<&'static str>,
    pub puml_input: Entity<TextareaState>,
    pub puml_auto: bool,
    pub puml_follow: bool,
    pub puml_status: String,
    pub puml_last_set: String,
    pub puml_task: Option<Task<()>>,
    pub md_input: Entity<TextareaState>,
    pub md_preview: bool,
    pub md_doc: String,
    pub ai: crate::services::ai::AiState,
    /// Fit the view once the canvas has been laid out (after open/import).
    pub pending_fit: bool,
    /// Canvas size of the last automatic fit; while set, a resized canvas is
    /// re-fitted. Cleared as soon as the user pans or zooms.
    pub auto_fit: Option<(f32, f32)>,
    _subs: Vec<Subscription>,
}

impl OpenPlant {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = Settings::load();
        let mut wb = Workbench::new();
        wb.sidebar_w = settings.sidebar_w;
        wb.navigator_w = settings.navigator_w;
        let explorer_search = cx.new(|cx| InputState::new(window, cx).placeholder("search"));
        let puml_input = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder(
                "@startuml\nclass Customer\nclass Order\nCustomer --> Order : places\n@enduml",
            )
        });
        let md_input = cx.new(|cx| {
            TextareaState::new(window, cx).placeholder("Write markdown documentation...")
        });
        let ai = crate::services::ai::AiState::new(&settings, window, cx);
        let mut subs = vec![];
        subs.push(
            cx.subscribe_in(&explorer_search, window, |_, _, _: &InputEvent, _, cx| {
                cx.notify()
            }),
        );
        subs.push(cx.subscribe_in(
            &puml_input,
            window,
            |this, _, ev: &InputEvent, window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.schedule_puml(window, cx);
                }
            },
        ));
        subs.push(
            cx.subscribe_in(&md_input, window, |this, inp, ev: &InputEvent, _, cx| {
                if matches!(ev, InputEvent::Change) {
                    let id = this.wb.project.active_diagram_id.clone();
                    let v = inp.read(cx).value().to_string();
                    this.md_doc = v.clone();
                    this.wb.node_docs.insert(id, v);
                }
            }),
        );
        let focus = cx.focus_handle();
        let canvas_focus = cx.focus_handle();
        window.focus(&canvas_focus, cx);
        Self {
            wb,
            settings,
            focus,
            canvas_focus,
            canvas_bounds: Rc::new(Cell::new((0.0, 0.0, 0.0, 0.0))),
            drag: Drag::None,
            hover: None,
            pending_from: None,
            space_down: false,
            cursor: CursorStyle::Arrow,
            open_menu: None,
            ctx_menu: None,
            palette: None,
            shortcuts_open: false,
            editing: None,
            explorer_search,
            explorer_sorted: false,
            collapsed: HashSet::new(),
            closed_sections: HashSet::new(),
            prop: PropInputs {
                node: None,
                fields: vec![],
                doc: None,
                font_size: None,
                _subs: vec![],
            },
            open_dropdown: None,
            color_pop: None,
            picker_for: None,
            puml_input,
            puml_auto: true,
            puml_follow: true,
            puml_status: String::new(),
            puml_last_set: String::new(),
            puml_task: None,
            md_input,
            md_preview: false,
            md_doc: String::new(),
            ai,
            pending_fit: false,
            auto_fit: None,
            _subs: subs,
        }
    }

    /// `openplant <file>`: open a project or PlantUML file at startup.
    pub fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let contents = std::fs::read_to_string(&path);
        self.load_contents(path, contents, false, window, cx);
    }

    pub fn xf(&self) -> Xf {
        let (x, y, _, _) = self.canvas_bounds.get();
        Xf {
            ox: x,
            oy: y,
            pan_x: self.wb.pan.0,
            pan_y: self.wb.pan.1,
            k: self.wb.zoom,
        }
    }

    pub(crate) fn canvas_rect(&self) -> Bounds<Pixels> {
        let (x, y, w, h) = self.canvas_bounds.get();
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    pub fn close_popups(&mut self) {
        self.open_menu = None;
        self.ctx_menu = None;
        self.open_dropdown = None;
        self.color_pop = None;
    }

    pub fn any_popup(&self) -> bool {
        self.open_menu.is_some()
            || self.ctx_menu.is_some()
            || self.open_dropdown.is_some()
            || self.color_pop.is_some()
    }

    pub fn toast(
        &mut self,
        kind: ToastKind,
        msg: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.wb.toast(kind, msg);
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(4500))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.wb.dismiss_toast(id);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
