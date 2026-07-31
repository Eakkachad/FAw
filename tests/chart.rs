//! S5 gate tests: chart data binding and native SVG glyph rendering.

use katsvg_engine::chart::{ChartColors, ChartGlyphRenderer};
use katsvg_engine::router::{ChartSpec, ChartType};
use katsvg_engine::InfographicIntentRouter;

const COLORS: ChartColors<'static> = ChartColors {
    bg: "#0B0F19",
    card_bg: "#111827",
    accent1: "#3B82F6",
    accent2: "#10B981",
    text: "#F9FAFB",
};

#[test]
fn chart_extracted_only_when_requested() {
    let r = InfographicIntentRouter::new();
    let no_chart = r.parse_and_route("Build a 4-step AI Agent Deployment Timeline in dark mode");
    assert!(no_chart.chart.is_none(), "no chart intent -> no chart bound");

    let bar = r.parse_and_route("Show a bar chart: Q1: 10, Q2: 25, Q3: 15, Q4: 30");
    let chart = bar.chart.expect("bar chart intent should bind a chart");
    assert_eq!(chart.chart_type, ChartType::Bar);
    assert_eq!(chart.values, vec![10.0, 25.0, 15.0, 30.0]);
    assert_eq!(chart.labels, vec!["q1", "q2", "q3", "q4"]);
}

#[test]
fn chart_values_never_invented() {
    let r = InfographicIntentRouter::new();
    let line = r.parse_and_route("line chart: jan: 5, feb: 8, mar: 12");
    let chart = line.chart.expect("line chart should bind");
    assert_eq!(chart.chart_type, ChartType::Line);
    // every value must round-trip from the prompt
    assert!(chart.values.iter().all(|v| [5.0, 8.0, 12.0].contains(v)));
}

#[test]
fn insufficient_pairs_binds_nothing() {
    let r = InfographicIntentRouter::new();
    let s = r.parse_and_route("show a chart with one point: a: 1");
    assert!(s.chart.is_none(), "single pair is not a chart series");
}

#[test]
fn all_six_glyphs_render_svg() {
    let spec = ChartSpec {
        chart_type: ChartType::Bar,
        labels: vec!["a".into(), "b".into(), "c".into()],
        values: vec![10.0, 25.0, 15.0],
        unit: None,
    };
    for ct in [
        ChartType::Bar,
        ChartType::Line,
        ChartType::Pie,
        ChartType::Scatter,
        ChartType::Heatmap,
        ChartType::Gauge,
    ] {
        let mut s = spec.clone();
        s.chart_type = ct;
        let svg = ChartGlyphRenderer::render(&s, &COLORS, 40, 240, 720, 260);
        assert!(!svg.trim().is_empty(), "{:?} produced no svg", ct);
        assert!(svg.contains("<g"), "{:?} missing group", ct);
    }
}

#[test]
fn glyph_output_is_deterministic() {
    let spec = ChartSpec {
        chart_type: ChartType::Bar,
        labels: vec!["a".into(), "b".into(), "c".into()],
        values: vec![10.0, 25.0, 15.0],
        unit: None,
    };
    let a = ChartGlyphRenderer::render(&spec, &COLORS, 40, 240, 720, 260);
    let b = ChartGlyphRenderer::render(&spec, &COLORS, 40, 240, 720, 260);
    assert_eq!(a, b, "chart glyph must be byte-identical across runs");
}
