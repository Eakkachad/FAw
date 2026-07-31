//! Integration tests for katSVG exporters — regression gates for D1 (PNG),
//! D2 (PPTX), D5 (offline SVG), and byte-determinism.

use katsvg_engine::{InfographicIntentRouter, PDFVectorExporter, PPTXPresentationExporter, PNGRasterExporter, SVGVectorRenderer};

fn spec() -> katsvg_engine::InfographicLayoutSpec {
    InfographicIntentRouter::new().parse_and_route(
        "Build a 4-step AI Agent Deployment Timeline in dark mode",
    )
}

#[test]
fn png_has_valid_signature_and_chunks() {
    let png = PNGRasterExporter::generate_png_bytes(&spec());
    // PNG signature
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "PNG signature missing");
    // IHDR chunk
    assert_eq!(&png[8..16], b"\x00\x00\x00\x0dIHDR", "IHDR chunk missing");
    // Correct dimensions (800 x 1131 from A4Poster)
    let w = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
    let h = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
    assert_eq!((w, h), (800, 1131));
    // IEND trailer present
    assert!(png.ends_with(b"IEND\xaeB`\x82"), "IEND trailer missing");
}

#[test]
fn png_is_deterministic() {
    let a = PNGRasterExporter::generate_png_bytes(&spec());
    let b = PNGRasterExporter::generate_png_bytes(&spec());
    assert_eq!(a, b, "PNG output must be byte-identical across runs");
}

#[test]
fn pptx_is_valid_ooxml_zip() {
    let pptx = PPTXPresentationExporter::generate_pptx_bytes(&spec());
    // ZIP local file header signature + [Content_Types].xml first entry
    assert_eq!(&pptx[..4], b"PK\x03\x04", "ZIP local header missing");
    assert!(pptx.windows(b"[Content_Types].xml".len()).any(|w| w == b"[Content_Types].xml"));
    // EOCD signature present
    assert!(pptx.windows(4).any(|w| w == b"PK\x05\x06"), "ZIP EOCD missing");
}

#[test]
fn pptx_is_deterministic() {
    let a = PPTXPresentationExporter::generate_pptx_bytes(&spec());
    let b = PPTXPresentationExporter::generate_pptx_bytes(&spec());
    assert_eq!(a, b, "PPTX output must be byte-identical across runs");
}

#[test]
fn pdf_is_valid_pdf17() {
    let pdf = PDFVectorExporter::generate_pdf_bytes(&spec());
    assert_eq!(&pdf[..8], b"%PDF-1.7", "PDF 1.7 header missing");
    assert!(pdf.windows(b"%%EOF".len()).any(|w| w == b"%%EOF"), "%%EOF trailer missing");
}

#[test]
fn svg_has_no_network_dependency() {
    let svg = SVGVectorRenderer::render(&spec());
    assert!(!svg.contains("@import"), "SVG must not load external fonts");
    assert!(!svg.contains("fonts.googleapis.com"), "SVG must not reference Google Fonts");
    assert!(svg.contains("<svg"), "SVG root missing");
}

#[test]
fn svg_is_deterministic() {
    let a = SVGVectorRenderer::render(&spec());
    let b = SVGVectorRenderer::render(&spec());
    assert_eq!(a, b, "SVG output must be byte-identical across runs");
}
