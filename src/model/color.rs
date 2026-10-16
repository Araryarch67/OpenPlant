//! Colour parsing and formatting.

/// Parse "#rgb" / "#rrggbb" / "#rrggbbaa" or a few CSS names into 0xRRGGBB.
pub fn parse_color(s: &str) -> Option<u32> {
    let t = s.trim();
    let named = match t.to_ascii_lowercase().as_str() {
        "white" => Some(0xffffff),
        "black" => Some(0x000000),
        "red" => Some(0xff0000),
        "green" => Some(0x008000),
        "blue" => Some(0x0000ff),
        "yellow" => Some(0xffff00),
        "gray" | "grey" => Some(0x808080),
        "lightgray" | "lightgrey" => Some(0xd3d3d3),
        "lightblue" => Some(0xadd8e6),
        "lightyellow" => Some(0xffffe0),
        "lightgreen" => Some(0x90ee90),
        "orange" => Some(0xffa500),
        "pink" => Some(0xffc0cb),
        "transparent" => None,
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let hex = t.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            let (r, g, b) = ((v >> 8) & 0xf, (v >> 4) & 0xf, v & 0xf);
            Some((r * 17) << 16 | (g * 17) << 8 | (b * 17))
        }
        6 => u32::from_str_radix(hex, 16).ok(),
        8 => u32::from_str_radix(&hex[..6], 16).ok(),
        _ => None,
    }
}

pub fn color_hex(c: u32) -> String {
    format!("#{c:06x}")
}
