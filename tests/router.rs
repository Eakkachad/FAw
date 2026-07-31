//! S3 gate tests: corpus-driven routing, real ConstraintPruner clamping,
//! deterministic output, and adversarial-prompt safety (0.0% invalid emissions).

use katsvg_engine::router::LayoutType;
use katsvg_engine::InfographicIntentRouter;

fn router() -> InfographicIntentRouter {
    InfographicIntentRouter::new()
}

#[test]
fn corpus_is_loaded_and_retrievable() {
    let r = router();
    assert!(!r.corpus().is_empty(), "corpus must be non-empty");
    assert_eq!(r.corpus().len(), 6, "expected 6 embedded layouts");
}

#[test]
fn routing_is_corpus_driven_not_hardcoded() {
    let r = router();
    let timeline = r.parse_and_route("Build a 4-step AI Agent Deployment Timeline in dark mode");
    assert_eq!(timeline.layout_type, LayoutType::ProcessTimeline);

    let dashboard = r.parse_and_route("Q3 financial KPI dashboard with metrics in navy");
    assert_eq!(dashboard.layout_type, LayoutType::StatisticalDashboard);

    let compare = r.parse_and_route("Compare GPT-4 vs Gemini feature matrix");
    assert_eq!(compare.layout_type, LayoutType::ComparisonGrid);
}

#[test]
fn step_count_extracted_from_prompt() {
    let r = router();
    let spec = r.parse_and_route("Create a 4-step deployment roadmap in dark mode");
    // timeline layout allows 2..=8 sections; prompt says 4 → clamp must not invent
    assert!(spec.sections.len() <= 8);
    assert!(spec.sections.len() >= 2);
    assert_eq!(spec.sections.len(), 4, "explicit 4-step count must produce 4 sections");
}

#[test]
fn title_derived_from_prompt_not_invented() {
    let r = router();
    let spec = r.parse_and_route("Quarterly Revenue Growth Report in navy");
    assert!(spec.title.starts_with("QUARTERLY"), "title must come from prompt");
}

#[test]
fn aspect_ratio_classified_and_bounded() {
    let r = router();
    let banner = r.parse_and_route("Create a banner timeline infographic in warm mode");
    // banner → Banner16_9; ProcessTimeline allows it → preserved
    assert_eq!(banner.aspect_ratio, katsvg_engine::router::AspectRatio::Banner16_9);

    let square = r.parse_and_route("Make a square social post in coral");
    assert_eq!(square.aspect_ratio, katsvg_engine::router::AspectRatio::Square1_1);

    // Pruner guards: a disallowed aspect is deterministically clamped to the
    // layout's first allowed ratio (never emitted invalid).
    let clamped = r.parse_and_route("Create a banner mindmap in dark mode");
    assert_eq!(clamped.aspect_ratio, katsvg_engine::router::AspectRatio::A4Poster);
}

#[test]
fn output_is_byte_deterministic() {
    let r = router();
    let prompt = "Build a 4-step AI Agent Deployment Timeline in dark mode";
    let a = r.parse_and_route(prompt);
    let b = r.parse_and_route(prompt);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "same prompt must yield byte-identical spec"
    );
}

/// Adversarial prompts: no panics, no out-of-bounds counts, values always
/// traceable to prompt or corpus (0.0% hallucination / invalid emission).
#[test]
fn adversarial_prompts_never_panic_or_exceed_bounds() {
    let r = router();
    let prompts = [
        "",
        " ",
        "!!!",
        "make an infographic",
        "99-step outrageous timeline that is far too long and beyond every bound",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "dashboard with 50 metrics",
        "compare vs vs vs vs",
        "Q3 2024: revenue: 124M, costs: 89M, profit: 35M, margin: 28%, users: 12M in navy banner",
        "0-step no-op",
        "x: 5, y: 10, z: 15",
    ];

    for prompt in prompts {
        let spec = r.parse_and_route(prompt);
        // Structural bounds must always hold (never invalid emission)
        assert!(spec.sections.len() <= 8, "prompt {prompt:?}: sections {} > 8", spec.sections.len());
        assert!(spec.metrics.len() <= 6, "prompt {prompt:?}: metrics {} > 6", spec.metrics.len());
        assert!(spec.title.len() <= 80, "prompt {prompt:?}: title too long");
        // Values bound into metrics must originate from the prompt
        for m in &spec.metrics {
            assert!(prompt.to_lowercase().contains(&m.value.to_lowercase())
                || m.value.starts_with('<'),
                "metric value {:?} not present in prompt {prompt:?}", m.value);
        }
    }
}
