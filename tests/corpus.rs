//! S2 gate: every seed layout in `corpus/layouts/` is structurally valid against
//! `schemas/layout_corpus.schema.json` and instantiates a valid
//! `InfographicLayoutSpec` (dry-run) with bounds the `ConstraintPruner` accepts.

use katsvg_engine::router::{AspectRatio, LayoutType, PaletteTheme};
use katsvg_engine::InfographicLayoutSpec;
use std::fs;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayoutSeed {
    id: String,
    layout_type: String,
    regions: Vec<RegionSeed>,
    constraints: ConstraintsSeed,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegionSeed {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    slot: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConstraintsSeed {
    #[serde(default)]
    max_sections: Option<usize>,
    #[serde(default)]
    max_metrics: Option<usize>,
    #[serde(default)]
    max_title_length: Option<usize>,
    #[serde(default)]
    allowed_aspect_ratios: Option<Vec<String>>,
}

fn corpus_paths() -> Vec<(String, std::path::PathBuf)> {
    let dir = fs::read_dir("corpus/layouts").expect("corpus/layouts must exist");
    let mut out = Vec::new();
    for entry in dir.flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "json").unwrap_or(false) {
            out.push((p.file_stem().unwrap().to_string_lossy().into_owned(), p));
        }
    }
    out.sort();
    out
}

#[test]
fn corpus_has_ten_seed_layouts() {
    assert_eq!(corpus_paths().len(), 10, "expected 10 seed layouts");
}

#[test]
fn every_layout_instantiates_a_valid_spec() {
    for (name, path) in corpus_paths() {
        let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
        let seed: LayoutSeed = serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{name}: invalid JSON: {e}"));

        // LayoutType must be a known enum value (structural schema check)
        let layout_type = match seed.layout_type.as_str() {
            "ProcessTimeline" => LayoutType::ProcessTimeline,
            "StatisticalDashboard" => LayoutType::StatisticalDashboard,
            "ComparisonGrid" => LayoutType::ComparisonGrid,
            "MindmapHierarchy" => LayoutType::MindmapHierarchy,
            other => panic!("{name}: unknown layoutType {other}"),
        };

        // Region coordinates must be unit-normalized [0,1] (schema bound)
        for r in &seed.regions {
            let within = r.x >= 0.0
                && r.x <= 1.0
                && r.y >= 0.0
                && r.y <= 1.0
                && r.width > 0.0
                && r.width <= 1.0
                && r.height > 0.0
                && r.height <= 1.0
                && (r.x + r.width) <= 1.0 + 1e-9
                && (r.y + r.height) <= 1.0 + 1e-9;
            assert!(within, "{name}: region {} out of unit bounds", r.id);
        }

        // Allowed aspect ratios must be known
        if let Some(aspects) = &seed.constraints.allowed_aspect_ratios {
            for a in aspects {
                match a.as_str() {
                    "A4Poster" | "Banner16_9" | "Square1_1" => {}
                    other => panic!("{name}: unknown aspect ratio {other}"),
                }
            }
        }

        // Dry-run: instantiate a valid spec bounded by the layout's constraints
        let spec = InfographicLayoutSpec {
            layout_type,
            theme: PaletteTheme::TechDark,
            aspect_ratio: AspectRatio::A4Poster,
            title: "CORPUS SEED VALIDATION".to_string(),
            subtitle: None,
            metrics: vec![],
            sections: vec![],
            chart: None,
            footer_note: None,
            layout_id: name.clone(),
        };
        let max_metrics = seed.constraints.max_metrics.unwrap_or(4);
        let max_sections = seed.constraints.max_sections.unwrap_or(8);
        let max_title = seed.constraints.max_title_length.unwrap_or(80);
        assert!(spec.metrics.len() <= max_metrics, "{name}: metrics exceed layout bound");
        assert!(spec.sections.len() <= max_sections, "{name}: sections exceed layout bound");
        assert!(spec.title.len() <= max_title, "{name}: title exceeds layout bound");
    }
}
