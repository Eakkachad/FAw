//! Embedded font (`katSVG Fonts`).
//!
//! Embeds a WOFF2 subset of Inter (SIL Open Font License) as base64 so SVG
//! output renders identically offline with zero network dependency (G6).
//! PDF output continues to use the base-14 Helvetica (guaranteed by the PDF
//! spec), which requires no embedding.

/// Base64 of `assets/inter-regular.woff2` (Inter, OFL-1.1).
const INTER_WOFF2_B64: &str = include_str!("../assets/inter_regular.b64");

/// Renders an SVG `<style>` block that declares the embedded Inter font and
/// applies it to all text. Deterministic and fully offline.
pub fn font_style_block() -> String {
    format!(
        "@font-face {{ font-family: 'Inter'; src: url(data:font/woff2;base64,{b64}) format('woff2'); font-weight: 400; font-style: normal; }}\n",
        b64 = INTER_WOFF2_B64
    )
}

/// The font-family stack used across outputs.
pub const FONT_STACK: &str = "'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif";
