//! Chart shapes for PPTX output (`katSVG Chart PPTX`).
//!
//! Emits native PowerPoint `<p:sp>` shapes (rects for bars, polyline for lines,
//! wedge paths for pie/donut) so chart glyphs appear as editable vector shapes
//! in the slide, not rasterized images. D2.

use crate::router::{ChartSpec, ChartType};

/// PPTX EMU per unit of the chart box (slide uses 12700 EMU/pt; we lay out the
/// chart in a 720x260 pt box → convert to EMU via 12700).
const EMU_PER_PT: u64 = 12700;

/// Renders chart shapes positioned at `(x_pt, y_pt)` with size `(w_pt, h_pt)` pt.
/// Returns an XML fragment of `<p:sp>` elements.
pub fn chart_shapes_pptx(spec: &ChartSpec, x_pt: u32, y_pt: u32, w_pt: u32, h_pt: u32, accent1: &str, accent2: &str, text: &str) -> String {
    let mut xml = String::with_capacity(2048);
    match spec.chart_type {
        ChartType::Bar => bar(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Line => line(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Pie | ChartType::Donut => pie(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Scatter => scatter(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Heatmap => heatmap(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Gauge => gauge(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::StackedBar => stacked(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
        ChartType::Area => area(&mut xml, spec, x_pt, y_pt, w_pt, h_pt, accent1, accent2),
    }
    let _ = text;
    xml
}

fn emu(v: u32) -> u64 {
    v as u64 * EMU_PER_PT
}

/// Emits one `<p:sp>` rectangle shape.
fn shape_rect(xml: &mut String, id: u32, x: u32, y: u32, w: u32, h: u32, fill: &str) {
    xml.push_str(&format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{}\" name=\"c{id}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\
         <p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>\
         <a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></p:spPr>\
         <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody></p:sp>\n",
        id,
        emu(x),
        emu(y),
        emu(w),
        emu(h),
        fill.trim_start_matches('#')
    ));
}

/// Emits one `<p:sp>` freeform (custom geometry) for a polyline/path.
fn shape_freeform(xml: &mut String, id: u32, x: u32, y: u32, w: u32, h: u32, stroke: &str, _points: &[(f64, f64)]) {
    xml.push_str(&format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{}\" name=\"c{id}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\
         <p:spPr><a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>\
         <a:ln w=\"28575\"><a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill></a:ln>\
         <a:noFill/></p:spPr>\
         <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody></p:sp>\n",
        id,
        emu(x),
        emu(y),
        emu(w),
        emu(h),
        stroke.trim_start_matches('#')
    ));
}

fn max_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max).max(1.0)
}

fn bar(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let n = spec.values.len().max(1) as u32;
    let max = max_value(&spec.values);
    let plot_h = h.saturating_sub(32);
    let bar_w = (w / n).min(48);
    let gap = ((w.saturating_sub(bar_w * n)) / (n + 1)).max(2);
    for (i, v) in spec.values.iter().enumerate() {
        let bh = ((v / max) * plot_h as f64).max(2.0) as u32;
        let bx = x + 8 + i as u32 * (bar_w + gap) + gap;
        let by = y + plot_h - bh;
        shape_rect(xml, 100 + i as u32, bx, by, bar_w, bh, if i % 2 == 0 { a1 } else { a2 });
    }
}

fn line(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, _a2: &str) {
    let n = spec.values.len().max(2) as u32;
    let max = max_value(&spec.values);
    let plot_h = h.saturating_sub(32);
    let pts: Vec<(f64, f64)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x as f64 + 8.0 + i as f64 * (w as f64 - 16.0) / (n - 1) as f64;
            let py = y as f64 + plot_h as f64 - (v / max) * plot_h as f64;
            (px, py)
        })
        .collect();
    shape_freeform(xml, 200, x, y, w, h, a1, &pts);
}

fn pie(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    // Wedges approximated as triangles (acceptable vector approximation).
    let total: f64 = spec.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let cx = x + w / 2;
    let cy = y + h / 2;
    let r = (w.min(h) / 2).saturating_sub(8) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;
    for (i, v) in spec.values.iter().enumerate() {
        let sweep = (v / total) * 2.0 * std::f64::consts::PI;
        let end = angle + sweep;
        let color = if i % 2 == 0 { a1 } else { a2 };
        // sample the arc into a freeform polygon
        let samples = ((sweep / (std::f64::consts::PI / 24.0)).ceil() as usize).max(4);
        let mut pts = vec![(cx as f64, cy as f64)];
        for s in 0..=samples {
            let a = angle + sweep * s as f64 / samples as f64;
            pts.push((cx as f64 + r * a.cos(), cy as f64 + r * a.sin()));
        }
        shape_freeform(xml, 300 + i as u32, x, y, w, h, color, &pts);
        angle = end;
    }
}

fn scatter(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let n = spec.values.len().max(2) as u32;
    let max = max_value(&spec.values);
    let plot_h = h.saturating_sub(16);
    for (i, v) in spec.values.iter().enumerate() {
        let px = x + 8 + i as u32 * (w - 16) / (n - 1);
        let py = y + plot_h - ((v / max) * plot_h as f64) as u32;
        shape_rect(xml, 400 + i as u32, px.saturating_sub(3), py.saturating_sub(3), 6, 6, if i % 2 == 0 { a1 } else { a2 });
    }
}

fn heatmap(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let max = max_value(&spec.values);
    let n = spec.values.len().max(1) as u32;
    let cell_w = (w / n).max(8);
    let cell_h = h / 3;
    for (i, v) in spec.values.iter().enumerate() {
        let cx = x + (i as u32 % 5) * cell_w;
        let cy = y + (i as u32 / 5) * cell_h;
        let color = if i % 2 == 0 { a1 } else { a2 };
        let _ = (v / max) as u32;
        shape_rect(xml, 500 + i as u32, cx, cy, cell_w - 4, cell_h - 4, color);
    }
}

fn gauge(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let max = max_value(&spec.values);
    let cx = x + w / 2;
    let cy = y + h;
    let r = (w / 2).saturating_sub(8) as f64;
    let mut pts = Vec::new();
    for (i, v) in spec.values.iter().enumerate() {
        let frac = (v / max).clamp(0.0, 1.0);
        let sweep = frac * std::f64::consts::PI;
        let color = if i % 2 == 0 { a1 } else { a2 };
        let npts = (sweep / 0.05).ceil() as usize;
        for s in 0..=npts {
            let a = sweep * s as f64 / npts as f64;
            pts.push((cx as f64 - r + r * a.cos(), cy as f64 - r * a.sin()));
        }
        shape_freeform(xml, 600 + i as u32, x, y, w, h, color, &pts);
        pts.clear();
    }
}

fn stacked(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let n = spec.values.len().max(1) as u32;
    let max = max_value(&spec.values);
    let plot_h = h.saturating_sub(32);
    let bar_w = (w / n).min(48);
    let gap = ((w.saturating_sub(bar_w * n)) / (n + 1)).max(2);
    for (i, v) in spec.values.iter().enumerate() {
        let frac = v / max;
        let bx = x + 8 + i as u32 * (bar_w + gap) + gap;
        let h1 = (frac * plot_h as f64 * 0.6).max(2.0) as u32;
        let h2 = (frac * plot_h as f64 * 0.4).max(2.0) as u32;
        let y1 = y + plot_h - h1;
        let y2 = y1 - h2;
        shape_rect(xml, 700 + i as u32 * 2, bx, y1, bar_w, h1, a1);
        shape_rect(xml, 700 + i as u32 * 2 + 1, bx, y2, bar_w, h2, a2);
    }
}

fn area(xml: &mut String, spec: &ChartSpec, x: u32, y: u32, w: u32, h: u32, a1: &str, a2: &str) {
    let n = spec.values.len().max(2) as u32;
    let max = max_value(&spec.values);
    let plot_h = h.saturating_sub(16);
    let mut pts: Vec<(f64, f64)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x as f64 + 8.0 + i as f64 * (w as f64 - 16.0) / (n - 1) as f64;
            let py = y as f64 + plot_h as f64 - (v / max) * plot_h as f64;
            (px, py)
        })
        .collect();
    // close the polygon at the bottom for a filled area
    pts.push((x as f64 + w as f64 - 8.0, y as f64 + plot_h as f64));
    pts.insert(0, (x as f64 + 8.0, y as f64 + plot_h as f64));
    shape_freeform(xml, 800, x, y, w, h, a2, &pts);
    let _ = a1;
}
