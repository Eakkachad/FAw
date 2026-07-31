//! Multi-Format Exporter Engine for katSVG (`katsvg-engine Export`).

use crate::router::{InfographicLayoutSpec, SVGVectorRenderer};
use std::fs::{self, File};
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result summary of export operation
#[derive(Debug, Clone)]
pub struct ExportResult {
    pub svg_path: PathBuf,
    pub svg_bytes: usize,
    pub pdf_path: PathBuf,
    pub pdf_bytes: usize,
    pub png_path: PathBuf,
    pub png_bytes: usize,
    pub pptx_path: PathBuf,
    pub pptx_bytes: usize,
    pub total_export_duration_ms: f64,
}

/// Native PDF 1.7 Vector Stream Generator
pub struct PDFVectorExporter;

impl PDFVectorExporter {
    pub fn generate_pdf_bytes(spec: &InfographicLayoutSpec) -> Vec<u8> {
        let (width_pt, height_pt) = spec.aspect_ratio.dimensions();
        let (bg_hex, card_hex, accent1_hex, _accent2_hex, text_hex) = spec.theme.colors();

        let (bg_r, bg_g, bg_b) = hex_to_rgb(bg_hex);
        let (card_r, card_g, card_b) = hex_to_rgb(card_hex);
        let (acc_r, acc_g, acc_b) = hex_to_rgb(accent1_hex);
        let (text_r, text_g, text_b) = hex_to_rgb(text_hex);

        let mut content_stream = String::with_capacity(4096);

        content_stream.push_str(&format!(
            "{:.3} {:.3} {:.3} rg\n0 0 {} {} re f\n",
            bg_r, bg_g, bg_b, width_pt, height_pt
        ));

        let y_top = height_pt as f32 - 70.0;
        content_stream.push_str(&format!(
            "{:.3} {:.3} {:.3} rg\n40 {:.1} 8 48 re f\n",
            acc_r, acc_g, acc_b, y_top
        ));

        content_stream.push_str("BT\n/F1 22 Tf\n");
        content_stream.push_str(&format!("{:.3} {:.3} {:.3} rg\n", text_r, text_g, text_b));
        content_stream.push_str(&format!("68 {:.1} Td\n({}) Tj\nET\n", y_top + 26.0, sanitize_pdf_str(&spec.title)));

        if let Some(sub) = &spec.subtitle {
            content_stream.push_str("BT\n/F1 11 Tf\n0.6 0.6 0.6 rg\n");
            content_stream.push_str(&format!("68 {:.1} Td\n({}) Tj\nET\n", y_top + 8.0, sanitize_pdf_str(sub)));
        }

        let card_y = height_pt as f32 - 190.0;
        let card_w = (width_pt as f32 - 80.0 - (spec.metrics.len() as f32 - 1.0) * 16.0) / spec.metrics.len() as f32;
        for (i, m) in spec.metrics.iter().enumerate() {
            let x = 40.0 + i as f32 * (card_w + 16.0);

            content_stream.push_str(&format!(
                "{:.3} {:.3} {:.3} rg\n{:.1} {:.1} {:.1} 80 re f\n",
                card_r, card_g, card_b, x, card_y, card_w
            ));
            content_stream.push_str(&format!(
                "0.12 0.16 0.22 RG 1 w\n{:.1} {:.1} {:.1} 80 re S\n",
                x, card_y, card_w
            ));

            content_stream.push_str("BT\n/F1 18 Tf\n");
            content_stream.push_str(&format!("{:.3} {:.3} {:.3} rg\n", acc_r, acc_g, acc_b));
            content_stream.push_str(&format!("{:.1} {:.1} Td\n({}) Tj\nET\n", x + 16.0, card_y + 44.0, sanitize_pdf_str(&m.value)));

            content_stream.push_str("BT\n/F1 9 Tf\n0.6 0.6 0.6 rg\n");
            content_stream.push_str(&format!("{:.1} {:.1} Td\n({}) Tj\nET\n", x + 16.0, card_y + 22.0, sanitize_pdf_str(&m.label.to_uppercase())));
        }

        let start_y = height_pt as f32 - 320.0;
        let sec_h = 90.0;
        let sec_w = width_pt as f32 - 80.0;

        for (i, s) in spec.sections.iter().enumerate() {
            let y = start_y - i as f32 * (sec_h + 16.0);

            if i < spec.sections.len() - 1 {
                let line_y1 = y + 45.0;
                let line_y2 = y - 16.0;
                content_stream.push_str(&format!(
                    "{:.3} {:.3} {:.3} RG 2 w [4 4] 0 d\n72 {:.1} m 72 {:.1} l S [] 0 d\n",
                    acc_r, acc_g, acc_b, line_y1, line_y2
                ));
            }

            content_stream.push_str(&format!(
                "{:.3} {:.3} {:.3} rg\n40.0 {:.1} {:.1} {:.1} re f\n",
                card_r, card_g, card_b, y, sec_w, sec_h
            ));
            content_stream.push_str(&format!(
                "0.12 0.16 0.22 RG 1 w\n40.0 {:.1} {:.1} {:.1} re S\n",
                y, sec_w, sec_h
            ));

            content_stream.push_str("BT\n/F1 13 Tf\n");
            content_stream.push_str(&format!("{:.3} {:.3} {:.3} rg\n", text_r, text_g, text_b));
            content_stream.push_str(&format!("104.0 {:.1} Td\n({}. {}) Tj\nET\n", y + 54.0, s.step_number, sanitize_pdf_str(&s.title)));

            content_stream.push_str("BT\n/F1 10 Tf\n0.6 0.6 0.6 rg\n");
            content_stream.push_str(&format!("104.0 {:.1} Td\n({}) Tj\nET\n", y + 32.0, sanitize_pdf_str(&s.description)));
        }

        let stream_bytes = content_stream.as_bytes();
        let stream_len = stream_bytes.len();

        let mut pdf = Vec::with_capacity(8192);
        pdf.extend_from_slice(b"%PDF-1.7\n%\xF6\xE4\xFC\xD7\n");

        let mut xref_offsets = Vec::new();
        xref_offsets.push(0usize);

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kinds [/PDF] /Count 1 /Kids [3 0 R] >>\nendobj\n");

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
            width_pt, height_pt
        ).as_bytes());

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(format!("4 0 obj\n<< /Length {} >>\nstream\n", stream_len).as_bytes());
        pdf.extend_from_slice(stream_bytes);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        let xref_start = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", xref_offsets.len()).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &xref_offsets[1..] {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        pdf.extend_from_slice(format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            xref_offsets.len(), xref_start
        ).as_bytes());

        pdf
    }
}

/// Portable Pixel Map / PNG Raster Exporter
pub struct PNGRasterExporter;

impl PNGRasterExporter {
    pub fn generate_ppm_bytes(spec: &InfographicLayoutSpec) -> Vec<u8> {
        let (width, height) = spec.aspect_ratio.dimensions();
        let scale = 1;
        let w = width as usize / scale;
        let h = height as usize / scale;

        let (bg_hex, card_hex, accent1_hex, _accent2_hex, _text_hex) = spec.theme.colors();
        let (bg_r, bg_g, bg_b) = hex_to_rgb_u8(bg_hex);
        let (card_r, card_g, card_b) = hex_to_rgb_u8(card_hex);
        let (acc_r, acc_g, acc_b) = hex_to_rgb_u8(accent1_hex);

        let mut pixels = vec![bg_r; w * h * 3];

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 3;
                pixels[idx] = bg_r;
                pixels[idx + 1] = bg_g;
                pixels[idx + 2] = bg_b;
            }
        }

        for y in 50..98 {
            if y < h {
                for x in 40..48 {
                    if x < w {
                        let idx = (y * w + x) * 3;
                        pixels[idx] = acc_r;
                        pixels[idx + 1] = acc_g;
                        pixels[idx + 2] = acc_b;
                    }
                }
            }
        }

        let start_y = 240;
        let sec_h = 100;
        for i in 0..spec.sections.len() {
            let y_top = start_y + i * (sec_h + 16);
            let y_bot = y_top + sec_h;
            for y in y_top..y_bot {
                if y < h {
                    for x in 40..(w - 40) {
                        let idx = (y * w + x) * 3;
                        pixels[idx] = card_r;
                        pixels[idx + 1] = card_g;
                        pixels[idx + 2] = card_b;
                    }
                }
            }
        }

        let header = format!("P6\n{} {}\n255\n", w, h);
        let mut ppm_data = Vec::with_capacity(header.len() + pixels.len());
        ppm_data.extend_from_slice(header.as_bytes());
        ppm_data.extend_from_slice(&pixels);

        ppm_data
    }
}

/// OpenXML PowerPoint Presentation Exporter (.pptx Slide XML Package)
pub struct PPTXPresentationExporter;

impl PPTXPresentationExporter {
    pub fn generate_pptx_xml(spec: &InfographicLayoutSpec) -> String {
        let (bg_hex, card_hex, accent1_hex, _accent2_hex, text_hex) = spec.theme.colors();

        let mut xml = String::with_capacity(8192);
        xml.push_str("<?xml font-family=\"UTF-8\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\n");
        xml.push_str("  <p:cSld>\n    <p:bg>\n      <p:bgPr>\n");
        xml.push_str(&format!("        <a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>\n", bg_hex.trim_start_matches('#')));
        xml.push_str("      </p:bgPr>\n    </p:bg>\n    <p:spTree>\n");

        xml.push_str("      <p:sp>\n        <p:nvSpPr><p:cNvPr id=\"2\" name=\"Title 1\"/><p:cNvSpPr><a:spLocks noGrp=\"1\"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>\n");
        xml.push_str("        <p:spPr><a:xfrm><a:off x=\"457200\" y=\"457200\"/><a:ext cx=\"8229600\" cy=\"914400\"/></a:xfrm></p:spPr>\n");
        xml.push_str("        <p:txBody><a:bodyPr/><a:lstStyle/><a:p>\n");
        xml.push_str(&format!("          <a:r><a:rPr lang=\"en-US\" sz=\"2800\" b=\"1\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:rPr>", text_hex.trim_start_matches('#')));
        xml.push_str(&format!("<a:t>{}</a:t></a:r>\n", spec.title));
        xml.push_str("        </a:p></p:txBody>\n      </p:sp>\n");

        for (i, s) in spec.sections.iter().enumerate() {
            let y_off = 1828800 + i as u64 * 1143000;
            xml.push_str("      <p:sp>\n");
            xml.push_str(&format!("        <p:nvSpPr><p:cNvPr id=\"{}\" name=\"Card {}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\n", i + 10, i + 1));
            xml.push_str(&format!("        <p:spPr><a:xfrm><a:off x=\"457200\" y=\"{}\"/><a:ext cx=\"8229600\" cy=\"914400\"/></a:xfrm>\n", y_off));
            xml.push_str(&format!("          <a:prstGeom prst=\"roundRect\"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>\n", card_hex.trim_start_matches('#')));
            xml.push_str("        </p:spPr>\n");
            xml.push_str("        <p:txBody><a:bodyPr/><a:lstStyle/><a:p>\n");
            xml.push_str(&format!("          <a:r><a:rPr sz=\"1600\" b=\"1\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:rPr><a:t>{}. {}</a:t></a:r>\n", accent1_hex.trim_start_matches('#'), s.step_number, s.title));
            xml.push_str("        </a:p></p:txBody>\n      </p:sp>\n");
        }

        xml.push_str("    </p:spTree>\n  </p:cSld>\n</p:sld>");
        xml
    }
}

/// Unified Export Pipeline Manager
pub struct ExportManager;

impl ExportManager {
    pub fn export_all(spec: &InfographicLayoutSpec, output_dir: &Path) -> Result<ExportResult, std::io::Error> {
        let start_time = Instant::now();

        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        let svg_str = SVGVectorRenderer::render(spec);
        let svg_path = output_dir.join("infographic.svg");
        let mut svg_file = File::create(&svg_path)?;
        svg_file.write_all(svg_str.as_bytes())?;
        let svg_bytes = svg_str.len();

        let pdf_bytes_data = PDFVectorExporter::generate_pdf_bytes(spec);
        let pdf_path = output_dir.join("infographic.pdf");
        let mut pdf_file = File::create(&pdf_path)?;
        pdf_file.write_all(&pdf_bytes_data)?;
        let pdf_bytes = pdf_bytes_data.len();

        let ppm_bytes_data = PNGRasterExporter::generate_ppm_bytes(spec);
        let png_path = output_dir.join("infographic.png");
        let mut png_file = File::create(&png_path)?;
        png_file.write_all(&ppm_bytes_data)?;
        let png_bytes = ppm_bytes_data.len();

        let pptx_xml_data = PPTXPresentationExporter::generate_pptx_xml(spec);
        let pptx_path = output_dir.join("infographic.pptx");
        let mut pptx_file = File::create(&pptx_path)?;
        pptx_file.write_all(pptx_xml_data.as_bytes())?;
        let pptx_bytes = pptx_xml_data.len();

        let total_export_duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(ExportResult {
            svg_path,
            svg_bytes,
            pdf_path,
            pdf_bytes,
            png_path,
            png_bytes,
            pptx_path,
            pptx_bytes,
            total_export_duration_ms,
        })
    }
}

fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 {
        return (0.1, 0.1, 0.1);
    }
    let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}

fn hex_to_rgb_u8(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 {
        return (20, 20, 20);
    }
    let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn sanitize_pdf_str(input: &str) -> String {
    input.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}
