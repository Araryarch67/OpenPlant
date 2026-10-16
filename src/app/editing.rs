//! Inline label editing on the canvas.

use super::*;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

impl OpenPlant {
    pub fn begin_edit(&mut self, target: EditTarget, window: &mut Window, cx: &mut Context<Self>) {
        let doc = self.wb.doc();
        let text = match &target {
            EditTarget::Shape(id) => doc.shape(id).map(|s| s.label.clone()),
            EditTarget::Relation(id) => doc
                .relations
                .iter()
                .find(|r| &r.id == id)
                .map(|r| r.label.clone().unwrap_or_default()),
        };
        let Some(text) = text else { return };
        let input =
            cx.new(|cx| InputState::new(window, cx).default_value(text.replace('\n', "\\n")));
        let sub = cx.subscribe_in(
            &input,
            window,
            |this, _, ev: &InputEvent, window, cx| match ev {
                InputEvent::PressEnter { .. } | InputEvent::Blur => this.commit_edit(window, cx),
                _ => {}
            },
        );
        input.update(cx, |s, cx| {
            s.focus(window, cx);
            s.select_all(window, cx);
        });
        self.editing = Some(Editing {
            target,
            input,
            _sub: sub,
        });
        cx.notify();
    }

    pub fn commit_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ed) = self.editing.take() else {
            return;
        };
        let value = ed.input.read(cx).value().to_string().replace("\\n", "\n");
        self.wb.checkpoint();
        match &ed.target {
            EditTarget::Shape(id) if !value.trim().is_empty() => self.wb.rename_node(id, &value),
            EditTarget::Relation(id) => self.wb.set_relation_label(id, &value),
            _ => {}
        }
        window.focus(&self.canvas_focus, cx);
        cx.notify();
    }
}
