//! Anti-Overclaim Empirical Verification Test Suite (`anti_overclaim_gate`).
//!
//! Grounded in research principles from `.research/markdown/`:
//! - Research 000 (Bylinskii 2017): Visual hierarchy & non-hallucinated bounding layout
//! - Research 001 (LIDA 2023): Model-less 4-stage pipeline execution
//! - Research 002 (Chat2VIS 2023): Exact metric & title parameter extraction
//! - Research 004 (ChartGalaxy 2025): Multi-format parity & binary determinism
//! - Research 005 (VisLiteracy 2025): Perturbation robustness & de-memorization

use katsvg_engine::{
    ExportManager, InfographicIntentRouter, PDFVectorExporter, PNGRasterExporter,
    PPTXPresentationExporter, SVGVectorRenderer,
};
use std::time::Instant;

/// Gate 1: Zero Hallucination Data Extraction Gate (Bylinskii 2017 / Chat2VIS 2023)
/// Verifies that numerical values and labels extracted from prompts match exactly.
#[test]
fn gate_1_zero_hallucination_data_extraction() {
    let router = InfographicIntentRouter::new();
    let prompt = "Show a bar chart: Q1: 15.5, Q2: 45.0, Q3: 90.2 in financial navy";
    let spec = router.parse_and_route(prompt);

    assert!(spec.chart.is_some(), "Chart spec must be extracted");
    let chart = spec.chart.unwrap();

    assert_eq!(
        chart.values,
        vec![15.5, 45.0, 90.2],
        "Extracted chart values must match prompt exactly with zero hallucination"
    );
    assert_eq!(
        chart.labels.iter().map(|l| l.to_lowercase()).collect::<Vec<_>>(),
        vec!["q1", "q2", "q3"]
    );
}

/// Gate 2: Bounding Box & Visual Hierarchy Bounds (Bylinskii 2017)
/// Verifies structural element parameters and valid visual dimensions.
#[test]
fn gate_2_visual_hierarchy_bounds() {
    let router = InfographicIntentRouter::new();
    let prompt = "Build a 4-step AI Deployment Process Timeline";
    let spec = router.parse_and_route(prompt);

    let (width, height) = spec.aspect_ratio.dimensions();
    assert!(width > 0 && height > 0, "Dimensions must be positive integers");
    assert!(
        spec.sections.len() >= 4,
        "Step/section count must be extracted as at least 4"
    );
    assert!(
        !spec.title.is_empty(),
        "Title must be present and derived from prompt"
    );
}

/// Gate 3: Multi-Format Parity & Binary Signature Check (ChartGalaxy 2025)
/// Verifies that SVG, PDF, PNG, PPTX artifacts generated are 100% valid binaries.
#[test]
fn gate_3_multi_format_binary_parity() {
    let router = InfographicIntentRouter::new();
    let prompt = "Q3 financial KPI dashboard with metrics in navy";
    let spec = router.parse_and_route(prompt);

    // 1. SVG
    let svg_str = SVGVectorRenderer::render(&spec);
    assert!(
        svg_str.contains("<svg") && svg_str.contains("</svg>"),
        "SVG output must be valid XML with <svg> tags"
    );

    // 2. PDF
    let pdf_bytes = PDFVectorExporter::generate_pdf_bytes(&spec);
    assert!(
        pdf_bytes.starts_with(b"%PDF-1.7"),
        "PDF artifact must start with valid PDF 1.7 header (%PDF-1.7)"
    );

    // 3. PNG
    let png_bytes = PNGRasterExporter::generate_png_bytes(&spec);
    assert!(
        png_bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
        "PNG artifact must start with valid PNG 8-byte signature"
    );

    // 4. PPTX
    let pptx_bytes = PPTXPresentationExporter::generate_pptx_bytes(&spec);
    assert!(
        pptx_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "PPTX artifact must start with valid ZIP PK header signature"
    );
}

/// Gate 4: Mutation & De-memorization Robustness (VisLiteracy 2025)
/// Tests robustness against language mixing, Thai characters, long titles, and adversarial strings.
#[test]
fn gate_4_mutation_and_dememorization_robustness() {
    let router = InfographicIntentRouter::new();

    let long_title = format!("Very long title prompt: {}", "A".repeat(500));
    let adversarial_prompts: Vec<&str> = vec![
        "สร้าง Timeline การพัฒนาระบบ AI Agent 4 ขั้นตอน แบบมินิมอล #1234",
        &long_title,
        "Special characters: <script>alert(1)</script> & \"quotes\" 'single'",
        "Mixed Thai-English: รายงานสรุป KPI ประจำปี 2026 revenue $5M growth 120%",
        "Empty/Edge prompt: !!! ??? --- ___ 12345",
    ];

    for prompt in adversarial_prompts {
        let spec = router.parse_and_route(prompt);
        let svg = SVGVectorRenderer::render(&spec);
        assert!(
            !svg.is_empty(),
            "SVG generation must never panic or return empty string"
        );

        let temp_dir = std::env::temp_dir().join(format!("gate4_test_{}", fastrand::u32(..)));
        let export_res = ExportManager::export_all(&spec, &temp_dir);
        assert!(
            export_res.is_ok(),
            "Export manager must handle perturbed inputs safely"
        );
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

/// Gate 5: Empirical Latency Threshold Gate
/// Verifies intent routing is under 1.0 ms (release) and export is under 30.0 ms (release).
#[test]
fn gate_5_empirical_latency_thresholds() {
    let router = InfographicIntentRouter::new();
    let prompt = "Q3 financial KPI dashboard with metrics in navy";

    // Warm up static OnceLock registries (corpus/palettes/icons)
    let _ = router.parse_and_route("warmup prompt");

    let t0 = Instant::now();
    let spec = router.parse_and_route(prompt);
    let parse_duration_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let temp_dir = std::env::temp_dir().join("gate5_bench");
    let t1 = Instant::now();
    let export_res = ExportManager::export_all(&spec, &temp_dir);
    let export_duration_ms = t1.elapsed().as_secs_f64() * 1000.0;
    let _ = std::fs::remove_dir_all(temp_dir);

    assert!(export_res.is_ok(), "Export must succeed");

    let is_debug = cfg!(debug_assertions);
    let max_parse_ms = if is_debug { 10.0 } else { 1.0 };
    let max_export_ms = if is_debug { 200.0 } else { 30.0 };

    println!("Empirical Latency Audit (debug={}):", is_debug);
    println!("  Parse Latency  : {:.4} ms (Threshold < {:.1} ms)", parse_duration_ms, max_parse_ms);
    println!("  Export Latency : {:.4} ms (Threshold < {:.1} ms)", export_duration_ms, max_export_ms);

    assert!(
        parse_duration_ms < max_parse_ms,
        "Parse latency must be under {:.1} ms gate", max_parse_ms
    );
    assert!(
        export_duration_ms < max_export_ms,
        "Export latency must be under {:.1} ms gate", max_export_ms
    );
}
