//! F4 gate tests: spec round-trip (route → emit-spec → reload → identical).

use katsvg_engine::InfographicIntentRouter;

#[test]
fn spec_serializes_and_reloads() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let json = serde_json::to_string_pretty(&spec).unwrap();
    let reloaded: katsvg_engine::InfographicLayoutSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_string(&reloaded).unwrap(),
        serde_json::to_string(&spec).unwrap()
    );
}

#[test]
fn round_trip_render_is_byte_identical() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let json = serde_json::to_string(&spec).unwrap();
    let reloaded: katsvg_engine::InfographicLayoutSpec = serde_json::from_str(&json).unwrap();

    let a = katsvg_engine::SVGVectorRenderer::render(&spec);
    let b = katsvg_engine::SVGVectorRenderer::render(&reloaded);
    assert_eq!(a, b, "reloaded spec must render byte-identical SVG");

    let pa = katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec);
    let pb = katsvg_engine::PNGRasterExporter::generate_png_bytes(&reloaded);
    assert_eq!(pa, pb, "reloaded spec must render byte-identical PNG");
}

#[test]
fn spec_carries_layout_id() {
    let r = InfographicIntentRouter::new();
    let hero = r.parse_and_route("Create a motivational hero quote poster in sunset");
    assert_eq!(hero.layout_id, "hero_quote");
    let json = serde_json::to_string(&hero).unwrap();
    let reloaded: katsvg_engine::InfographicLayoutSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(
        reloaded.layout_id, "hero_quote",
        "layout_id survives round-trip"
    );
}
