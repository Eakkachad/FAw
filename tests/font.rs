//! P4/F2 gate tests: embedded fonts (Inter + conditional Noto Sans Thai).

use katsvg_engine::InfographicIntentRouter;
use katsvg_engine::font::{font_stack, font_style_block, has_non_ascii};

#[test]
fn font_block_declares_embedded_inter() {
    let block = font_style_block(false);
    assert!(block.contains("@font-face"), "missing @font-face");
    assert!(
        block.contains("data:font/woff2;base64"),
        "Inter embedded as woff2 data URI"
    );
    assert!(block.contains("format('woff2')"), "woff2 format required");
}

#[test]
fn font_block_conditionally_embeds_thai() {
    let ascii = font_style_block(false);
    assert!(
        !ascii.contains("Noto Sans Thai"),
        "no Thai face for ascii-only text"
    );

    let thai = font_style_block(true);
    assert!(
        thai.contains("Noto Sans Thai"),
        "Thai face present when has_thai"
    );
    assert!(
        thai.contains("data:font/truetype;base64"),
        "Thai embedded as truetype data URI"
    );
}

#[test]
fn font_stack_has_fallbacks() {
    assert!(
        font_stack(false).contains("Inter"),
        "Inter should lead the stack"
    );
    assert!(
        font_stack(false).contains("system-ui"),
        "should include system fallback"
    );
    assert!(
        font_stack(true).contains("Noto Sans Thai"),
        "Thai stack includes Noto Sans Thai"
    );
}

#[test]
fn has_non_ascii_detects_thai() {
    assert!(has_non_ascii("รายงานการเงิน"));
    assert!(!has_non_ascii("Quarterly report"));
}

#[test]
fn svg_embeds_thai_font_for_thai_prompt() {
    let r = InfographicIntentRouter::new();
    let svg = katsvg_engine::SVGVectorRenderer::render(&r.parse_and_route("สร้างไทม์ไลน์การพัฒนาระบบ"));
    assert!(
        svg.contains("Noto Sans Thai"),
        "Thai prompt SVG must embed Noto Sans Thai"
    );
    assert!(!svg.contains("googleapis"), "no network ref");
    assert!(!svg.contains("@import"), "no @import");
}

#[test]
fn svg_skips_thai_font_for_ascii_prompt() {
    let r = InfographicIntentRouter::new();
    let svg =
        katsvg_engine::SVGVectorRenderer::render(&r.parse_and_route("Quarterly revenue dashboard"));
    assert!(
        !svg.contains("Noto Sans Thai"),
        "ascii prompt SVG must not carry the Thai font"
    );
    assert!(
        svg.contains("data:font/woff2;base64"),
        "Inter still embedded"
    );
}
