//! Integration Example for katsvg-engine Crate

use katsvg_engine::{ExportManager, InfographicIntentRouter};
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("=== katSVG Engine Integration Demo ===");

    let router = InfographicIntentRouter::new();
    let prompt = "Build a 4-step AI Agent Deployment Timeline in tech dark theme";

    let start = Instant::now();
    let spec = router.parse_and_route(prompt);

    let out_dir = Path::new("demo_output");
    let result = ExportManager::export_all(&spec, out_dir).expect("Export failed");

    let duration = start.elapsed();

    println!("⚡ Total Latency: {:.3?} ms", duration.as_secs_f64() * 1000.0);
    println!("📄 SVG Path: {:?}", result.svg_path);
    println!("📕 PDF Path: {:?}", result.pdf_path);
    println!("🖼️ PNG Path: {:?}", result.png_path);
    println!("📊 PPTX Path: {:?}", result.pptx_path);
}
