#![allow(clippy::too_many_arguments)]
//! Chart drawing for PDF output (`katSVG Chart PDF`).
//!
//! Emits PDF content-stream operators (rects `re f`, lines `m l S`, arcs via
//! sampled line segments) to draw chart glyphs as vector primitives. PDF uses a
//! bottom-left origin; callers pass the region in PDF coordinates. D3.

use crate::router::{ChartSpec, ChartType};

/// Appends PDF operators drawing the chart inside region
/// `(x, y)` (bottom-left) of size `(w, h)` in PDF user units.
pub fn chart_ops_pdf(
    spec: &ChartSpec,
    stream: &mut String,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1_rgb: (f32, f32, f32),
    a2_rgb: (f32, f32, f32),
) {
    match spec.chart_type {
        ChartType::Bar => bar(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Line => line(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Pie | ChartType::Donut => pie(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Scatter => scatter(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Heatmap => heatmap(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Gauge => gauge(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::StackedBar => stacked(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
        ChartType::Area => area(stream, spec, x, y, w, h, a1_rgb, a2_rgb),
    }
}

fn max_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max).max(1.0)
}

fn rgb(c: (f32, f32, f32)) -> String {
    format!("{:.3} {:.3} {:.3}", c.0, c.1, c.2)
}

fn bar(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let n = spec.values.len().max(1) as f32;
    let max = max_value(&spec.values) as f32;
    let plot_h = h - 20.0;
    let bar_w = (w / n).min(48.0);
    let gap = ((w - bar_w * n) / (n + 1.0)).max(2.0);
    for (i, v) in spec.values.iter().enumerate() {
        let bh = (((*v) as f32 / max) * plot_h).max(2.0);
        let bx = x + 8.0 + i as f32 * (bar_w + gap) + gap;
        let by = y + plot_h - bh;
        let color = if i % 2 == 0 { a1 } else { a2 };
        stream.push_str(&format!(
            "{} rg\n{:.1} {:.1} {:.1} {:.1} re f\n",
            rgb(color),
            bx,
            by,
            bar_w,
            bh
        ));
    }
}

fn line(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let n = spec.values.len().max(2) as f32;
    let max = max_value(&spec.values) as f32;
    let plot_h = h - 20.0;
    let pts: Vec<(f32, f32)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x + 8.0 + i as f32 * (w - 16.0) / (n - 1.0);
            let py = y + plot_h - ((*v) as f32 / max) * plot_h;
            (px, py)
        })
        .collect();
    if pts.is_empty() {
        return;
    }
    stream.push_str(&format!("{} RG 2 w\n", rgb(a1)));
    for pair in pts.windows(2) {
        stream.push_str(&format!(
            "{:.1} {:.1} m {:.1} {:.1} l S\n",
            pair[0].0, pair[0].1, pair[1].0, pair[1].1
        ));
    }
    // point markers
    stream.push_str(&format!("{} rg\n", rgb(a2)));
    for (px, py) in &pts {
        stream.push_str(&format!("{:.1} {:.1} 3 0 360 arc f\n", px, py));
    }
}

fn pie(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let total: f64 = spec.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let cx = (x + w / 2.0) as f64;
    let cy = (y + h / 2.0) as f64;
    let r = ((w.min(h) / 2.0 - 8.0).max(1.0)) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;
    for (i, v) in spec.values.iter().enumerate() {
        let sweep = (v / total) * 2.0 * std::f64::consts::PI;
        let end = angle + sweep;
        let color = if i % 2 == 0 { a1 } else { a2 };
        stream.push_str(&format!("{} rg\n", rgb(color)));
        stream.push_str(&format!("{:.1} {:.1} m\n", cx, cy));
        let samples = ((sweep / (std::f64::consts::PI / 24.0)).ceil() as usize).max(4);
        for s in 0..=samples {
            let a = angle + sweep * s as f64 / samples as f64;
            let px = cx + r * a.cos();
            let py = cy + r * a.sin();
            stream.push_str(&format!("{:.1} {:.1} l\n", px, py));
        }
        stream.push_str("h f\n");
        angle = end;
    }
}

fn scatter(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let n = spec.values.len().max(2) as f32;
    let max = max_value(&spec.values) as f32;
    let plot_h = h - 8.0;
    stream.push_str(&format!("{} rg\n", rgb((0.55, 0.61, 0.71))));
    stream.push_str(&format!(
        "{:.1} {:.1} {:.1} {:.1} re f\n",
        x + 8.0,
        y,
        w - 16.0,
        plot_h
    ));
    for (i, v) in spec.values.iter().enumerate() {
        let px = x + 8.0 + i as f32 * (w - 16.0) / (n - 1.0);
        let py = y + plot_h - ((*v) as f32 / max) * plot_h;
        let color = if i % 2 == 0 { a1 } else { a2 };
        stream.push_str(&format!(
            "{} rg\n{:.1} {:.1} 3 0 360 arc f\n",
            rgb(color),
            px,
            py
        ));
    }
}

fn heatmap(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let max = max_value(&spec.values) as f32;
    let n = spec.values.len().max(1) as f32;
    let cell_w = (w / n).max(8.0);
    let cell_h = h / 3.0;
    for (i, v) in spec.values.iter().enumerate() {
        let intensity = ((*v) as f32 / max).clamp(0.0, 1.0);
        let base = if i % 2 == 0 { a1 } else { a2 };
        let cx = x + (i as f32 % 5.0) * cell_w;
        let cy = y + (i as u32 / 5) as f32 * cell_h;
        let color = (base.0 * intensity, base.1 * intensity, base.2 * intensity);
        stream.push_str(&format!(
            "{} rg\n{:.1} {:.1} {:.1} {:.1} re f\n",
            rgb(color),
            cx,
            cy,
            cell_w - 4.0,
            cell_h - 4.0
        ));
    }
}

fn gauge(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let max = max_value(&spec.values) as f32;
    let cx = (x + w / 2.0) as f64;
    let cy = y as f64;
    let r = (w / 2.0 - 8.0).max(1.0) as f64;
    for (i, v) in spec.values.iter().enumerate() {
        let frac = ((*v) as f32 / max).clamp(0.0, 1.0);
        let sweep = frac as f64 * std::f64::consts::PI;
        let color = if i % 2 == 0 { a1 } else { a2 };
        stream.push_str(&format!("{} RG 6 w\n", rgb(color)));
        let npts = (sweep / 0.05).ceil() as usize;
        let mut first = true;
        for s in 0..=npts {
            let a = sweep * s as f64 / npts as f64;
            let px = cx - r + r * a.cos();
            let py = cy + r * a.sin();
            if first {
                stream.push_str(&format!("{:.1} {:.1} m\n", px, py));
                first = false;
            } else {
                stream.push_str(&format!("{:.1} {:.1} l\n", px, py));
            }
        }
        stream.push_str("S\n");
    }
}

fn stacked(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let n = spec.values.len().max(1) as f32;
    let max = max_value(&spec.values) as f32;
    let plot_h = h - 20.0;
    let bar_w = (w / n).min(48.0);
    let gap = ((w - bar_w * n) / (n + 1.0)).max(2.0);
    for (i, v) in spec.values.iter().enumerate() {
        let frac = (*v) as f32 / max;
        let bx = x + 8.0 + i as f32 * (bar_w + gap) + gap;
        let h1 = (frac * plot_h * 0.6).max(2.0);
        let h2 = (frac * plot_h * 0.4).max(2.0);
        let y1 = y + plot_h - h1;
        let y2 = y1 - h2;
        stream.push_str(&format!(
            "{} rg\n{:.1} {:.1} {:.1} {:.1} re f\n",
            rgb(a1),
            bx,
            y1,
            bar_w,
            h1
        ));
        stream.push_str(&format!(
            "{} rg\n{:.1} {:.1} {:.1} {:.1} re f\n",
            rgb(a2),
            bx,
            y2,
            bar_w,
            h2
        ));
    }
}

fn area(
    stream: &mut String,
    spec: &ChartSpec,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a1: (f32, f32, f32),
    a2: (f32, f32, f32),
) {
    let n = spec.values.len().max(2) as f32;
    let max = max_value(&spec.values) as f32;
    let plot_h = h - 8.0;
    let pts: Vec<(f32, f32)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x + 8.0 + i as f32 * (w - 16.0) / (n - 1.0);
            let py = y + plot_h - ((*v) as f32 / max) * plot_h;
            (px, py)
        })
        .collect();
    if pts.is_empty() {
        return;
    }
    // filled polygon: baseline → points → baseline
    stream.push_str(&format!("{} rg\n", rgb(a2)));
    stream.push_str(&format!("{:.1} {:.1} m\n", x + 8.0, y));
    for (px, py) in &pts {
        stream.push_str(&format!("{:.1} {:.1} l\n", px, py));
    }
    stream.push_str(&format!("{:.1} {:.1} l h f\n", x + w - 8.0, y));
    // stroke line
    stream.push_str(&format!("{} RG 2 w\n", rgb(a1)));
    for pair in pts.windows(2) {
        stream.push_str(&format!(
            "{:.1} {:.1} m {:.1} {:.1} l S\n",
            pair[0].0, pair[0].1, pair[1].0, pair[1].1
        ));
    }
}
