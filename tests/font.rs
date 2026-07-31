//! P4 gate tests: embedded Inter font (offline SVG rendering).

use katsvg_engine::font::{FONT_STACK, font_style_block};
use katsvg_engine::InfographicIntentRouter;

#[test]
fn font_block_declares_embedded_inter() {
    let block = font_style_block();
    assert!(block.contains("@font-face"), "missing @font-face");
    assert!(block.contains("data:font/woff2;base64"), "font must be embedded as base64 data URI");
    assert!(block.contains("format('woff2')"), "woff2 format required");
}

#[test]
fn font_stack_has_fallbacks() {
    assert!(FONT_STACK.contains("Inter"), "Inter should lead the stack");
    assert!(FONT_STACK.contains("system-ui"), "should include system fallback");
}

#[test]
fn svg_embeds_font_and_has_no_network_refs() {
    let r = InfographicIntentRouter::new();
    let svg = katsvg_engine::SVGVectorRenderer::render(&r.parse_and_route("test infographic"));
    assert!(svg.contains("@font-face"), "SVG must embed the font");
    assert!(svg.contains("data:font/woff2;base64"), "SVG must carry the font data URI");
    assert!(!svg.contains("googleapis"), "no Google Fonts network reference");
    assert!(!svg.contains("@import"), "no @import network reference");
    assert!(svg.contains("http://www.w3.org/2000/svg"), "xmlns namespace is allowed");
}
