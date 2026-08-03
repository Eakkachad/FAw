//! Multi-Format Exporter Engine for katSVG (`katsvg-engine Export`).
//!
//! Generates real, valid binary/document artifacts from an `InfographicLayoutSpec`:
//! - **SVG**  — native vector (see `router.rs::SVGVectorRenderer`)
//! - **PDF**  — valid PDF 1.7 stream (base-14 Helvetica)
//! - **PNG**  — valid PNG (signature + IHDR + IDAT + IEND, CRC-checked) via `png`
//! - **PPTX** — valid OOXML ZIP package (`[Content_Types].xml`, rels, slides,
//!             slideMaster, slideLayout, theme, docProps)

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
        let (bg_hex, card_hex, accent1_hex, accent2_hex, text_hex) = spec.theme.colors();

        let (bg_r, bg_g, bg_b) = hex_to_rgb(bg_hex);
        let (a2r, a2g, a2b) = hex_to_rgb(accent2_hex);
        let (card_r, card_g, card_b) = hex_to_rgb(card_hex);
        let (acc_r, acc_g, acc_b) = hex_to_rgb(accent1_hex);
        let (text_r, text_g, text_b) = hex_to_rgb(text_hex);

        // F2: determine whether we need the embedded Thai (Noto Sans Thai) font.
        let needs_thai = crate::font::has_non_ascii(&spec.title)
            || spec.subtitle.as_deref().is_some_and(crate::font::has_non_ascii)
            || spec.metrics.iter().any(|m| crate::font::has_non_ascii(&m.label) || crate::font::has_non_ascii(&m.value))
            || spec.sections.iter().any(|s| crate::font::has_non_ascii(&s.title));

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

        pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 22.0, text_r, text_g, text_b, 68.0, y_top + 26.0, &spec.title);

        if let Some(sub) = &spec.subtitle {
            pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 11.0, 0.6, 0.6, 0.6, 68.0, y_top + 8.0, sub);
        }

        let card_y = height_pt as f32 - 190.0;
        let card_w = if spec.metrics.is_empty() {
            0.0
        } else {
            (width_pt as f32 - 80.0 - (spec.metrics.len() as f32 - 1.0) * 16.0) / spec.metrics.len() as f32
        };
        for (i, m) in spec.metrics.iter().enumerate() {
            let x = 40.0 + i as f32 * (card_w + 16.0);

            if card_w <= 0.0 {
                continue;
            }

            content_stream.push_str(&format!(
                "{:.3} {:.3} {:.3} rg\n{:.1} {:.1} {:.1} 80 re f\n",
                card_r, card_g, card_b, x, card_y, card_w
            ));
            content_stream.push_str(&format!(
                "0.12 0.16 0.22 RG 1 w\n{:.1} {:.1} {:.1} 80 re S\n",
                x, card_y, card_w
            ));

            pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 18.0, acc_r, acc_g, acc_b, x + 16.0, card_y + 44.0, &m.value);

            pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 9.0, 0.6, 0.6, 0.6, x + 16.0, card_y + 22.0, &m.label.to_uppercase());

            // Icon glyph at top-right of the card (F3)
            crate::icon_raster::draw_icon_pdf(&mut content_stream, x + card_w - 20.0, card_y + 62.0, 1.5, a2r, a2g, a2b, &m.icon);
        }

        // Chart region (SVG y 240..500 → PDF bottom-left y = height-500, h=260) — D3
        if let Some(chart) = &spec.chart {
            let (_bgc, _cbc, a1_hex, a2_hex, _t_hex) = spec.theme.colors();
            let (a1r, a1g, a1b) = hex_to_rgb(a1_hex);
            let (a2r, a2g, a2b) = hex_to_rgb(a2_hex);
            crate::chart_pdf::chart_ops_pdf(
                chart,
                &mut content_stream,
                40.0,
                height_pt as f32 - 500.0,
                width_pt as f32 - 80.0,
                260.0,
                (a1r, a1g, a1b),
                (a2r, a2g, a2b),
            );
        }

        let start_y = if spec.chart.is_some() {
            height_pt as f32 - 620.0
        } else {
            height_pt as f32 - 320.0
        };
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

            let step_label = format!("{}. {}", s.step_number, s.title);
            pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 13.0, text_r, text_g, text_b, 104.0, y + 54.0, &step_label);

            pdf_emit_text(&mut content_stream, spec, needs_thai, "F1", 10.0, 0.6, 0.6, 0.6, 104.0, y + 32.0, &s.description);
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
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /Font << /F1 5 0 R{} >> >> >>\nendobj\n",
            width_pt, height_pt,
            if needs_thai { " /F2 6 0 R".to_string() } else { String::new() }
        ).as_bytes());

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(format!("4 0 obj\n<< /Length {} >>\nstream\n", stream_len).as_bytes());
        pdf.extend_from_slice(stream_bytes);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        xref_offsets.push(pdf.len());
        pdf.extend_from_slice(b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        // F2: embedded Noto Sans Thai as a Type0 CID font (only when Thai text present)
        if needs_thai {
            xref_offsets.push(pdf.len());
            pdf.extend_from_slice(format!(
                "6 0 obj\n<< /Type /Font /Subtype /Type0 /BaseFont /NotoSansThai /Encoding /Identity-H /DescendantFonts [7 0 R] >>\nendobj\n"
            ).as_bytes());

            xref_offsets.push(pdf.len());
            pdf.extend_from_slice(format!(
                "7 0 obj\n<< /Type /Font /Subtype /CIDFontType2 /BaseFont /NotoSansThai /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor 8 0 R /CIDToGIDMap /Identity /DW 1000 >>\nendobj\n"
            ).as_bytes());

            xref_offsets.push(pdf.len());
            pdf.extend_from_slice(format!(
                "8 0 obj\n<< /Type /FontDescriptor /FontName /NotoSansThai /Flags 4 /FontBBox [{} {} {} {}] /ItalicAngle 0 /Ascent {} /Descent {} /CapHeight {} /StemV 80 /FontFile2 9 0 R >>\nendobj\n",
                crate::pdf_font::FONT_BBOX[0], crate::pdf_font::FONT_BBOX[1], crate::pdf_font::FONT_BBOX[2], crate::pdf_font::FONT_BBOX[3],
                crate::pdf_font::FONT_ASCENT, crate::pdf_font::FONT_DESCENT, crate::pdf_font::FONT_CAPHEIGHT
            ).as_bytes());

            xref_offsets.push(pdf.len());
            let font_bytes = crate::pdf_font::font_file_bytes();
            pdf.extend_from_slice(format!("9 0 obj\n<< /Length {} >>\nstream\n", font_bytes.len()).as_bytes());
            pdf.extend_from_slice(font_bytes);
            pdf.extend_from_slice(b"\nendstream\nendobj\n");
        }

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

/// Real PNG Raster Exporter (valid signature / IHDR / IDAT / IEND via `png` crate)
pub struct PNGRasterExporter;

impl PNGRasterExporter {
    /// Rasterizes the layout of `spec` into a valid PNG byte buffer, including
    /// readable text (title, metric values, section titles) via the embedded
    /// Inter font rasterizer (`text.rs`).
    pub fn generate_png_bytes(spec: &InfographicLayoutSpec) -> Vec<u8> {
        let (width, height) = spec.aspect_ratio.dimensions();
        let w = width as usize;
        let h = height as usize;

        let (bg_hex, card_hex, accent1_hex, accent2_hex, text_hex) = spec.theme.colors();
        let (bg_r, bg_g, bg_b) = hex_to_rgb_u8(bg_hex);
        let (card_r, card_g, card_b) = hex_to_rgb_u8(card_hex);
        let (acc_r, acc_g, acc_b) = hex_to_rgb_u8(accent1_hex);
        let (acc2_r, acc2_g, acc2_b) = hex_to_rgb_u8(accent2_hex);
        let (text_r, text_g, text_b) = hex_to_rgb_u8(text_hex);

        let mut px = vec![bg_r, bg_g, bg_b];
        px = px.repeat(w * h);

        let renderer = crate::text::TextRenderer::new();

        // Title accent bar (mirrors SVG translate(40,50) rect 8x48)
        fill_rect(&mut px, w, 40, 50, 48, 98, (acc_r, acc_g, acc_b));

        // Title + subtitle text
        renderer.draw_text(&mut px, w, h, 60.0, 78.0, 28.0, (text_r, text_g, text_b), &spec.title);
        if let Some(sub) = &spec.subtitle {
            renderer.draw_text(&mut px, w, h, 60.0, 100.0, 14.0, (154, 163, 175), sub);
        }

        // Metric cards (mirrors SVG cards at y=130, height=80)
        let card_w = if spec.metrics.is_empty() {
            0
        } else {
            (width - 80 - (spec.metrics.len() as u32 - 1) * 16) / spec.metrics.len() as u32
        };
        for (i, m) in spec.metrics.iter().enumerate() {
            if card_w == 0 {
                break;
            }
            let x = 40 + i as u32 * (card_w + 16);
            fill_rect(&mut px, w, x as usize, 130, (x + card_w) as usize, 210, (card_r, card_g, card_b));
            // metric value + label (real text now, P5)
            renderer.draw_text(&mut px, w, h, (x + 16) as f32, 172.0, 24.0, (acc_r, acc_g, acc_b), &m.value);
            renderer.draw_text(&mut px, w, h, (x + 16) as f32, 194.0, 11.0, (154, 163, 175), &m.label.to_uppercase());
            // Icon glyph at top-right of the card (F3)
            let icon_cx = (x + card_w - 20) as usize;
            let icon_cy = 160usize;
            crate::icon_raster::draw_icon_raster(&mut px, w, h, icon_cx, icon_cy, 2, (acc2_r, acc2_g, acc2_b), &m.icon);
        }

        // Chart region (mirrors SVG chart at y=240, height 260) — D1 parity
        if let Some(chart) = &spec.chart {
            let chart_w = width as usize - 80;
            let chart_h = 260usize;
            crate::chart_raster::draw_chart_raster(
                chart,
                &mut px,
                w,
                h,
                40,
                240,
                chart_w,
                chart_h,
                (acc_r, acc_g, acc_b),
                hex_to_rgb_u8(accent2_hex),
                (text_r, text_g, text_b),
            );
        }

        // Section cards + step connectors (mirrors SVG at start_y=240, sec_h=100)
        let start_y = if spec.chart.is_some() { 520 } else { 240 };
        let sec_h = 100usize;
        for (i, s) in spec.sections.iter().enumerate() {
            let y = start_y + i * (sec_h + 16);

            if i < spec.sections.len() - 1 {
                let line_y1 = y + 48;
                let line_y2 = y + sec_h + 16;
                fill_rect(&mut px, w, 70, line_y1, 74, line_y2, (acc_r, acc_g, acc_b));
            }

            fill_rect(&mut px, w, 40, y, w - 40, y + sec_h, (card_r, card_g, card_b));
            // step number circle + text
            fill_rect(&mut px, w, 16, y + 26, 46, y + 46, (acc_r, acc_g, acc_b));
            renderer.draw_text(&mut px, w, h, (24.5) as f32, y as f32 + 44.0, 13.0, (bg_r, bg_g, bg_b), &s.step_number.to_string());
            // title + description
            renderer.draw_text(&mut px, w, h, 60.0, y as f32 + 38.0, 16.0, (text_r, text_g, text_b), &s.title);
            renderer.draw_text(&mut px, w, h, 60.0, y as f32 + 60.0, 12.0, (154, 163, 175), &s.description);
        }

        let mut out = Vec::with_capacity(px.len() / 3 + 1024);
        {
            let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut writer = enc.write_header().expect("PNG header write failed");
            writer.write_image_data(&px).expect("PNG data write failed");
        }
        out
    }
}

fn fill_rect(buf: &mut [u8], w: usize, x0: usize, y0: usize, x1: usize, y1: usize, rgb: (u8, u8, u8)) {
    let x1 = x1.min(w);
    let (r, g, b) = rgb;
    for y in y0..y1 {
        if y >= buf.len() / (3 * w) {
            break;
        }
        let row = y * w * 3;
        for x in x0..x1 {
            let i = row + x * 3;
            if i + 2 < buf.len() {
                buf[i] = r;
                buf[i + 1] = g;
                buf[i + 2] = b;
            }
        }
    }
}

/// Valid OpenXML PowerPoint Presentation Package Exporter (.pptx ZIP container)
pub struct PPTXPresentationExporter;

impl PPTXPresentationExporter {
    /// Builds a complete, valid `.pptx` package as a ZIP byte buffer.
    pub fn generate_pptx_bytes(spec: &InfographicLayoutSpec) -> Vec<u8> {
        let (bg_hex, card_hex, accent1_hex, accent2_hex, text_hex) = spec.theme.colors();
        let (w_pt, h_pt) = spec.aspect_ratio.dimensions();

        // Presentation slide size in EMU (1 pt = 12700 EMU)
        let sld_cx = w_pt * 12700;
        let sld_cy = h_pt * 12700;

        // ── Slide body: title + metric cards + section cards ─────────────────
        let mut slide_xml = String::with_capacity(8192);
        slide_xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        slide_xml.push_str("<p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\n");
        slide_xml.push_str("  <p:cSld>\n    <p:bg>\n      <p:bgPr>\n");
        slide_xml.push_str(&format!("        <a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>\n", bg_hex.trim_start_matches('#')));
        slide_xml.push_str("      </p:bgPr>\n    </p:bg>\n    <p:spTree>\n");
        slide_xml.push_str("      <p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\n");

        slide_xml.push_str("      <p:sp>\n        <p:nvSpPr><p:cNvPr id=\"2\" name=\"Title 1\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\n");
        slide_xml.push_str("        <p:spPr><a:xfrm><a:off x=\"457200\" y=\"457200\"/><a:ext cx=\"8229600\" cy=\"914400\"/></a:xfrm></p:spPr>\n");
        slide_xml.push_str("        <p:txBody><a:bodyPr/><a:lstStyle/><a:p>\n");
        slide_xml.push_str(&format!("          <a:r><a:rPr lang=\"en-US\" sz=\"2800\" b=\"1\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:rPr>", text_hex.trim_start_matches('#')));
        slide_xml.push_str(&format!("<a:t>{}</a:t></a:r>\n", escape_xml(&spec.title)));
        slide_xml.push_str("        </a:p></p:txBody>\n      </p:sp>\n");

        // Metric cards (F3: PPTX previously had no metric cards)
        if !spec.metrics.is_empty() {
            let n = spec.metrics.len() as f32;
            let card_w_emu = 8229600u64;
            let gap_emu = 120000u64;
            let total_w = card_w_emu * n as u64 + gap_emu * (n as u64 - 1);
            let start_x = (12192000u64.saturating_sub(total_w)) / 2;
            for (i, m) in spec.metrics.iter().enumerate() {
                let x_off = start_x + i as u64 * (card_w_emu + gap_emu);
                let id = 30 + i as u32;
                slide_xml.push_str("      <p:sp>\n");
                slide_xml.push_str(&format!("        <p:nvSpPr><p:cNvPr id=\"{}\" name=\"Metric {}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\n", id, i + 1));
                slide_xml.push_str(&format!("        <p:spPr><a:xfrm><a:off x=\"{}\" y=\"1371600\"/><a:ext cx=\"{}\" cy=\"914400\"/></a:xfrm>\n", x_off, card_w_emu));
                slide_xml.push_str(&format!("          <a:prstGeom prst=\"roundRect\"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>\n", card_hex.trim_start_matches('#')));
                slide_xml.push_str("        </p:spPr>\n");
                slide_xml.push_str("        <p:txBody><a:bodyPr/><a:lstStyle/><a:p>\n");
                slide_xml.push_str(&format!("          <a:r><a:rPr sz=\"2000\" b=\"1\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r>\n", accent1_hex.trim_start_matches('#'), escape_xml(&m.value)));
                slide_xml.push_str("        </a:p></p:txBody>\n      </p:sp>\n");
                // icon mark (stroke rect approximating the glyph)
                slide_xml.push_str("      <p:sp>\n");
                slide_xml.push_str(&format!("        <p:nvSpPr><p:cNvPr id=\"{}\" name=\"Icon {}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\n", id + 100, i + 1));
                slide_xml.push_str(&format!("        <p:spPr><a:xfrm><a:off x=\"{}\" y=\"1676400\"/><a:ext cx=\"274320\" cy=\"274320\"/></a:xfrm>\n", x_off + card_w_emu - 400000));
                slide_xml.push_str("          <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:ln w=\"38100\"><a:solidFill><a:srgbClr val=\"");
                slide_xml.push_str(accent2_hex.trim_start_matches('#'));
                slide_xml.push_str("\"/></a:solidFill></a:ln><a:noFill/></p:spPr>\n        <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody>\n      </p:sp>\n");
            }
        }

        // Chart shapes (native PPTX vectors, D2 parity)
        if let Some(chart) = &spec.chart {
            let (_bg, _cbg, a1, a2, tx) = spec.theme.colors();
            let shapes = crate::chart_pptx::chart_shapes_pptx(
                chart, 40, 240, 720, 260, a1, a2, tx,
            );
            slide_xml.push_str(&shapes);
        }

        for (i, s) in spec.sections.iter().enumerate() {
            let y_off = 1828800 + i as u64 * 1143000;
            slide_xml.push_str("      <p:sp>\n");
            slide_xml.push_str(&format!("        <p:nvSpPr><p:cNvPr id=\"{}\" name=\"Card {}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\n", i + 10, i + 1));
            slide_xml.push_str(&format!("        <p:spPr><a:xfrm><a:off x=\"457200\" y=\"{}\"/><a:ext cx=\"8229600\" cy=\"914400\"/></a:xfrm>\n", y_off));
            slide_xml.push_str(&format!("          <a:prstGeom prst=\"roundRect\"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>\n", card_hex.trim_start_matches('#')));
            slide_xml.push_str("        </p:spPr>\n");
            slide_xml.push_str("        <p:txBody><a:bodyPr/><a:lstStyle/><a:p>\n");
            slide_xml.push_str(&format!("          <a:r><a:rPr sz=\"1600\" b=\"1\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:rPr><a:t>{}. {}</a:t></a:r>\n", accent1_hex.trim_start_matches('#'), s.step_number, escape_xml(&s.title)));
            slide_xml.push_str("        </a:p></p:txBody>\n      </p:sp>\n");
        }

        slide_xml.push_str("    </p:spTree>\n  </p:cSld>\n  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>\n</p:sld>");

        // ── Fixed OOXML parts ────────────────────────────────────────────────
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#;

        let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;

        let presentation_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
  <p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst>
  <p:sldSz cx="{}" cy="{}"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
            sld_cx, sld_cy
        );

        let presentation_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#;

        let slide_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;

        let slide_layout_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank">
  <p:cSld name="Blank"><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
  </p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#;

        let slide_layout_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;

        let slide_master_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld name="Master"><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
  </p:spTree></p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId2"/></p:sldLayoutIdLst>
</p:sldMaster>"#;

        let slide_master_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;

        let theme_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="katSVG Theme">
  <a:themeElements>
    <a:clrScheme name="katSVG">
      <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
      <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="1F497D"/></a:dk2>
      <a:lt2><a:srgbClr val="EEECE1"/></a:lt2>
      <a:accent1><a:srgbClr val="4F81BD"/></a:accent1>
      <a:accent2><a:srgbClr val="C0504D"/></a:accent2>
      <a:accent3><a:srgbClr val="9BBB59"/></a:accent3>
      <a:accent4><a:srgbClr val="8064A2"/></a:accent4>
      <a:accent5><a:srgbClr val="4BACC6"/></a:accent5>
      <a:accent6><a:srgbClr val="F79646"/></a:accent6>
      <a:hlink><a:srgbClr val="0000FF"/></a:hlink>
      <a:folHlink><a:srgbClr val="800080"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst>
      <a:lnStyleLst><a:ln w="9525" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="25400" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln><a:ln w="38100" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:prstDash val="solid"/></a:ln></a:lnStyleLst>
      <a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst>
      <a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#;

        let theme_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#;

        let core_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>katSVG Infographic</dc:title>
  <dc:creator>katSVG Engine</dc:creator>
  <cp:lastModifiedBy>katSVG Engine</cp:lastModifiedBy>
  <dcterms:created xsi:type="dcterms:W3CDTF">2026-01-01T00:00:00Z</dcterms:created>
  <dcterms:modified xsi:type="dcterms:W3CDTF">2026-01-01T00:00:00Z</dcterms:modified>
</cp:coreProperties>"#;

        let app_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <Application>katSVG Engine</Application>
  <SlideCount>1</SlideCount>
  <NotesSlideCount>0</NotesSlideCount>
  <PresentationFormat>Custom</PresentationFormat>
</Properties>"#;

        let entries: Vec<(&str, &[u8])> = vec![
            ("[Content_Types].xml", content_types.as_bytes()),
            ("_rels/.rels", root_rels.as_bytes()),
            ("docProps/app.xml", app_xml.as_bytes()),
            ("docProps/core.xml", core_xml.as_bytes()),
            ("ppt/presentation.xml", presentation_xml.as_bytes()),
            ("ppt/_rels/presentation.xml.rels", presentation_rels.as_bytes()),
            ("ppt/slides/slide1.xml", slide_xml.as_bytes()),
            ("ppt/slides/_rels/slide1.xml.rels", slide_rels.as_bytes()),
            ("ppt/slideLayouts/slideLayout1.xml", slide_layout_xml.as_bytes()),
            ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", slide_layout_rels.as_bytes()),
            ("ppt/slideMasters/slideMaster1.xml", slide_master_xml.as_bytes()),
            ("ppt/slideMasters/_rels/slideMaster1.xml.rels", slide_master_rels.as_bytes()),
            ("ppt/theme/theme1.xml", theme_xml.as_bytes()),
            ("ppt/theme/_rels/theme1.xml.rels", theme_rels.as_bytes()),
        ];

        zip_package(&entries)
    }
}

/// Minimal, deterministic ZIP writer (STORE method, no compression).
/// Produces a spec-compliant ZIP container with a central directory + EOCD.
fn zip_package(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let crc_table = crc32_table();
    let mut out = Vec::new();
    let mut central = Vec::new();

    for (name, data) in entries {
        let offset = out.len() as u32;
        let name_bytes = name.as_bytes();
        let crc = crc32(&crc_table, data);

        // Local file header
        out.extend_from_slice(&0x04034b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // flags (UTF-8 names)
        out.extend_from_slice(&0u16.to_le_bytes()); // method: stored
        out.extend_from_slice(&0u16.to_le_bytes()); // mod time
        out.extend_from_slice(&0x21u16.to_le_bytes()); // mod date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // comp size
        out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // uncomp size
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(data);

        // Central directory entry
        central.extend_from_slice(&0x02014b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0x0800u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // method: stored
        central.extend_from_slice(&0u16.to_le_bytes()); // mod time
        central.extend_from_slice(&0x21u16.to_le_bytes()); // mod date
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra len
        central.extend_from_slice(&0u16.to_le_bytes()); // comment len
        central.extend_from_slice(&0u16.to_le_bytes()); // disk start
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name_bytes);
    }

    let cd_offset = out.len() as u32;
    out.extend_from_slice(&central);
    let cd_size = central.len() as u32;

    // End of central directory record
    out.extend_from_slice(&0x06054b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len

    out
}

fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    table
}

fn crc32(table: &[u32; 256], data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
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

        let png_bytes_data = PNGRasterExporter::generate_png_bytes(spec);
        let png_path = output_dir.join("infographic.png");
        let mut png_file = File::create(&png_path)?;
        png_file.write_all(&png_bytes_data)?;
        let png_bytes = png_bytes_data.len();

        let pptx_bytes_data = PPTXPresentationExporter::generate_pptx_bytes(spec);
        let pptx_path = output_dir.join("infographic.pptx");
        let mut pptx_file = File::create(&pptx_path)?;
        pptx_file.write_all(&pptx_bytes_data)?;
        let pptx_bytes = pptx_bytes_data.len();

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

/// Emit a PDF text-show operator for `text` at `(x, y)` with font `font_ref`,
/// using the embedded Thai CID font when `text` has non-ASCII glyphs.
#[allow(clippy::too_many_arguments)]
fn pdf_emit_text(
    stream: &mut String,
    spec: &crate::router::InfographicLayoutSpec,
    needs_thai: bool,
    font_ref: &str,
    size: f32,
    r: f32,
    g: f32,
    b: f32,
    x: f32,
    y: f32,
    text: &str,
) {
    stream.push_str("BT\n");
    if needs_thai && crate::font::has_non_ascii(text) {
        stream.push_str(&format!("/F2 {size} Tf\n"));
        stream.push_str(&format!("{:.3} {:.3} {:.3} rg\n", r, g, b));
        stream.push_str(&format!("{:.1} {:.1} Td\n", x, y));
        if crate::font::has_non_ascii(text) {
            let hex = crate::pdf_font::encode_cid_hex(text);
            stream.push_str(&format!("{hex} Tj\n"));
        } else {
            // no cmap → fall back to Latin-safe literal (won't render Thai, won't crash)
            stream.push_str(&format!("({}) Tj\n", sanitize_pdf_str(text)));
        }
    } else {
        stream.push_str(&format!("/{font_ref} {size} Tf\n"));
        stream.push_str(&format!("{:.3} {:.3} {:.3} rg\n", r, g, b));
        stream.push_str(&format!("{:.1} {:.1} Td\n({}) Tj\n", x, y, sanitize_pdf_str(text)));
    }
    stream.push_str("ET\n");
    let _ = spec;
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
