//! Relation painting: routes, markers, labels.

use super::*;
use crate::model::*;
use crate::theme::{BLUE, PAPER, c, ca};
use gpui_kit::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_relation(
    r: &Relation,
    a: &Shape,
    b: &Shape,
    xf: &Xf,
    selected: bool,
    editing: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let route = ortho_route(a, b);
    let pts: Vec<Point<Pixels>> = route.pts.iter().map(|(x, y)| xf.p(*x, *y)).collect();
    let k = xf.k;
    let color = if selected { c(BLUE) } else { c(EDGE) };
    let sw = (if selected { 2.2 } else { 1.4 }) * k.max(0.5);
    let dash = r.kind.is_dashed().then_some((6.0 * k, 4.0 * k));
    stroke_path(window, &pts, sw, color, dash, false);
    let n = pts.len();
    let (tip, prev) = (route.pts[n - 1], route.pts[n - 2]);
    let (dx, dy) = (tip.0 - prev.0, tip.1 - prev.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let at =
        |back: f32, side: f32| xf.p(tip.0 - ux * back + nx * side, tip.1 - uy * back + ny * side);
    use RelationKind::*;
    match r.kind {
        Generalization | Realization => {
            let tri = [at(0.0, 0.0), at(11.0, 6.0), at(11.0, -6.0)];
            fill_path(window, &tri, c(PAPER));
            stroke_path(window, &tri, sw, color, None, true);
        }
        Aggregation | Composition => {
            let dia = [at(0.0, 0.0), at(7.0, 5.0), at(14.0, 0.0), at(7.0, -5.0)];
            fill_path(
                window,
                &dia,
                if r.kind == Composition {
                    color
                } else {
                    c(PAPER)
                },
            );
            stroke_path(window, &dia, sw, color, None, true);
        }
        Association | Link | FreeLine | NoteLink => {}
        _ => {
            stroke_path(
                window,
                &[at(9.0, 5.0), at(0.0, 0.0), at(9.0, -5.0)],
                sw,
                color,
                None,
                false,
            );
        }
    }
    if let Some(label) = r.label.as_deref().filter(|_| !editing) {
        let label = label.replace("<<", "«").replace(">>", "»");
        let fs = 12.0 * k;
        let lx = xf.sx(route.label.0);
        let ly = xf.sy(route.label.1) - 6.0 * k - fs * 0.95;
        let wguess = label.chars().count() as f32 * fs * 0.55;
        window.paint_quad(fill(
            Bounds::new(
                point(px(lx - wguess / 2.0 - 2.0), px(ly)),
                size(px(wguess + 4.0), px(fs * 1.2)),
            ),
            ca(PAPER, 0.85),
        ));
        text_line(
            window,
            cx,
            &label,
            lx,
            ly,
            fs,
            c(0x333333),
            "Liberation Sans",
            false,
            TAlign::Center,
        );
    }
}
