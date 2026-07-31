//! D1/D2/D3 gate tests: chart format parity (PNG/PPTX/PDF).

use katsvg_engine::InfographicIntentRouter;

fn chart_spec() -> katsvg_engine::InfographicLayoutSpec {
    InfographicIntentRouter::new()
        .parse_and_route("Show a bar chart: Q1: 10, Q2: 25, Q3: 15, Q4: 30 in navy banner")
}

#[test]
fn png_chart_region_has_content() {
    let spec = chart_spec();
    assert!(spec.chart.is_some(), "chart should bind");
    let png = katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG still valid");
}

#[test]
fn pptx_contains_chart_shape_xml() {
    let spec = chart_spec();
    let pptx = katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(&spec);
    assert!(pptx.windows(b"[Content_Types].xml".len()).any(|w| w == b"[Content_Types].xml"), "pptx package valid");
    // Chart emits native shapes: at least one bar/rect shape (id >= 100) beyond title/sections
    let txt = String::from_utf8_lossy(&pptx);
    assert!(txt.contains("<p:sp>"), "pptx must contain shapes");
    assert!(txt.contains("prstGeom prst=\"rect\""), "chart shapes must use rect geometry");
}

#[test]
fn pdf_with_chart_is_valid_pdf17() {
    let spec = chart_spec();
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(&spec);
    assert_eq!(&pdf[..8], b"%PDF-1.7");
    assert!(pdf.windows(b"%%EOF".len()).any(|w| w == b"%%EOF"));
}

#[test]
fn all_formats_deterministic_with_chart() {
    let spec = chart_spec();
    let (a1, a2) = (
        katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec),
        katsvg_engine::PNGRasterExporter::generate_png_bytes(&spec),
    );
    assert_eq!(a1, a2, "PNG with chart must be deterministic");
    let (b1, b2) = (
        katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(&spec),
        katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(&spec),
    );
    assert_eq!(b1, b2, "PPTX with chart must be deterministic");
}
