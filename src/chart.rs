//! Native SVG Chart Glyph Engine (`katSVG Charts`).
//!
//! Renders deterministic, dependency-free chart glyphs directly as SVG vector
//! fragments from a `ChartSpec`. Every glyph uses theme roles only; values are
//! plotted 1:1 from the spec (no aggregation, no invented data).

use crate::router::{ChartSpec, ChartType};

/// Theme roles injected into chart glyphs.
#[derive(Debug, Clone, Copy)]
pub struct ChartColors<'a> {
    pub bg: &'a str,
    pub card_bg: &'a str,
    pub accent1: &'a str,
    pub accent2: &'a str,
    pub text: &'a str,
}

/// Native SVG Chart Glyph Renderer
pub struct ChartGlyphRenderer;

impl ChartGlyphRenderer {
    /// Renders the chart glyph for `spec` into an SVG fragment positioned at
    /// `(x, y)` with size `(w, h)`, using the injected theme roles.
    pub fn render(spec: &ChartSpec, colors: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
        match spec.chart_type {
            ChartType::Bar => render_bar(spec, colors, x, y, w, h),
            ChartType::Line => render_line(spec, colors, x, y, w, h),
            ChartType::Pie => render_pie(spec, colors, x, y, w, h),
            ChartType::Scatter => render_scatter(spec, colors, x, y, w, h),
            ChartType::Heatmap => render_heatmap(spec, colors, x, y, w, h),
            ChartType::Gauge => render_gauge(spec, colors, x, y, w, h),
            ChartType::Donut => render_donut(spec, colors, x, y, w, h),
            ChartType::StackedBar => render_stacked_bar(spec, colors, x, y, w, h),
            ChartType::Area => render_area(spec, colors, x, y, w, h),
        }
    }
}

fn max_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max).max(1.0)
}

fn render_bar(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let series = spec.all_series();
    let n = spec.labels.len().max(1) as u32;
    let s_count = series.len() as u32;
    let max = series.iter().flatten().copied().fold(0.0, f64::max).max(1.0);
    let plot_w = w - 16;
    let plot_h = h - 32;
    let group_w = (plot_w / n).min(48);
    let bar_w = (group_w / s_count).max(3);
    let gap = ((plot_w - group_w * n) / (n + 1)).max(2);

    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n  <line x1=\"8\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#374151\" stroke-width=\"1\" />\n",
        x, y, plot_h, plot_w, plot_h
    );
    for (i, label) in spec.labels.iter().enumerate() {
        let gx = 8 + i as u32 * (group_w + gap) + gap;
        for (s, vals) in series.iter().enumerate() {
            let v = vals.get(i).copied().unwrap_or(0.0);
            let bh = ((v / max) * plot_h as f64).max(2.0) as u32;
            let bx = gx + s as u32 * bar_w;
            let by = plot_h - bh;
            let color = if s % 2 == 0 { c.accent1 } else { c.accent2 };
            svg.push_str(&format!(
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\" />\n",
                bx, by, bar_w, bh, color
            ));
        }
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"{}\" text-anchor=\"middle\">{}</text>\n",
            gx + group_w / 2,
            plot_h + 16,
            c.text,
            label
        ));
    }
    svg.push_str("</g>\n");
    svg
}

fn render_line(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let series = spec.all_series();
    let n = spec.labels.len().max(2) as u32;
    let max = series.iter().flatten().copied().fold(0.0, f64::max).max(1.0);
    let plot_w = w - 16;
    let plot_h = h - 32;

    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n",
        x, y
    );
    for (s, vals) in series.iter().enumerate() {
        let color = if s % 2 == 0 { c.accent1 } else { c.accent2 };
        let mut points = String::new();
        for (i, v) in vals.iter().enumerate() {
            let px = 8 + i as u32 * (plot_w - 16) / (n - 1);
            let py = plot_h - ((v / max) * plot_h as f64) as u32;
            points.push_str(&format!("{},{} ", px, py));
        }
        svg.push_str(&format!(
            "  <polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" />\n",
            points.trim(), color
        ));
        for (i, v) in vals.iter().enumerate() {
            let px = 8 + i as u32 * (plot_w - 16) / (n - 1);
            let py = plot_h - ((v / max) * plot_h as f64) as u32;
            svg.push_str(&format!(
                "  <circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{}\" />\n",
                px, py, color
            ));
            let _ = v;
        }
    }
    for (i, label) in spec.labels.iter().enumerate() {
        let px = 8 + i as u32 * (plot_w - 16) / (n - 1);
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#9CA3AF\" text-anchor=\"middle\">{}</text>\n",
            px, plot_h + 16, label
        ));
    }
    svg.push_str("</g>\n");
    svg
}

fn render_pie(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let total: f64 = spec.values.iter().sum();
    if total <= 0.0 {
        return String::new();
    }
    let cx = w / 2;
    let cy = h / 2;
    let r = (w.min(h) / 2 - 8) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;

    let mut svg = format!("<g transform=\"translate({}, {})\">\n", x, y);
    for (i, v) in spec.values.iter().enumerate() {
        let frac = v / total;
        let sweep = frac * 2.0 * std::f64::consts::PI;
        let end = angle + sweep;
        let x0 = cx as f64 + r * angle.cos();
        let y0 = cy as f64 + r * angle.sin();
        let x1 = cx as f64 + r * end.cos();
        let y1 = cy as f64 + r * end.sin();
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
        let color = if i % 2 == 0 { c.accent1 } else { c.accent2 };
        svg.push_str(&format!(
            "  <path d=\"M {} {} L {} {} A {} {} 0 {} 1 {} {} Z\" fill=\"{}\" />\n",
            cx, cy, x0, y0, r, r, large, x1, y1, color
        ));
        angle = end;
    }
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />\n",
        cx, cy, (r * 0.5) as u32, c.bg
    ));
    svg.push_str("</g>\n");
    svg
}

fn render_scatter(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let n = spec.values.len() as u32;
    let max = max_value(&spec.values);
    let plot_w = w - 16;
    let plot_h = h - 16;

    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n  <rect x=\"8\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#374151\" stroke-width=\"1\" />\n",
        x, y, plot_w - 8, plot_h
    );
    for (i, v) in spec.values.iter().enumerate() {
        let px = 8 + if n == 1 { 0 } else { i as u32 * (plot_w - 8) / (n - 1) };
        let py = plot_h - ((v / max) * plot_h as f64) as u32;
        let color = if i % 2 == 0 { c.accent1 } else { c.accent2 };
        svg.push_str(&format!(
            "  <circle cx=\"{}\" cy=\"{}\" r=\"5\" fill=\"{}\" opacity=\"0.85\" />\n",
            px, py, color
        ));
    }
    svg.push_str("</g>\n");
    svg
}

fn render_heatmap(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let max = max_value(&spec.values);
    let n = spec.values.len() as u32;
    let cell_w = (w / n).max(8);
    let cell_h = h / 3;
    let mut svg = format!("<g transform=\"translate({}, {})\">\n", x, y);
    for (i, v) in spec.values.iter().enumerate() {
        let intensity = (v / max).clamp(0.0, 1.0);
        let color = if i % 2 == 0 { c.accent1 } else { c.accent2 };
        let cx = (i as u32 % 5) * cell_w;
        let cy = (i as u32 / 5) * cell_h;
        svg.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"{}\" opacity=\"{:.2}\" />\n",
            cx, cy, cell_w - 4, cell_h - 4, color, intensity
        ));
    }
    svg.push_str("</g>\n");
    svg
}

fn render_gauge(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let max = max_value(&spec.values);
    let cx = w / 2;
    let cy = h;
    let r = (w / 2 - 8) as f64;
    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n  <path d=\"M {} {} A {} {} 0 1 1 {} {}\" fill=\"none\" stroke=\"#374151\" stroke-width=\"12\" stroke-linecap=\"round\" />\n",
        x, y, cx - (r as u32), cy, r, r, cx + (r as u32), cy
    );
    for (i, v) in spec.values.iter().enumerate() {
        let frac = (v / max).clamp(0.0, 1.0);
        let sweep = frac * std::f64::consts::PI;
        let color = if i % 2 == 0 { c.accent1 } else { c.accent2 };
        let x0 = cx as f64 - r + (r * sweep.cos());
        let y0 = cy as f64 - (r * sweep.sin());
    svg.push_str(&format!(
        "  <path d=\"M {} {} A {} {} 0 0 1 {} {}\" fill=\"none\" stroke=\"{}\" stroke-width=\"12\" stroke-linecap=\"round\" />\n",
        cx as f64 - r, cy as f64, r, r, x0, y0, color
    ));
    }
    svg.push_str("</g>\n");
    svg
}

/// Donut chart: pie slices with a wide center hole (accent1/accent2 alternating).
fn render_donut(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let total: f64 = spec.values.iter().sum();
    if total <= 0.0 {
        return String::new();
    }
    let cx = w / 2;
    let cy = h / 2;
    let r = (w.min(h) / 2 - 8) as f64;
    let hole = (r * 0.55) as u32;
    let mut angle = -std::f64::consts::FRAC_PI_2;

    let mut svg = format!("<g transform=\"translate({}, {})\">\n", x, y);
    for (i, v) in spec.values.iter().enumerate() {
        let sweep = (v / total) * 2.0 * std::f64::consts::PI;
        let end = angle + sweep;
        let x0 = cx as f64 + r * angle.cos();
        let y0 = cy as f64 + r * angle.sin();
        let x1 = cx as f64 + r * end.cos();
        let y1 = cy as f64 + r * end.sin();
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };
        let color = if i % 2 == 0 { c.accent1 } else { c.accent2 };
        svg.push_str(&format!(
            "  <path d=\"M {} {} L {} {} A {} {} 0 {} 1 {} {} Z\" fill=\"{}\" />\n",
            cx, cy, x0, y0, r, r, large, x1, y1, color
        ));
        angle = end;
    }
    svg.push_str(&format!(
        "  <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" />\n",
        cx, cy, hole, c.bg
    ));
    svg.push_str("</g>\n");
    svg
}

/// Stacked bar chart: each bar is a stack of two segments (accent1 bottom, accent2 top).
fn render_stacked_bar(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let n = spec.values.len() as u32;
    let max = max_value(&spec.values);
    let plot_w = w - 16;
    let plot_h = h - 32;
    let bar_w = (plot_w / n).min(48);
    let gap = ((plot_w - bar_w * n) / (n + 1)).max(2);

    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n  <line x1=\"8\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#374151\" stroke-width=\"1\" />\n",
        x, y, plot_h, plot_w, plot_h
    );
    for (i, v) in spec.values.iter().enumerate() {
        let frac = v / max;
        let bx = 8 + i as u32 * (bar_w + gap) + gap;
        let h1 = (frac * plot_h as f64 * 0.6).max(2.0) as u32;
        let h2 = (frac * plot_h as f64 * 0.4).max(2.0) as u32;
        let y1 = plot_h - h1;
        let y2 = y1 - h2;
        svg.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{}\" />\n",
            bx, y1, bar_w, h1, c.accent1
        ));
        svg.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{}\" />\n",
            bx, y2, bar_w, h2, c.accent2
        ));
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"{}\" text-anchor=\"middle\">{}</text>\n",
            bx + bar_w / 2,
            plot_h + 16,
            c.text,
            spec.labels.get(i).map(|s| s.as_str()).unwrap_or("")
        ));
    }
    svg.push_str("</g>\n");
    svg
}

/// Area chart: line with filled area beneath (accent1 stroke, accent2 translucent fill).
fn render_area(spec: &ChartSpec, c: &ChartColors<'_>, x: u32, y: u32, w: u32, h: u32) -> String {
    let n = spec.values.len() as u32;
    let max = max_value(&spec.values);
    let plot_w = w - 16;
    let plot_h = h - 16;

    let mut line = String::new();
    let mut area = String::new();
    for (i, v) in spec.values.iter().enumerate() {
        let px = 8 + if n == 1 { 0 } else { i as u32 * (plot_w - 16) / (n - 1) };
        let py = plot_h - ((v / max) * plot_h as f64) as u32;
        line.push_str(&format!("{},{} ", px, py));
        area.push_str(&format!("{},{} ", px, py));
    }

    let mut svg = format!(
        "<g transform=\"translate({}, {})\">\n  <polyline points=\"8,{} {}\" fill=\"none\" stroke=\"#374151\" stroke-width=\"1\" />\n",
        x, y, plot_h, if n > 0 { line.split(' ').last().unwrap_or("") } else { "" }
    );
    svg.push_str(&format!(
        "  <polygon points=\"8,{} {} 8,{}\" fill=\"{}\" fill-opacity=\"0.25\" />\n",
        plot_h, area.trim(), plot_h, c.accent2
    ));
    svg.push_str(&format!(
        "  <polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\" />\n",
        line.trim(), c.accent1
    ));
    for (i, v) in spec.values.iter().enumerate() {
        let px = 8 + if n == 1 { 0 } else { i as u32 * (plot_w - 16) / (n - 1) };
        let py = plot_h - ((v / max) * plot_h as f64) as u32;
        svg.push_str(&format!(
            "  <circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{}\" />\n",
            px, py, c.accent2
        ));
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#9CA3AF\" text-anchor=\"middle\">{}</text>\n",
            px,
            plot_h + 14,
            spec.labels.get(i).map(|s| s.as_str()).unwrap_or("")
        ));
        let _ = v;
    }
    svg.push_str("</g>\n");
    svg
}
