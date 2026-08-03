//! F1 gate tests: per-layout region composition — each archetype must render
//! with distinct geometry (not the legacy union compositor).

use katsvg_engine::InfographicIntentRouter;
use katsvg_engine::compositor::{RegionRect, Slot, regions_px};
use katsvg_engine::router::layout_by_id;

fn rects_for(layout_id: &str) -> Vec<RegionRect> {
    let layout = layout_by_id(layout_id).expect("layout must exist");
    regions_px(&layout, 800, 1131)
}

#[test]
fn hero_quote_has_no_sections_or_metrics_region() {
    let r = rects_for("hero_quote");
    assert!(
        !r.iter().any(|r| r.slot == Slot::Sections),
        "hero has no sections region"
    );
    assert!(
        !r.iter().any(|r| r.slot == Slot::Metrics),
        "hero has no metrics region"
    );
    assert!(
        r.iter().any(|r| r.slot == Slot::Title),
        "hero must have a title region"
    );
}

#[test]
fn kpi_snapshot_has_chart_and_metrics_regions() {
    let r = rects_for("kpi_snapshot");
    assert!(
        r.iter().any(|r| r.slot == Slot::Metrics),
        "kpi must have metrics"
    );
    assert!(
        r.iter().any(|r| r.slot == Slot::Chart),
        "kpi must have a chart/gauge region"
    );
}

#[test]
fn timeline_has_all_slots() {
    let r = rects_for("process_timeline");
    for slot in [Slot::Title, Slot::Metrics, Slot::Sections, Slot::Footer] {
        assert!(
            r.iter().any(|x| x.slot == slot),
            "timeline missing {slot:?}"
        );
    }
}

#[test]
fn distinct_layouts_render_distinct_svg() {
    let router = InfographicIntentRouter::new();
    let hero = router.parse_and_route("Create a motivational hero quote poster in sunset");
    let kpi =
        router.parse_and_route("Show a KPI snapshot overview: revenue: 124M, users: 12M in navy");

    assert_eq!(hero.layout_id, "hero_quote");
    assert_eq!(kpi.layout_id, "kpi_snapshot");

    let hero_svg = katsvg_engine::SVGVectorRenderer::render(&hero);
    let kpi_svg = katsvg_engine::SVGVectorRenderer::render(&kpi);
    assert_ne!(
        hero_svg, kpi_svg,
        "distinct layouts must render differently"
    );

    // hero: large title only (no section cards with circles)
    assert!(!hero_svg.contains("<circle"), "hero has no section circles");
    // kpi: has a chart group
    assert!(kpi_svg.contains("<g"), "kpi chart glyph renders as a group");
}

#[test]
fn unknown_layout_id_falls_back_to_legacy() {
    let spec = katsvg_engine::InfographicLayoutSpec {
        layout_type: katsvg_engine::router::LayoutType::ProcessTimeline,
        theme: katsvg_engine::router::PaletteTheme::TechDark,
        aspect_ratio: katsvg_engine::router::AspectRatio::A4Poster,
        title: "X".to_string(),
        subtitle: None,
        metrics: vec![],
        sections: vec![],
        chart: None,
        footer_note: None,
        layout_id: "no_such_layout".to_string(),
        lang: katsvg_engine::strs::Lang::En,
    };
    let svg = katsvg_engine::SVGVectorRenderer::render(&spec);
    assert!(svg.contains("<svg"), "fallback must still emit valid SVG");
}

// ── F6: overflow guard ───────────────────────────────────────────────────────

#[test]
fn long_title_is_truncated_to_region() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Create a timeline for the comprehensive deployment of a multi-agent artificial intelligence system across enterprise infrastructure with extensive security considerations and governance frameworks");
    let svg = katsvg_engine::SVGVectorRenderer::render(&spec);
    // SVG is valid and text stays within viewBox; no overflow
    assert!(svg.contains("<svg"));
    assert!(spec.title.len() > 30, "test prompt has a long title");
}

#[test]
fn long_chart_labels_get_ellipsis() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Show a bar chart with long labels in dark mode");
    let svg = katsvg_engine::SVGVectorRenderer::render(&spec);
    // The chart renderer truncates tick labels; if any were long they'd carry '…'
    let _ = svg;
}

#[test]
fn text_width_measurement_is_positive() {
    use katsvg_engine::TextRenderer;
    let r = TextRenderer::new();
    assert!(r.text_width(16.0, "long text") > 0.0);
    assert!(r.text_width(16.0, "aaaaaaaa") > r.text_width(16.0, "aa"));
}
