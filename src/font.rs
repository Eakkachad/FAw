//! Embedded font (`katSVG Fonts`).
//!
//! Embeds a WOFF2 subset of Inter (SIL Open Font License) as base64 so SVG
//! output renders identically offline with zero network dependency (G6).
//! PDF output continues to use the base-14 Helvetica (guaranteed by the PDF
//! spec), which requires no embedding.

/// Base64 of `assets/inter-regular.woff2` (Inter, OFL-1.1).
const INTER_WOFF2_B64: &str = include_str!("../assets/inter_regular.b64");

/// Base64 of `assets/noto-sans-thai.ttf` (Noto Sans Thai, OFL-1.1).
/// Embedded only when the spec contains non-ASCII text (Thai), so most SVG
/// output stays compact.
const NOTO_THAI_TTF_B64: &str = include_str!("../assets/noto_thai.b64");

/// Renders an SVG `<style>` block declaring the embedded Inter font and,
/// when `has_cjk` (Thai/CJK text present), a Noto Sans Thai fallback.
/// Deterministic and fully offline.
pub fn font_style_block(has_thai: bool) -> String {
    let inter = format!(
        "@font-face {{ font-family: 'Inter'; src: url(data:font/woff2;base64,{b64}) format('woff2'); font-weight: 400; font-style: normal; }}\n",
        b64 = INTER_WOFF2_B64
    );
    let thai = if has_thai {
        format!(
            "@font-face {{ font-family: 'Noto Sans Thai'; src: url(data:font/truetype;base64,{b64}) format('truetype'); font-weight: 400; font-style: normal; }}\n",
            b64 = NOTO_THAI_TTF_B64
        )
    } else {
        String::new()
    };
    format!("{inter}{thai}")
}

/// The font-family stack used across outputs (Inter first, Thai fallback).
pub fn font_stack(has_thai: bool) -> String {
    if has_thai {
        "'Inter', 'Noto Sans Thai', 'Segoe UI', system-ui, -apple-system, sans-serif".to_string()
    } else {
        "'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif".to_string()
    }
}

/// Detect whether a string contains non-ASCII (Thai/CJK) characters.
pub fn has_non_ascii(text: &str) -> bool {
    text.chars().any(|c| !c.is_ascii())
}
