//! Action handlers bound on the root element.

use crate::app::*;
use crate::canvas::*;
use crate::commands::*;
use crate::ui::widgets::Btn;
use gpui_kit::*;

macro_rules! cmd {
    ($root:ident, $cx:ident, $a:ty, $id:expr) => {
        $root = $root.on_action($cx.listener(|this, _: &$a, w, cx| this.run($id, w, cx)));
    };
}

macro_rules! nudge {
    ($root:ident, $cx:ident, $a:ty, $m:ident, $dx:expr, $dy:expr) => {
        $root = $root.on_action($cx.listener(move |this, _: &$a, _, cx| this.$m($dx, $dy, cx)));
    };
}

impl OpenPlant {
    pub(crate) fn bind_actions(&self, mut r: Btn, cx: &mut Context<Self>) -> Btn {
        cmd!(r, cx, NewProject, "file.new");
        cmd!(r, cx, OpenFile, "file.open");
        cmd!(r, cx, SaveFile, "file.save");
        cmd!(r, cx, SaveAs, "file.save-as");
        cmd!(r, cx, Quit, "app.quit");
        cmd!(r, cx, Undo, "edit.undo");
        cmd!(r, cx, Redo, "edit.redo");
        cmd!(r, cx, Cut, "edit.cut");
        cmd!(r, cx, Copy, "edit.copy");
        cmd!(r, cx, Paste, "edit.paste");
        cmd!(r, cx, Duplicate, "edit.duplicate");
        cmd!(r, cx, DeleteSel, "edit.delete");
        cmd!(r, cx, DeleteAll, "edit.delete-all");
        cmd!(r, cx, SelectAll, "edit.select-all");
        cmd!(r, cx, SelectNone, "edit.select-none");
        cmd!(r, cx, SelectVertices, "edit.select-vertices");
        cmd!(r, cx, SelectEdges, "edit.select-edges");
        cmd!(r, cx, SelectNext, "edit.select-next");
        cmd!(r, cx, SelectPrev, "edit.select-prev");
        cmd!(r, cx, EditLabel, "edit.label");
        cmd!(r, cx, ToFront, "arrange.front");
        cmd!(r, cx, ToBack, "arrange.back");
        cmd!(r, cx, BringForward, "arrange.forward");
        cmd!(r, cx, SendBackward, "arrange.backward");
        cmd!(r, cx, CopySize, "edit.copy-size");
        cmd!(r, cx, PasteSize, "edit.paste-size");
        cmd!(r, cx, Autosize, "format.autosize");
        cmd!(r, cx, ZoomIn, "view.zoom-in");
        cmd!(r, cx, ZoomOut, "view.zoom-out");
        cmd!(r, cx, ResetView, "view.reset");
        cmd!(r, cx, FitWindow, "view.fit");
        cmd!(r, cx, ToggleGrid, "view.grid");
        cmd!(r, cx, ToggleOutline, "view.outline");
        cmd!(r, cx, ToggleFormat, "view.navigator");
        cmd!(r, cx, ToggleShapes, "view.sidebar");
        cmd!(r, cx, Palette, "view.palette");
        cmd!(r, cx, Shortcuts, "help.shortcuts");
        r = r.on_action(cx.listener(|this, _: &Cancel, w, cx| this.cancel(w, cx)));
        let g = GRID;
        nudge!(r, cx, NudgeLeft, nudge, -1.0, 0.0);
        nudge!(r, cx, NudgeRight, nudge, 1.0, 0.0);
        nudge!(r, cx, NudgeUp, nudge, 0.0, -1.0);
        nudge!(r, cx, NudgeDown, nudge, 0.0, 1.0);
        nudge!(r, cx, NudgeLeftGrid, nudge, -g, 0.0);
        nudge!(r, cx, NudgeRightGrid, nudge, g, 0.0);
        nudge!(r, cx, NudgeUpGrid, nudge, 0.0, -g);
        nudge!(r, cx, NudgeDownGrid, nudge, 0.0, g);
        nudge!(r, cx, GrowLeft, grow, -1.0, 0.0);
        nudge!(r, cx, GrowRight, grow, 1.0, 0.0);
        nudge!(r, cx, GrowUp, grow, 0.0, -1.0);
        nudge!(r, cx, GrowDown, grow, 0.0, 1.0);
        nudge!(r, cx, GrowLeftGrid, grow, -g, 0.0);
        nudge!(r, cx, GrowRightGrid, grow, g, 0.0);
        nudge!(r, cx, GrowUpGrid, grow, 0.0, -g);
        nudge!(r, cx, GrowDownGrid, grow, 0.0, g);
        r
    }

    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.take().is_some()
            || std::mem::take(&mut self.shortcuts_open)
            || self.picker_for.take().is_some()
            || self.editing.take().is_some()
        {
            window.focus(&self.canvas_focus, cx);
        } else if self.any_popup() {
            self.close_popups();
        } else {
            return self.run("edit.cancel", window, cx);
        }
        cx.notify();
    }

    pub(crate) fn focus_canvas(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.canvas_focus, cx);
    }
}
