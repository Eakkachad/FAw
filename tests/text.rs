//! P5 gate tests: PNG text rasterization.

use katsvg_engine::router::InfographicLayoutSpec;
use katsvg_engine::{InfographicIntentRouter, TextRenderer};

#[test]
fn text_renderer_draws_into_buffer() {
    let renderer = TextRenderer::new();
    let (w, h) = (300, 80);
    let mut buf = vec![0u8; w * h * 3]; // black background
    renderer.draw_text(&mut buf, w, h, 10.0, 40.0, 24.0, (255, 255, 255), "Hello ไทย");
    let non_bg = buf.chunks_exact(3).filter(|p| *p != [0, 0, 0]).count();
    assert!(non_bg > 100, "text rasterization must produce visible pixels, got {non_bg}");
}

#[test]
fn text_width_is_positive_and_monotonic() {
    let renderer = TextRenderer::new();
    let w1 = renderer.text_width(16.0, "a");
    let w2 = renderer.text_width(16.0, "abcdef");
    assert!(w1 > 0.0);
    assert!(w2 > w1, "longer text should be wider");
}

#[test]
fn png_with_text_is_still_valid_png() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let png = katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG signature preserved");
    assert!(png.ends_with(b"IEND\xaeB`\x82"), "IEND trailer preserved");
}

#[test]
fn png_is_deterministic_with_text() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let a = katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec);
    let b = katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec);
    assert_eq!(a, b, "PNG output must be byte-identical (font rasterization is deterministic)");
}

#[test]
fn thai_text_rasterizes_without_panic() {
    let renderer = TextRenderer::new();
    let (w, h) = (400, 60);
    let mut buf = vec![255u8; w * h * 3];
    renderer.draw_text(&mut buf, w, h, 10.0, 40.0, 20.0, (0, 0, 0), "รายงานการเงินประจำไตรมาส");
    let changed = buf
        .chunks_exact(3)
        .filter(|p| *p != [255, 255, 255])
        .count();
    assert!(changed > 50, "Thai text should rasterize visible glyphs, got {changed}");
}
