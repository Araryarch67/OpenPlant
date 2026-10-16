//! Original line-art glyphs, painted with paths (no icon pack).
//! Coordinates are in each glyph's own viewbox and scaled to the element.

use crate::canvas::ellipse;
use crate::theme::{BLUE, PANEL, YELLOW, c};
use gpui_kit::*;

#[derive(Clone, Copy)]
pub enum Op {
    Line(f32, f32, f32, f32),
    Dashed(f32, f32, f32, f32),
    Rect(f32, f32, f32, f32),
    Circle(f32, f32, f32),
    FillCircle(f32, f32, f32),
    Ellipse(f32, f32, f32, f32),
    /// Closed polygon: (points, filled with stroke colour, hollow-filled with panel)
    Poly(&'static [(f32, f32)], Fill),
    /// Open polyline.
    Path(&'static [(f32, f32)]),
}

use Op::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Fill {
    None,
    Solid,
    Hollow,
}

pub(crate) const ARROW_HEAD: &[(f32, f32)] = &[(8.4, 3.9), (12.0, 3.5), (11.1, 7.0)];

pub(crate) const TRI: &[(f32, f32)] = &[(12.5, 2.5), (8.3, 3.1), (8.9, 7.1)];

pub(crate) const DIAMOND: &[(f32, f32)] = &[(5.5, 8.2), (8.8, 5.0), (12.0, 8.2), (8.8, 11.4)];

pub fn tool_glyph(tool: &str, edge: bool) -> (&'static [Op], u32) {
    let ops: &'static [Op] = match tool {
        "Class" => &[
            Rect(2.5, 1.5, 11.0, 11.0),
            Line(2.5, 5.0, 13.5, 5.0),
            Line(2.5, 9.0, 13.5, 9.0),
        ],
        "Interface" => &[Circle(8.0, 7.0, 4.6)],
        "Actor" => &[
            Circle(8.0, 3.0, 2.0),
            Line(8.0, 5.0, 8.0, 9.0),
            Line(4.5, 6.5, 11.5, 6.5),
            Line(8.0, 9.0, 5.5, 13.0),
            Line(8.0, 9.0, 10.5, 13.0),
        ],
        "Usecase" => &[Ellipse(8.0, 7.0, 6.5, 5.0)],
        "Database" | "Storage" => &[
            Ellipse(8.0, 3.5, 5.5, 2.0),
            Line(2.5, 3.5, 2.5, 10.5),
            Line(13.5, 3.5, 13.5, 10.5),
            Path(&[
                (2.5, 10.5),
                (4.0, 11.9),
                (8.0, 12.5),
                (12.0, 11.9),
                (13.5, 10.5),
            ]),
        ],
        "Enumeration" => &[
            Rect(2.5, 1.5, 11.0, 11.0),
            Line(5.0, 6.0, 11.0, 6.0),
            Line(5.0, 8.5, 11.0, 8.5),
            Line(5.0, 11.0, 9.0, 11.0),
        ],
        "Frame" => &[
            Rect(1.5, 2.5, 13.0, 9.5),
            Path(&[(1.5, 2.5), (6.5, 2.5), (7.7, 4.5), (14.0, 4.5)]),
        ],
        "Package" | "Folder" => &[Poly(
            &[
                (1.5, 11.5),
                (1.5, 4.0),
                (6.0, 4.0),
                (7.5, 6.0),
                (14.5, 6.0),
                (14.5, 11.5),
            ],
            Fill::None,
        )],
        "Object" => &[Rect(2.5, 2.0, 11.0, 10.0), Line(2.5, 5.5, 13.5, 5.5)],
        "Note" => &[
            Poly(
                &[
                    (2.5, 1.5),
                    (8.5, 1.5),
                    (11.5, 4.5),
                    (11.5, 12.5),
                    (2.5, 12.5),
                ],
                Fill::None,
            ),
            Path(&[(8.5, 1.5), (8.5, 4.5), (11.5, 4.5)]),
        ],
        "Rectangle" | "Artifact" | "Node" => &[Rect(2.5, 2.5, 11.0, 9.0)],
        "Queue" => &[
            Ellipse(3.5, 7.0, 2.0, 4.5),
            Line(3.5, 2.5, 12.5, 2.5),
            Line(3.5, 11.5, 12.5, 11.5),
            Ellipse(12.5, 7.0, 2.0, 4.5),
        ],
        "Cloud" => &[Path(&[
            (4.0, 11.0),
            (2.0, 9.6),
            (2.4, 7.2),
            (4.4, 6.4),
            (5.4, 3.8),
            (8.4, 3.0),
            (10.6, 4.6),
            (13.0, 5.0),
            (14.2, 7.6),
            (13.4, 10.4),
            (11.4, 11.0),
            (4.0, 11.0),
        ])],
        "Component" => &[
            Rect(3.5, 2.0, 10.0, 10.0),
            Rect(1.8, 4.0, 3.4, 2.0),
            Rect(1.8, 8.0, 3.4, 2.0),
        ],
        "Association" => &[Line(1.5, 11.5, 12.0, 3.5)],
        "Directed Association" => &[Line(1.5, 11.5, 12.0, 3.5), Path(ARROW_HEAD)],
        "Aggregation" => &[Line(1.5, 11.5, 6.0, 7.6), Poly(DIAMOND, Fill::Hollow)],
        "Composition" => &[Line(1.5, 11.5, 6.0, 7.6), Poly(DIAMOND, Fill::Solid)],
        "Dependency" => &[Dashed(1.5, 11.5, 12.0, 3.5), Path(ARROW_HEAD)],
        "Generalization" => &[Line(1.5, 11.5, 8.6, 6.0), Poly(TRI, Fill::Hollow)],
        "Realization" => &[Dashed(1.5, 11.5, 9.0, 5.5), Poly(TRI, Fill::Hollow)],
        "Note Link" => &[Dashed(1.5, 11.5, 12.0, 3.5)],
        _ if edge => &[Line(1.5, 11.5, 12.0, 3.5)],
        _ => &[Rect(2.5, 2.5, 11.0, 9.0)],
    };
    (ops, if edge { BLUE } else { YELLOW })
}

/// Paint `ops` (viewbox `vw`x`vh`) into `bounds`.
pub fn paint_ops(
    ops: &[Op],
    view: (f32, f32),
    bounds: Bounds<Pixels>,
    color: Hsla,
    stroke: f32,
    window: &mut Window,
) {
    let ox = f32::from(bounds.origin.x);
    let oy = f32::from(bounds.origin.y);
    let k = (f32::from(bounds.size.width) / view.0).min(f32::from(bounds.size.height) / view.1);
    let p = |x: f32, y: f32| point(px(ox + x * k), px(oy + y * k));
    let sw = px((stroke * k).max(1.0));
    for op in ops {
        match *op {
            Line(x1, y1, x2, y2) => {
                let mut b = PathBuilder::stroke(sw);
                b.move_to(p(x1, y1));
                b.line_to(p(x2, y2));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            Dashed(x1, y1, x2, y2) => {
                let mut b = PathBuilder::stroke(sw).dash_array(&[px(2.4 * k), px(2.0 * k)]);
                b.move_to(p(x1, y1));
                b.line_to(p(x2, y2));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            Rect(x, y, w, h) => {
                window.paint_quad(quad(
                    Bounds::new(p(x, y), size(px(w * k), px(h * k))),
                    px(1.0),
                    transparent_black(),
                    sw,
                    color,
                    BorderStyle::Solid,
                ));
            }
            Circle(cx, cy, r) | FillCircle(cx, cy, r) => {
                let filled = matches!(op, FillCircle(..));
                ellipse(
                    window,
                    p(cx, cy),
                    r * k,
                    r * k,
                    color,
                    if filled { None } else { Some(sw) },
                );
            }
            Ellipse(cx, cy, rx, ry) => ellipse(window, p(cx, cy), rx * k, ry * k, color, Some(sw)),
            Poly(pts, fill) => {
                if fill != Fill::None {
                    let mut b = PathBuilder::fill();
                    poly(&mut b, pts, &p, true);
                    if let Ok(path) = b.build() {
                        window.paint_path(path, if fill == Fill::Solid { color } else { c(PANEL) });
                    }
                }
                let mut b = PathBuilder::stroke(sw);
                poly(&mut b, pts, &p, true);
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            Path(pts) => {
                let mut b = PathBuilder::stroke(sw);
                poly(&mut b, pts, &p, false);
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
        }
    }
}

pub(crate) fn poly(
    b: &mut PathBuilder,
    pts: &[(f32, f32)],
    p: &impl Fn(f32, f32) -> Point<Pixels>,
    close: bool,
) {
    for (i, (x, y)) in pts.iter().enumerate() {
        if i == 0 {
            b.move_to(p(*x, *y));
        } else {
            b.line_to(p(*x, *y));
        }
    }
    if close {
        b.close();
    }
}

/// An element that paints a glyph.
pub fn glyph_el(
    ops: &'static [Op],
    view: (f32, f32),
    color: Hsla,
    w: f32,
    h: f32,
    stroke: f32,
) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| paint_ops(ops, view, bounds, color, stroke, window),
    )
    .w(px(w))
    .h(px(h))
    .flex_none()
}

pub fn tool_icon(tool: &str, edge: bool) -> impl IntoElement {
    let (ops, col) = tool_glyph(tool, edge);
    glyph_el(ops, (16.0, 14.0), c(col), 16.0, 14.0, 1.3)
}

pub const CHEV_DOWN: &[Op] = &[Path(&[(2.0, 3.5), (5.0, 6.5), (8.0, 3.5)])];

pub const CHEV_UP: &[Op] = &[Path(&[(2.0, 6.5), (5.0, 3.5), (8.0, 6.5)])];

pub const CHEV_RIGHT: &[Op] = &[Path(&[(3.5, 2.0), (6.5, 5.0), (3.5, 8.0)])];

pub const CLOSE: &[Op] = &[Line(2.5, 2.5, 9.5, 9.5), Line(9.5, 2.5, 2.5, 9.5)];

pub const MAGNIFIER: &[Op] = &[Circle(6.0, 6.0, 4.2), Line(9.2, 9.2, 13.0, 13.0)];

pub fn chevron(dir: &str, color: Hsla) -> impl IntoElement {
    let ops = match dir {
        "up" => CHEV_UP,
        "right" => CHEV_RIGHT,
        _ => CHEV_DOWN,
    };
    glyph_el(ops, (10.0, 10.0), color, 10.0, 10.0, 1.6)
}

pub fn close_x(color: Hsla) -> impl IntoElement {
    glyph_el(CLOSE, (12.0, 12.0), color, 12.0, 12.0, 1.6)
}

pub fn magnifier(color: Hsla) -> impl IntoElement {
    glyph_el(MAGNIFIER, (14.0, 14.0), color, 14.0, 14.0, 1.6)
}

/// Select tool cursor: white arrow with dark outline.
pub fn cursor_arrow() -> impl IntoElement {
    canvas(
        |_, _, _| {},
        |bounds, _, window, _| {
            let ox = f32::from(bounds.origin.x);
            let oy = f32::from(bounds.origin.y);
            let k = f32::from(bounds.size.height) / 26.0;
            let pts = [
                (5.0, 3.0),
                (5.0, 19.5),
                (9.5, 15.3),
                (12.3, 21.8),
                (14.8, 20.6),
                (12.0, 14.1),
                (17.2, 14.1),
            ];
            let p = |x: f32, y: f32| point(px(ox + x * k), px(oy + y * k));
            let mut f = PathBuilder::fill();
            poly(&mut f, &pts, &p, true);
            if let Ok(path) = f.build() {
                window.paint_path(path, c(0xf4f4f4));
            }
            let mut s = PathBuilder::stroke(px(1.4 * k));
            poly(&mut s, &pts, &p, true);
            if let Ok(path) = s.build() {
                window.paint_path(path, c(0x141b1e));
            }
        },
    )
    .w(px(15.4))
    .h(px(20.0))
}

/// Tool strip glyphs (18x18 viewbox).
pub fn strip_glyph(id: &str) -> &'static [Op] {
    match id {
        "ext" => &[
            Rect(3.5, 4.5, 11.0, 10.0),
            Circle(9.0, 4.5, 2.2),
            FillCircle(14.5, 9.5, 1.8),
        ],
        "diagrams" => &[
            Rect(3.0, 3.0, 5.0, 5.0),
            Rect(10.0, 3.0, 5.0, 5.0),
            Rect(3.0, 10.0, 5.0, 5.0),
            Rect(10.0, 10.0, 5.0, 5.0),
        ],
        "model" => &[
            Rect(5.0, 5.0, 10.0, 10.0),
            Rect(3.5, 3.5, 10.0, 10.0),
            Rect(2.0, 2.0, 10.0, 10.0),
        ],
        "nav" => &[
            Circle(9.0, 9.0, 5.0),
            Line(9.0, 1.5, 9.0, 4.5),
            Line(9.0, 13.5, 9.0, 16.5),
            Line(1.5, 9.0, 4.5, 9.0),
            Line(13.5, 9.0, 16.5, 9.0),
            FillCircle(9.0, 9.0, 1.4),
        ],
        "share" => &[
            Line(5.0, 9.0, 12.0, 4.5),
            Line(5.0, 9.0, 12.0, 13.5),
            FillCircle(4.0, 9.0, 2.0),
            FillCircle(13.5, 4.0, 2.0),
            FillCircle(13.5, 14.0, 2.0),
        ],
        "puml" => &[
            Rect(3.0, 3.0, 12.0, 12.0),
            Path(&[(6.5, 7.0), (4.8, 9.0), (6.5, 11.0)]),
            Path(&[(11.5, 7.0), (13.2, 9.0), (11.5, 11.0)]),
            Line(10.0, 6.5, 8.0, 11.5),
        ],
        "ai" => &[
            Poly(
                &[
                    (9.0, 2.5),
                    (10.6, 7.4),
                    (15.5, 9.0),
                    (10.6, 10.6),
                    (9.0, 15.5),
                    (7.4, 10.6),
                    (2.5, 9.0),
                    (7.4, 7.4),
                ],
                Fill::None,
            ),
            Circle(14.0, 14.0, 1.6),
        ],
        _ => &[Rect(3.0, 3.0, 12.0, 12.0)],
    }
}

pub fn diagram_doc_icon() -> impl IntoElement {
    glyph_el(
        &[
            Rect(2.5, 1.5, 9.0, 11.0),
            Path(&[(8.5, 1.5), (8.5, 4.5), (11.5, 4.5)]),
        ],
        (14.0, 14.0),
        c(BLUE),
        14.0,
        14.0,
        1.4,
    )
}

/// Explorer node-kind icons (14x14).
pub fn kind_icon(kind: crate::model::NodeKind) -> impl IntoElement {
    use crate::model::NodeKind::*;
    let (ops, col): (&'static [Op], u32) = match kind {
        Project => (
            &[
                Rect(2.5, 4.0, 7.0, 7.0),
                Path(&[(2.5, 4.0), (4.5, 1.5), (11.5, 1.5), (9.5, 4.0)]),
                Path(&[(9.5, 4.0), (11.5, 1.5), (11.5, 8.5), (9.5, 11.0)]),
            ],
            0xe57474,
        ),
        Model => (
            &[Poly(
                &[
                    (1.5, 11.5),
                    (1.5, 2.5),
                    (6.5, 2.5),
                    (7.5, 4.0),
                    (12.5, 4.0),
                    (12.5, 11.5),
                ],
                Fill::Solid,
            )],
            0xe5c76b,
        ),
        Diagram => (
            &[
                Rect(3.0, 1.5, 8.0, 11.0),
                Path(&[(8.5, 1.5), (8.5, 4.5), (11.0, 4.5)]),
                Line(5.0, 7.0, 9.0, 7.0),
                Line(5.0, 9.5, 9.0, 9.5),
            ],
            0x67b0e8,
        ),
        Class | Interface => (
            &[Rect(2.5, 2.5, 9.0, 9.0), Line(2.5, 5.5, 11.5, 5.5)],
            0xe5c76b,
        ),
        Package => (
            &[Poly(
                &[
                    (1.5, 11.5),
                    (1.5, 4.0),
                    (6.0, 4.0),
                    (7.5, 6.0),
                    (12.5, 6.0),
                    (12.5, 11.5),
                ],
                Fill::None,
            )],
            0xe5c76b,
        ),
    };
    glyph_el(ops, (14.0, 14.0), c(col), 14.0, 14.0, 1.3)
}
