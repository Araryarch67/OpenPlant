//! Command palette state.

use super::*;
use crate::commands::*;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

impl OpenPlant {
    pub fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.is_some() {
            self.palette = None;
            window.focus(&self.canvas_focus, cx);
            cx.notify();
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Type a command..."));
        let sub = cx.subscribe_in(
            &input,
            window,
            |this, inp, ev: &InputEvent, window, cx| match ev {
                InputEvent::Change => {
                    if let Some(p) = &mut this.palette {
                        p.idx = 0;
                    }
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => {
                    let q = inp.read(cx).value().to_lowercase();
                    let idx = this.palette.as_ref().map(|p| p.idx).unwrap_or(0);
                    let list = palette_matches(&q);
                    this.palette = None;
                    if let Some(c) = list.get(idx) {
                        this.run(c.id, window, cx);
                    }
                    cx.notify();
                }
                _ => {}
            },
        );
        input.update(cx, |s, cx| s.focus(window, cx));
        self.palette = Some(PaletteState {
            input,
            idx: 0,
            _sub: sub,
        });
        cx.notify();
    }
}
