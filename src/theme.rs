//! Everblush palette (original theme).

use gpui_kit::component::theme::{Theme, ThemeMode};
use gpui_kit::{App, Hsla, px, rgb};

pub const DEEP: u32 = 0x141b1e;
pub const PANEL: u32 = 0x1d2529;
pub const RAISED: u32 = 0x252e33;
pub const INPUTBG: u32 = 0x101619;
pub const HOVER: u32 = 0x2e393e;
pub const FG: u32 = 0xdadada;
pub const MUTED: u32 = 0x8b989c;
pub const LINE: u32 = 0x2d363b;
pub const RED: u32 = 0xe57474;
pub const GREEN: u32 = 0x8ccf7e;
pub const YELLOW: u32 = 0xe5c76b;
pub const BLUE: u32 = 0x67b0e8;
pub const PAPER: u32 = 0xf4f1e8;
pub const PAPERGRID: u32 = 0xddd8c8;
pub const CHIP: u32 = 0xc9c2ae;

pub const UI_FONT: &str = "Inter";
pub const MONO_FONT: &str = "Iosevka Nerd Font";

pub fn c(v: u32) -> Hsla {
    rgb(v).into()
}

pub fn ca(v: u32, alpha: f32) -> Hsla {
    let mut h: Hsla = rgb(v).into();
    h.a = alpha;
    h
}

/// Selected-row tint (`bg-everblue/20`).
pub fn sel_bg() -> Hsla {
    ca(BLUE, 0.2)
}

/// Map stored CSS-style font stacks onto fonts present on Linux.
/// Arial/Liberation Sans share metrics, so saved layouts keep their fit.
pub fn canvas_font(css: &str) -> &'static str {
    let first = css
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_lowercase();
    match first.as_str() {
        "georgia" => "DejaVu Serif",
        "times new roman" | "serif" => "Liberation Serif",
        "courier new" | "monospace" | "monospaced" => "Liberation Mono",
        "verdana" => "DejaVu Sans",
        _ => "Liberation Sans",
    }
}

pub fn apply(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    let t = Theme::global_mut(cx);
    t.font_family = UI_FONT.into();
    t.font_size = px(13.);
    t.mono_font_family = MONO_FONT.into();
    t.radius = px(6.);
    t.radius_lg = px(8.);
    t.shadow = false;
    let col = &mut t.colors;
    col.background = c(PANEL);
    col.foreground = c(FG);
    col.border = c(LINE);
    col.input = c(LINE);
    col.ring = c(BLUE);
    col.caret = c(FG);
    col.selection = ca(BLUE, 0.35);
    col.muted = c(RAISED);
    col.muted_foreground = c(MUTED);
    col.popover = c(RAISED);
    col.popover_foreground = c(FG);
    col.accent = c(HOVER);
    col.accent_foreground = c(FG);
    col.primary = c(BLUE);
    col.primary_hover = c(BLUE);
    col.primary_active = c(BLUE);
    col.primary_foreground = c(0x10161a);
    col.secondary = c(RAISED);
    col.secondary_hover = c(HOVER);
    col.secondary_active = c(HOVER);
    col.secondary_foreground = c(FG);
    col.scrollbar = ca(PANEL, 0.0);
    col.scrollbar_thumb = c(HOVER);
    col.scrollbar_thumb_hover = c(MUTED);
    col.list_hover = c(HOVER);
    col.list_active = sel_bg();
}
