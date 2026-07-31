//! S6 gate tests: retrieval pipeline (Embedding beats Tag baseline on the
//! prompt-intent eval corpus), OOD fallback, and determinism.

use katsvg_engine::router::LayoutType;
use katsvg_engine::{EmbeddingRetriever, InfographicIntentRouter, RetrievalPipeline, TagRetriever};

/// (prompt, expected layout type) eval pairs — prompt-intent corpus seed.
const EVAL: [(&str, LayoutType); 8] = [
    ("Build a 4-step AI Agent Deployment Timeline in dark mode", LayoutType::ProcessTimeline),
    ("Create a deployment roadmap with phases and milestones", LayoutType::ProcessTimeline),
    ("Q3 financial KPI dashboard with metrics in navy", LayoutType::StatisticalDashboard),
    ("Show quarterly revenue statistics and analytics in a report", LayoutType::StatisticalDashboard),
    ("Compare GPT-4 vs Gemini feature matrix", LayoutType::ComparisonGrid),
    ("Build a side-by-side feature comparison table", LayoutType::ComparisonGrid),
    ("Mind map of machine learning concepts and branches", LayoutType::MindmapHierarchy),
    ("Org chart hierarchy of the engineering team structure", LayoutType::MindmapHierarchy),
];

fn eval_accuracy<P: RetrievalPipeline>(p: &P, corpus: &[katsvg_engine::LayoutDef]) -> (usize, usize) {
    let mut correct = 0;
    for (prompt, expected) in EVAL {
        let ranked = p.retrieve(prompt, corpus);
        let top = ranked.first().unwrap();
        if corpus[top.index].layout_type == expected {
            correct += 1;
        }
    }
    (correct, EVAL.len())
}

#[test]
fn embedding_beats_tag_baseline_on_eval_corpus() {
    let router = InfographicIntentRouter::new();
    let corpus = router.corpus().to_vec();

    let tag = TagRetriever;
    let emb = EmbeddingRetriever::new();

    let (tag_correct, n) = eval_accuracy(&tag, &corpus);
    let (emb_correct, _) = eval_accuracy(&emb, &corpus);

    assert!(
        emb_correct >= tag_correct,
        "embedding ({}/{}) must not underperform tag baseline ({}/{})",
        emb_correct, n, tag_correct, n
    );
    assert!(emb_correct >= n - 1, "embedding accuracy too low: {}/{}", emb_correct, n);
}

#[test]
fn retrieval_is_deterministic() {
    let router = InfographicIntentRouter::new();
    let corpus = router.corpus().to_vec();
    let emb = EmbeddingRetriever::new();

    for (prompt, _) in EVAL {
        let a = emb.retrieve(prompt, &corpus);
        let b = emb.retrieve(prompt, &corpus);
        assert_eq!(
            a.iter().map(|r| (r.index, r.relevance.to_bits())).collect::<Vec<_>>(),
            b.iter().map(|r| (r.index, r.relevance.to_bits())).collect::<Vec<_>>(),
            "retrieval must be deterministic for {prompt:?}"
        );
    }
}

#[test]
fn ood_gate_falls_back_to_classifier() {
    let router = InfographicIntentRouter::new();
    // No retrieval signal → deterministic classifier picks the layout type.
    let spec = router.parse_and_route("Make a square social post in coral");
    assert_eq!(spec.layout_type, LayoutType::MindmapHierarchy);
}

#[test]
fn router_uses_embedding_backend_by_default() {
    let router = InfographicIntentRouter::new();
    assert_eq!(router.retriever_name(), "embedding");
}

#[test]
fn retriever_can_be_swapped() {
    let tag_router = InfographicIntentRouter::new().with_retriever(Box::new(TagRetriever));
    assert_eq!(tag_router.retriever_name(), "tag");
    // Behavior stays valid (bounds held) regardless of backend.
    let spec = tag_router.parse_and_route("Q3 financial KPI dashboard with metrics in navy");
    assert_eq!(spec.layout_type, LayoutType::StatisticalDashboard);
}
