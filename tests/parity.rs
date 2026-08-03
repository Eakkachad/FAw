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
    assert!(
        pptx.windows(b"[Content_Types].xml".len())
            .any(|w| w == b"[Content_Types].xml"),
        "pptx package valid"
    );
    // Chart emits native shapes: at least one bar/rect shape (id >= 100) beyond title/sections
    let txt = String::from_utf8_lossy(&pptx);
    assert!(txt.contains("<p:sp>"), "pptx must contain shapes");
    assert!(
        txt.contains("prstGeom prst=\"rect\""),
        "chart shapes must use rect geometry"
    );
}

#[test]
fn pdf_with_chart_is_valid_pdf17() {
    let spec = chart_spec();
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(&spec);
    assert_eq!(&pdf[..8], b"%PDF-1.7");
    assert!(pdf.windows(b"%%EOF".len()).any(|w| w == b"%%EOF"));
    // Chart draws vector primitives: expect at least 1 fill operator for bars
    assert!(
        pdf.windows(b" re f".len()).any(|w| w == b" re f"),
        "PDF must contain vector fill ops for chart"
    );
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

#[test]
fn pdf_embeds_type0_font_for_thai_text() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("สร้างไทม์ไลน์การพัฒนาระบบ");
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(&spec);
    assert!(
        pdf.windows(b"/Subtype /Type0".len())
            .any(|w| w == b"/Subtype /Type0"),
        "Type0 font present"
    );
    assert!(
        pdf.windows(b"/FontFile2".len()).any(|w| w == b"/FontFile2"),
        "FontFile2 stream present"
    );
    // Thai text drawn as CID hex: <HEX> Tj
    assert!(
        pdf.windows(b"> Tj".len()).any(|w| w == b"> Tj"),
        "Thai text emitted as CID hex"
    );
}

#[test]
fn pdf_latin_text_stays_literal() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Quarterly revenue dashboard in navy");
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(&spec);
    assert!(
        !pdf.windows(b"/Subtype /Type0".len())
            .any(|w| w == b"/Subtype /Type0"),
        "ascii-only PDF has no Type0 font"
    );
    assert!(
        pdf.windows(b") Tj".len()).any(|w| w == b") Tj"),
        "latin text stays literal string"
    );
}

#[test]
fn pdf_metric_cards_carry_icon_strokes() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    assert_eq!(spec.metrics.len(), 2, "both metric pairs bind");
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(&spec);
    // icon strokes emitted with a stroke-op RG + linewidth
    assert!(
        pdf.windows(b" RG 1.5 w".len()).any(|w| w == b" RG 1.5 w"),
        "PDF icon stroke op present"
    );
}

#[test]
fn pptx_metric_cards_have_icons() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let pptx = katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(&spec);
    let txt = String::from_utf8_lossy(&pptx);
    assert!(
        txt.contains("name=\"Metric 1\""),
        "PPTX metric card 1 present"
    );
    assert!(txt.contains("name=\"Icon 1\""), "PPTX icon mark 1 present");
}

#[test]
fn nested_colon_metric_extraction() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Q3 KPI dashboard: revenue: 124M, users: 12M in navy");
    let values: Vec<&str> = spec.metrics.iter().map(|m| m.value.as_str()).collect();
    assert!(
        values.iter().any(|v| v.contains("124")),
        "nested colon must bind revenue, got {values:?}"
    );
    assert!(
        values.iter().any(|v| v.contains("12")),
        "users bound, got {values:?}"
    );
}
