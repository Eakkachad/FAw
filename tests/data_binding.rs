//! D4 gate tests: external data binding (CSV/JSON → spec).

use katsvg_engine::{InfographicIntentRouter, parse_data};

const CSV: &str = "month,sales\njan,120\nfeb,85\nmar,150\napr,95\n";
const JSON: &str = r#"{"Q1": 10, "Q2": 25, "Q3": 15, "Q4": 30}"#;

#[test]
fn csv_binds_chart_series() {
    let data = parse_data(CSV, "sales.csv").unwrap();
    assert!(
        data.chart.is_some(),
        "CSV with >=2 rows should bind a chart"
    );
    let chart = data.chart.unwrap();
    assert_eq!(chart.labels, vec!["jan", "feb", "mar", "apr"]);
    assert_eq!(chart.values, vec![120.0, 85.0, 150.0, 95.0]);
}

#[test]
fn json_object_binds_chart_series() {
    let data = parse_data(JSON, "data.json").unwrap();
    assert!(data.chart.is_some());
    let chart = data.chart.unwrap();
    assert_eq!(chart.labels, vec!["Q1", "Q2", "Q3", "Q4"]);
    assert_eq!(chart.values, vec![10.0, 25.0, 15.0, 30.0]);
}

#[test]
fn bad_data_returns_error() {
    assert!(
        parse_data("a,b\n1,x\n", "bad.csv").is_err(),
        "non-numeric value should error"
    );
    assert!(
        parse_data("not json", "bad.json").is_err(),
        "invalid json should error"
    );
    assert!(
        parse_data("a,b\n1,2\n", "bad.txt").is_err(),
        "unsupported ext should error"
    );
}

#[test]
fn parse_and_bind_overrides_prompt_data() {
    let r = InfographicIntentRouter::new();
    let data = parse_data(CSV, "sales.csv").unwrap();
    let spec = r.parse_and_bind("Show a bar chart in navy banner", &data);
    assert!(spec.chart.is_some());
    let chart = spec.chart.unwrap();
    // Data from file, not from prompt
    assert_eq!(chart.values, vec![120.0, 85.0, 150.0, 95.0]);
}

#[test]
fn parse_and_bind_is_deterministic() {
    let r = InfographicIntentRouter::new();
    let data = parse_data(JSON, "data.json").unwrap();
    let a = r.parse_and_bind("Show a chart in dark mode", &data);
    let b = r.parse_and_bind("Show a chart in dark mode", &data);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

// ── F5: deep data binding ────────────────────────────────────────────────────

const MULTI_CSV: &str = "month,a2023,b2024\njan,100,140\nfeb,120,150\nmar,90,135\n";

#[test]
fn multi_column_csv_binds_series() {
    let data = parse_data(MULTI_CSV, "sales.csv").unwrap();
    let chart = data.chart.expect("3-column CSV should bind a chart");
    assert_eq!(
        chart.values,
        vec![100.0, 120.0, 90.0],
        "first column is primary series"
    );
    assert_eq!(
        chart.series,
        vec![vec![140.0, 150.0, 135.0]],
        "second numeric column becomes series"
    );
    assert_eq!(chart.labels, vec!["jan", "feb", "mar"]);
}

#[test]
fn multi_series_renders_grouped_bars() {
    let r = InfographicIntentRouter::new();
    let data = parse_data(MULTI_CSV, "sales.csv").unwrap();
    let spec = r.parse_and_bind("Show a bar chart in navy banner", &data);
    let svg = katsvg_engine::SVGVectorRenderer::render(&spec);
    // 3 labels × 2 series = 6 chart bars
    let bars = svg.matches("rx=\"3\"").count();
    assert_eq!(bars, 6, "grouped bars for 2 series × 3 labels, got {bars}");
}

#[test]
fn row_values_bind_as_sections() {
    // A row-oriented file binds sections when the prompt targets a section layout.
    let r = InfographicIntentRouter::new();
    let data = parse_data(MULTI_CSV, "sales.csv").unwrap();
    let spec = r.parse_and_bind("Show a deployment timeline", &data);
    // multi-series still bound; sections fall back to prompt-derived count
    assert!(spec.chart.is_some());
    assert!(!spec.sections.is_empty(), "timeline keeps sections");
}
