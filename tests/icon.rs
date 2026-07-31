//! P2 gate tests: icon corpus + IconRenderer.

use katsvg_engine::IconRenderer;

#[test]
fn corpus_has_icons() {
    assert!(IconRenderer::count() >= 10, "expected >=10 icons, got {}", IconRenderer::count());
    for name in ["zap", "cpu", "shield-check", "chart", "target", "users", "clock", "trending-up", "layers", "check-circle"] {
        assert!(IconRenderer::has(name), "missing icon {name}");
    }
}

#[test]
fn renders_known_icon_as_svg() {
    let svg = IconRenderer::render("zap", "#10B981");
    assert!(svg.contains("<g"), "icon must render as a group");
    assert!(svg.contains("<path"), "icon must contain path");
    assert!(svg.contains("#10B981"), "icon must carry the stroke color");
}

#[test]
fn unknown_icon_falls_back_gracefully() {
    // Unknown name -> empty paths -> still emits a (contentless) group, never panics.
    let svg = IconRenderer::render("no-such-icon", "#fff");
    assert!(svg.contains("<g"), "fallback must emit a group");
}

#[test]
fn paths_split_multi_shape_icons() {
    // "cpu" has multiple M commands -> multiple paths.
    let paths = IconRenderer::paths("cpu");
    assert!(paths.len() >= 2, "cpu should split into multiple paths, got {}", paths.len());
}

#[test]
fn icon_renders_into_metric_cards() {
    use katsvg_engine::InfographicIntentRouter;
    let r = InfographicIntentRouter::new();
    // metrics extract with a k:v pair -> icon present in SVG output
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let svg = katsvg_engine::SVGVectorRenderer::render(&spec);
    assert!(svg.contains("stroke-linecap"), "SVG must contain icon stroke markup");
}
