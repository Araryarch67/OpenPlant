//! Headless UI tests: the real OpenPlant view in a GPUI test window, driven
//! through native pointer and keyboard dispatch (never the user's desktop).

mod canvas;
mod drawer;
mod files;
mod keyboard;
mod menus;
mod panels;

use crate::app::OpenPlant;
use crate::model::{DiagramDoc, Shape};
use gpui_kit::base::test_support::{self as observe, ElementSnapshot};
use gpui_kit::{
    AnyWindowHandle, App, AppContext, Bounds, Context, ElementId, Entity, InputEvent as _,
    KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, Window,
    WindowBounds, WindowOptions, point, px, size,
};
use std::time::Duration;

pub(crate) fn id(name: &str) -> ElementId {
    crate::ui::eid(name)
}

pub(crate) fn ctrl() -> Modifiers {
    Modifiers {
        control: true,
        ..Default::default()
    }
}

pub(crate) fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Default::default()
    }
}

pub(crate) fn alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Default::default()
    }
}

/// Complete a frame so observed elements and listeners are current.
fn frame(w: &mut Window, cx: &mut App) {
    w.refresh();
    w.draw(cx).clear(cx);
}

/// Native key-down/key-up pair; Enter and Tab only run their bindings.
fn press_key(w: &mut Window, key: Keystroke, cx: &mut App) {
    let key = if matches!(key.key.as_str(), "enter" | "tab") {
        w.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        key
    } else {
        let key = key.with_simulated_ime();
        w.dispatch_keystroke(key.clone(), cx);
        key
    };
    w.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
}

fn input_text(w: &mut Window, text: &str, cx: &mut App) {
    for ch in text.chars() {
        let t = if ch == '\n' {
            "enter".to_string()
        } else {
            ch.to_string()
        };
        let mut key = Keystroke::parse(&t).expect("valid keystroke");
        if ch != '\n' {
            key.key_char = Some(ch.to_string());
            w.dispatch_keystroke(key, cx);
        } else {
            // Newlines in multi-line fields come through the IME path.
            key.key_char = Some("\n".into());
            w.dispatch_keystroke(key, cx);
        }
        frame(w, cx);
    }
}

pub(crate) struct Ui<'a> {
    pub cx: &'a mut TestAppContext,
    pub win: AnyWindowHandle,
    pub app: Entity<OpenPlant>,
}

impl<'a> Ui<'a> {
    /// Default project (Customer -> Order) in a 1600x900 window, like `shots/`.
    pub fn open(cx: &'a mut TestAppContext) -> Self {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::theme::apply(cx);
            crate::commands::bind_keys(cx);
        });
        let (win, app) = cx.update(|cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1600.), px(900.)),
                })),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| OpenPlant::new(window, cx))
            })
            .expect("open test window")
        });
        let mut ui = Ui { cx, win, app };
        ui.settle();
        ui
    }

    pub fn win<R>(&mut self, f: impl FnOnce(&mut Window, &mut App) -> R) -> R {
        self.cx
            .update_window(self.win, |_, w, cx| f(w, cx))
            .expect("window alive")
    }

    pub fn read<R>(&mut self, f: impl FnOnce(&OpenPlant) -> R) -> R {
        self.app.read_with(self.cx, |a, _| f(a))
    }

    pub fn update<R>(
        &mut self,
        f: impl FnOnce(&mut OpenPlant, &mut Window, &mut Context<OpenPlant>) -> R,
    ) -> R {
        let app = self.app.clone();
        let r = self.win(|w, cx| app.update(cx, |a, cx| f(a, w, cx)));
        self.settle();
        r
    }

    pub fn doc<R>(&mut self, f: impl FnOnce(&DiagramDoc) -> R) -> R {
        self.read(|a| f(a.wb.doc()))
    }

    pub fn shape(&mut self, id: &str) -> Shape {
        self.doc(|d| d.shape(id).cloned())
            .unwrap_or_else(|| panic!("no shape {id}"))
    }

    /// Run pending effects and tasks, then draw a fresh frame.
    pub fn settle(&mut self) {
        self.cx.run_until_parked();
        self.win(frame);
        self.cx.run_until_parked();
        self.win(frame);
    }

    pub fn advance(&mut self, ms: u64) {
        self.cx.executor().advance_clock(Duration::from_millis(ms));
        self.settle();
    }

    pub fn find(&mut self, target: impl Into<ElementId>) -> Option<ElementSnapshot> {
        let target = target.into();
        self.win(|w, cx| {
            frame(w, cx);
            observe::find(w, &[], &target)
        })
    }

    pub fn exists(&mut self, target: impl Into<ElementId>) -> bool {
        self.find(target).is_some_and(|s| s.visible())
    }

    pub fn bounds(&mut self, target: impl Into<ElementId>) -> Bounds<Pixels> {
        let target = target.into();
        match self.find(target.clone()) {
            Some(s) if s.visible() => s.bounds(),
            _ => {
                let paths = self.win(|w, _| observe::registered_paths(w));
                panic!("{target:?} not visible; registered: {paths}")
            }
        }
    }

    pub fn click(&mut self, target: impl Into<ElementId>) {
        let p = self.bounds(target).center();
        self.click_at(p, MouseButton::Left, Default::default());
    }

    pub fn hover(&mut self, target: impl Into<ElementId>) {
        let p = self.bounds(target).center();
        self.mv(p, None, Default::default());
        self.settle();
    }

    /// Press space-separated keys ("ctrl-z ctrl-y").
    pub fn press(&mut self, keys: &str) {
        for k in keys.split(' ') {
            let key = Keystroke::parse(k).unwrap_or_else(|_| panic!("bad key {k}"));
            self.win(|w, cx| press_key(w, key, cx));
            self.settle();
        }
    }

    /// Type into the focused field.
    pub fn type_text(&mut self, text: &str) {
        self.win(|w, cx| input_text(w, text, cx));
        self.settle();
    }

    /// Native drag-and-drop between two observed elements.
    pub fn drag_ids(&mut self, from: impl Into<ElementId>, to: impl Into<ElementId>) {
        let (a, b) = (self.bounds(from).center(), self.bounds(to).center());
        self.drag(a, b);
    }

    // ---- raw pointer -------------------------------------------------------

    fn send(&mut self, ev: gpui_kit::PlatformInput) {
        self.win(|w, cx| {
            w.dispatch_event(ev, cx);
            frame(w, cx);
        });
        self.cx.run_until_parked();
    }

    pub fn mv(&mut self, p: Point<Pixels>, pressed: Option<MouseButton>, modifiers: Modifiers) {
        self.send(
            MouseMoveEvent {
                position: p,
                pressed_button: pressed,
                modifiers,
            }
            .to_platform_input(),
        );
    }

    pub fn down(
        &mut self,
        p: Point<Pixels>,
        button: MouseButton,
        modifiers: Modifiers,
        clicks: usize,
    ) {
        self.send(
            MouseDownEvent {
                button,
                position: p,
                modifiers,
                click_count: clicks,
                first_mouse: false,
            }
            .to_platform_input(),
        );
    }

    pub fn up(
        &mut self,
        p: Point<Pixels>,
        button: MouseButton,
        modifiers: Modifiers,
        clicks: usize,
    ) {
        self.send(
            MouseUpEvent {
                button,
                position: p,
                modifiers,
                click_count: clicks,
            }
            .to_platform_input(),
        );
    }

    pub fn click_at(&mut self, p: Point<Pixels>, button: MouseButton, modifiers: Modifiers) {
        self.mv(p, None, modifiers);
        self.down(p, button, modifiers, 1);
        self.up(p, button, modifiers, 1);
        self.settle();
    }

    pub fn double_click_at(&mut self, p: Point<Pixels>) {
        let m = Modifiers::default();
        self.mv(p, None, m);
        self.down(p, MouseButton::Left, m, 1);
        self.up(p, MouseButton::Left, m, 1);
        self.down(p, MouseButton::Left, m, 2);
        self.up(p, MouseButton::Left, m, 2);
        self.settle();
    }

    /// Press at `from` with `down_mods`, move in steps holding `move_mods`, release.
    pub fn drag_with(
        &mut self,
        from: Point<Pixels>,
        to: Point<Pixels>,
        down_mods: Modifiers,
        move_mods: Modifiers,
    ) {
        self.mv(from, None, down_mods);
        self.down(from, MouseButton::Left, down_mods, 1);
        for t in 1..=8 {
            let f = t as f32 / 8.0;
            let p = point(from.x + (to.x - from.x) * f, from.y + (to.y - from.y) * f);
            self.mv(p, Some(MouseButton::Left), move_mods);
        }
        self.up(to, MouseButton::Left, move_mods, 1);
        self.settle();
    }

    pub fn drag(&mut self, from: Point<Pixels>, to: Point<Pixels>) {
        self.drag_with(from, to, Modifiers::default(), Modifiers::default());
    }

    pub fn wheel(&mut self, p: Point<Pixels>, dx: f32, dy: f32, modifiers: Modifiers) {
        self.send(
            ScrollWheelEvent {
                position: p,
                delta: ScrollDelta::Pixels(point(px(dx), px(dy))),
                modifiers,
                touch_phase: TouchPhase::Moved,
            }
            .to_platform_input(),
        );
        self.settle();
    }

    // ---- canvas coordinates ------------------------------------------------

    /// Window position of a world point.
    pub fn at(&mut self, wx: f32, wy: f32) -> Point<Pixels> {
        self.read(|a| {
            let xf = a.xf();
            point(px(xf.sx(wx)), px(xf.sy(wy)))
        })
    }

    pub fn center_of(&mut self, id: &str) -> Point<Pixels> {
        let s = self.shape(id);
        self.at(s.x + s.w / 2.0, s.y + s.h / 2.0)
    }
}
