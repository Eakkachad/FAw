//! Embedded Thai font for PDF Type0 embedding (`katSVG PDF Font`).
//!
//! F2: exposes the embedded Noto Sans Thai TTF plus a glyph-ID lookup backed by
//! `ab_glyph` (robust across variable fonts — no hand-rolled cmap parsing). PDF
//! uses Identity-H encoding with CIDToGIDMap=Identity, so CID = glyph ID.

use ab_glyph::Font;

const NOTO_THAI_TTF: &[u8] = include_bytes!("../assets/noto-sans-thai.ttf");

/// Font metrics (from the Noto Sans Thai hhea/head tables).
pub const FONT_ASCENT: i32 = 1069;
pub const FONT_DESCENT: i32 = -293;
pub const FONT_CAPHEIGHT: i32 = 700;
pub const FONT_BBOX: [i32; 4] = [-662, -293, 1336, 1069];

/// The embedded Noto Sans Thai TTF bytes (for the FontFile2 stream).
pub fn font_file_bytes() -> &'static [u8] {
    NOTO_THAI_TTF
}

fn font() -> &'static ab_glyph::FontArc {
    static FONT: std::sync::OnceLock<ab_glyph::FontArc> = std::sync::OnceLock::new();
    FONT.get_or_init(|| {
        ab_glyph::FontArc::try_from_slice(NOTO_THAI_TTF)
            .expect("embedded Noto Sans Thai TTF must parse")
    })
}

/// Encode `text` as a PDF hex CID string (Identity-H; CID = glyph ID).
/// Missing glyphs map to glyph 0 (`.notdef`).
pub fn encode_cid_hex(text: &str) -> String {
    let f = font();
    let mut out = String::with_capacity(text.len() * 4 + 2);
    out.push('<');
    for c in text.chars() {
        let gid = f.glyph_id(c).0;
        out.push_str(&format!("{gid:04X}"));
    }
    out.push('>');
    out
}
