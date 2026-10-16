//! Standalone SVG of the active diagram (same look as the canvas) and PNG
//! rasterised from it with resvg.

use crate::canvas::{bbox, is_container, ortho_route, straight_route};
use crate::model::*;
use std::fmt::Write;

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn css_font(css: &str) -> String {
    esc(css)
}

pub fn to_svg(doc: &DiagramDoc) -> String {
    let (x0, y0, x1, y1) = bbox(doc).unwrap_or((0.0, 0.0, 400.0, 300.0));
    let title_h = doc
        .title
        .as_ref()
        .map(|t| t.split('\n').count() as f32 * 19.0 + 14.0)
        .unwrap_or(0.0);
    let (ox, oy) = (x0 - 30.0, y0 - 30.0 - title_h);
    let (w, h) = (x1 - x0 + 60.0, y1 - y0 + 60.0 + title_h);
    let mut s = String::new();
    let _ = write!(
        s,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="{ox:.0} {oy:.0} {w:.0} {h:.0}">
<defs>
<marker id="open" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1l8 4-8 4" fill="none" stroke="#555" stroke-width="1.4"/></marker>
<marker id="tri" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M1 1l8 4-8 4z" fill="#f4f1e8" stroke="#555" stroke-width="1.4"/></marker>
<marker id="dia" viewBox="0 0 12 12" refX="11" refY="6" markerWidth="9" markerHeight="9" orient="auto-start-reverse"><path d="M6 1l5 5-5 5-5-5z" fill="#f4f1e8" stroke="#555" stroke-width="1.4"/></marker>
<marker id="diaf" viewBox="0 0 12 12" refX="11" refY="6" markerWidth="9" markerHeight="9" orient="auto-start-reverse"><path d="M6 1l5 5-5 5-5-5z" fill="#555" stroke="#555" stroke-width="1.2"/></marker>
</defs>
<rect x="{ox:.0}" y="{oy:.0}" width="{w:.0}" height="{h:.0}" fill="#f4f1e8"/>
"##
    );
    if let Some(t) = &doc.title {
        for (i, ln) in t.split('\n').enumerate() {
            let _ = writeln!(
                s,
                r##"<text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="15" font-weight="bold" fill="#111">{}</text>"##,
                (x0 + x1) / 2.0,
                oy + 30.0 + 15.0 + i as f32 * 19.0,
                esc(ln)
            );
        }
    }
    let mut ordered: Vec<&Shape> = doc
        .shapes
        .iter()
        .filter(|s| is_container(doc, &s.id))
        .collect();
    ordered.sort_by_key(|s| s.parent_id.is_some());
    for sh in &ordered {
        shape_svg(&mut s, sh);
    }
    for r in &doc.relations {
        let (Some(a), Some(b)) = (doc.shape(&r.from), doc.shape(&r.to)) else {
            continue;
        };
        let route = ortho_route(a, b);
        let d: String = route
            .pts
            .iter()
            .enumerate()
            .map(|(i, (x, y))| format!("{}{x:.0} {y:.0}", if i == 0 { "M" } else { "L" }))
            .collect();
        use RelationKind::*;
        let marker = match r.kind {
            Generalization | Realization => " marker-end=\"url(#tri)\"",
            Aggregation => " marker-end=\"url(#dia)\"",
            Composition => " marker-end=\"url(#diaf)\"",
            Association | Link | FreeLine | NoteLink => "",
            _ => " marker-end=\"url(#open)\"",
        };
        let dash = if r.kind.is_dashed() {
            " stroke-dasharray=\"6 4\""
        } else {
            ""
        };
        let _ = writeln!(
            s,
            r##"<path d="{d}" fill="none" stroke="#555" stroke-width="1.4"{dash}{marker}/>"##
        );
        if let Some(l) = &r.label {
            let _ = writeln!(
                s,
                r##"<text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="#333" stroke="#f4f1e8" stroke-width="3" paint-order="stroke">{}</text>"##,
                route.label.0,
                route.label.1 - 6.0,
                esc(&l.replace("<<", "«").replace(">>", "»"))
            );
        }
    }
    for sh in &doc.shapes {
        if let Some(t) = sh.note_of.as_ref().and_then(|t| doc.shape(t)) {
            let p = straight_route(sh, t);
            let _ = writeln!(
                s,
                r##"<path d="M{:.0} {:.0}L{:.0} {:.0}" stroke="#777" stroke-width="1.2" stroke-dasharray="5 4"/>"##,
                p[0].0, p[0].1, p[1].0, p[1].1
            );
        }
    }
    for sh in doc.shapes.iter().filter(|s| !is_container(doc, &s.id)) {
        shape_svg(&mut s, sh);
    }
    s.push_str("</svg>\n");
    s
}

pub(crate) fn shape_svg(s: &mut String, sh: &Shape) {
    let st = sh.style.clone().unwrap_or_default();
    let fill = esc(st
        .fill
        .as_deref()
        .unwrap_or(if sh.kind() == ShapeKind::Note {
            "#fefce8"
        } else {
            DEFAULT_FILL
        }));
    let stroke = esc(st.stroke.as_deref().unwrap_or(DEFAULT_STROKE));
    let text = esc(st.text.as_deref().unwrap_or(DEFAULT_TEXT));
    let font = css_font(st.font.as_deref().unwrap_or(DEFAULT_FONT));
    let fs = st.font_size.unwrap_or(DEFAULT_FONT_SIZE);
    let dash = match st.dash {
        Some(Dash::Dashed) => " stroke-dasharray=\"6 4\"",
        Some(Dash::Dotted) => " stroke-dasharray=\"2 3\"",
        _ => "",
    };
    let (w, h) = (sh.w, sh.h);
    let lines: Vec<&str> = sh.label.split('\n').collect();
    let n = lines.len() as f32;
    let _ = write!(s, r#"<g transform="translate({:.0},{:.0})">"#, sh.x, sh.y);
    let base = r#"stroke-width="1.5""#;
    let label = |s: &mut String, x: f32, y0: f32, anchor: &str, size: f32| {
        for (i, ln) in lines.iter().enumerate() {
            let _ = write!(
                s,
                r#"<text x="{x:.1}" y="{:.1}" text-anchor="{anchor}" font-family="{font}" font-size="{size}" fill="{text}">{}</text>"#,
                y0 + i as f32 * 15.0,
                esc(ln)
            );
        }
    };
    let center = h / 2.0 - ((n - 1.0) * 15.0) / 2.0 + 5.0;
    match sh.kind() {
        ShapeKind::Actor => {
            let cx = w / 2.0;
            let _ = write!(
                s,
                r#"<circle cx="{cx}" cy="26" r="9" fill="{fill}" stroke="{stroke}" {base}/><path d="M{cx} 35V58M{} 43H{}M{cx} 58L{} 78M{cx} 58L{} 78" stroke="{stroke}" {base} fill="none"/>"#,
                cx - 14.0,
                cx + 14.0,
                cx - 11.0,
                cx + 11.0
            );
            label(s, cx, 92.0, "middle", fs);
        }
        ShapeKind::Database => {
            let cx = w / 2.0;
            let rx = w / 2.0 - 10.0;
            let _ = write!(
                s,
                r#"<path d="M{} 18v{} a{rx} 10 0 0 0 {} 0v-{}" fill="{fill}" stroke="{stroke}" {base}{dash}/><ellipse cx="{cx}" cy="18" rx="{rx}" ry="10" fill="{fill}" stroke="{stroke}" {base}/>"#,
                cx - rx,
                h - 42.0,
                rx * 2.0,
                h - 42.0
            );
            label(s, cx, 48.0, "middle", fs);
        }
        ShapeKind::Package => {
            let _ = write!(
                s,
                r#"<rect x="8" y="4" width="56" height="16" fill="{fill}" stroke="{stroke}" {base}/><rect x="8" y="14" width="{}" height="{}" fill="{fill}" stroke="{stroke}" {base}{dash}/>"#,
                w - 16.0,
                h - 22.0
            );
            label(s, w / 2.0, 44.0, "middle", fs);
        }
        ShapeKind::Note => {
            let _ = write!(
                s,
                r#"<path d="M0 0H{}L{w} 14V{h}H0Z" fill="{fill}" stroke="{stroke}" {base}{dash}/><path d="M{} 0V14H{w}" fill="none" stroke="{stroke}"/>"#,
                w - 14.0,
                w - 14.0
            );
            label(s, 10.0, 22.0, "start", fs - 1.0);
        }
        ShapeKind::Component => {
            let _ = write!(
                s,
                r#"<rect width="{w}" height="{h}" fill="{fill}" stroke="{stroke}" {base}{dash}/><rect x="6" y="14" width="12" height="8" fill="{fill}" stroke="{stroke}"/><rect x="6" y="28" width="12" height="8" fill="{fill}" stroke="{stroke}"/>"#
            );
            label(s, w / 2.0 + 6.0, 22.0, "middle", fs);
        }
        ShapeKind::Usecase => {
            let _ = write!(
                s,
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{fill}" stroke="{stroke}" {base}{dash}/>"#,
                w / 2.0,
                h / 2.0,
                w / 2.0 - 2.0,
                h / 2.0 - 2.0
            );
            label(s, w / 2.0, center, "middle", fs);
        }
        ShapeKind::Box => {
            let _ = write!(
                s,
                r#"<rect width="{w}" height="{h}" fill="{fill}" stroke="{stroke}" {base}{dash}/>"#
            );
            label(s, w / 2.0, center, "middle", fs);
        }
        ShapeKind::Class | ShapeKind::Interface => {
            let _ = write!(
                s,
                r#"<rect width="{w}" height="{h}" fill="{fill}" stroke="{stroke}" {base}{dash}/><line x1="0" y1="30" x2="{w}" y2="30" stroke="{stroke}"/><line x1="0" y1="{}" x2="{w}" y2="{}" stroke="{stroke}"/>"#,
                h - 28.0,
                h - 28.0
            );
            label(s, w / 2.0, 20.0, "middle", fs);
        }
    }
    s.push_str("</g>\n");
}

pub fn to_png(doc: &DiagramDoc) -> Result<Vec<u8>, String> {
    let svg = to_svg(doc);
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(&svg, &opt).map_err(|e| e.to_string())?;
    let scale = 2.0;
    let size = tree
        .size()
        .to_int_size()
        .scale_by(scale)
        .ok_or("diagram too large")?;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(size.width(), size.height()).ok_or("diagram too large")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plantuml::parse_plantuml;

    fn sample_doc() -> DiagramDoc {
        let p = parse_plantuml(include_str!(
            "../../sample-puml/enterprise-library-management.puml"
        ));
        DiagramDoc {
            id: "d".into(),
            name: "Sample".into(),
            kind: "Class".into(),
            shapes: p.shapes,
            relations: p.relations,
            preamble: Some(p.preamble),
            legend: Some(p.legend),
            title: p.title_text,
        }
    }

    #[test]
    fn svg_is_well_formed_and_complete() {
        let doc = sample_doc();
        let svg = to_svg(&doc);
        let tree =
            resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default()).expect("valid svg");
        assert!(tree.size().width() > 500.0);
        assert!(svg.contains("Circulation Service"));
        assert!(svg.contains("«include»"));
        assert_eq!(svg.matches("<g transform").count(), doc.shapes.len());
    }

    #[test]
    fn png_has_signature() {
        let png = to_png(&sample_doc()).expect("png renders");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert!(png.len() > 10_000);
    }
}
