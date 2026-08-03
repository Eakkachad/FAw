#![allow(clippy::too_many_arguments)]
//! Chart glyph rasterizer for PNG output (`katSVG Chart Raster`).
//!
//! Renders chart glyphs into an RGB pixel buffer, mirroring `chart.rs` geometry
//! (same normalization math) so PNG/PDF/PPTX share the same chart layout. D1.

use crate::router::{ChartSpec, ChartType};

/// Draw a chart glyph into an RGB buffer (`w` width, 3 bytes/pixel) at region
/// `(x, y)`..`(x+w, y+h)`.
pub fn draw_chart_raster(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    text: (u8, u8, u8),
) {
    match spec.chart_type {
        ChartType::Bar => bar(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Line => line(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Pie | ChartType::Donut => pie(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Scatter => scatter(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Heatmap => heatmap(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Gauge => gauge(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::StackedBar => stacked(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
        ChartType::Area => area(spec, buf, w, h, x, y, cw, ch, a1, a2, text),
    }
}

fn max_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0, f64::max).max(1.0)
}

fn set(buf: &mut [u8], w: usize, h: usize, px: usize, py: usize, c: (u8, u8, u8)) {
    if px < w && py < h {
        let i = (py * w + px) * 3;
        buf[i] = c.0;
        buf[i + 1] = c.1;
        buf[i + 2] = c.2;
    }
}

fn vline(buf: &mut [u8], w: usize, h: usize, x: usize, y0: usize, y1: usize, c: (u8, u8, u8)) {
    for y in y0..y1 {
        set(buf, w, h, x, y, c);
    }
}

fn rect(
    buf: &mut [u8],
    w: usize,
    h: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    c: (u8, u8, u8),
) {
    for y in y0..y1 {
        for x in x0..x1 {
            set(buf, w, h, x, y, c);
        }
    }
}

fn bar(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let n = spec.values.len().max(1);
    let max = max_value(&spec.values);
    let plot_w = cw.saturating_sub(16);
    let plot_h = ch.saturating_sub(32);
    let bar_w = (plot_w / n).clamp(2, 48);
    let gap = ((plot_w.saturating_sub(bar_w * n)) / (n + 1)).max(2);
    vline(buf, w, h, x + 8, y + plot_h, y + plot_h + 2, (55, 65, 81));
    for (i, v) in spec.values.iter().enumerate() {
        let bh = ((v / max) * plot_h as f64).max(2.0) as usize;
        let bx = x + 8 + i * (bar_w + gap) + gap;
        let by = y + plot_h - bh;
        rect(
            buf,
            w,
            h,
            bx,
            by,
            bx + bar_w,
            y + plot_h,
            if i % 2 == 0 { a1 } else { a2 },
        );
    }
}

fn line(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let n = spec.values.len().max(2);
    let max = max_value(&spec.values);
    let plot_w = cw.saturating_sub(16);
    let plot_h = ch.saturating_sub(32);
    let pts: Vec<(usize, usize)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x + 8 + i * (plot_w - 16) / (n - 1);
            let py = y + plot_h - ((v / max) * plot_h as f64) as usize;
            (px, py)
        })
        .collect();
    for pair in pts.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        let steps = x1.abs_diff(x0).max(y1.abs_diff(y0)).max(1);
        for s in 0..=steps {
            let t = s as isize;
            let st = steps as isize;
            let px = (x0 as isize + (x1 as isize - x0 as isize) * t / st) as usize;
            let py = (y0 as isize + (y1 as isize - y0 as isize) * t / st) as usize;
            set(buf, w, h, px, py, a1);
        }
    }
    for (px, py) in &pts {
        rect(
            buf,
            w,
            h,
            px.saturating_sub(2),
            py.saturating_sub(2),
            px + 2,
            py + 2,
            a2,
        );
    }
}

fn pie(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let total: f64 = spec.values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let is_donut = spec.chart_type == ChartType::Donut;
    let cx = x + cw / 2;
    let cy = y + ch / 2;
    let r = (cw.min(ch) / 2).saturating_sub(8) as f64;
    let mut angle = -std::f64::consts::FRAC_PI_2;
    // Approximate arcs by sampling points.
    for (i, v) in spec.values.iter().enumerate() {
        let frac = v / total;
        let sweep = frac * 2.0 * std::f64::consts::PI;
        let end = angle + sweep;
        let color = if i % 2 == 0 { a1 } else { a2 };
        let samples = ((sweep / (std::f64::consts::PI / 48.0)).ceil() as usize).max(4);
        let mut prev: Option<(isize, isize)> = None;
        for s in 0..=samples {
            let a = angle + sweep * s as f64 / samples as f64;
            let px = cx as f64 + r * a.cos();
            let py = cy as f64 + r * a.sin();
            if let Some((lx, ly)) = prev {
                draw_triangle(
                    buf,
                    w,
                    h,
                    cx as isize,
                    cy as isize,
                    lx,
                    ly,
                    px as isize,
                    py as isize,
                    color,
                );
            }
            prev = Some((px as isize, py as isize));
        }
        angle = end;
    }
    if is_donut {
        let hole = (r * 0.55) as usize;
        rect(
            buf,
            w,
            h,
            cx.saturating_sub(hole),
            cy.saturating_sub(hole),
            cx + hole,
            cy + hole,
            (11, 15, 25),
        );
    }
}

fn draw_triangle(
    buf: &mut [u8],
    w: usize,
    h: usize,
    ax: isize,
    ay: isize,
    bx: isize,
    by: isize,
    cx: isize,
    cy: isize,
    color: (u8, u8, u8),
) {
    let minx = ax.min(bx).min(cx).max(0) as usize;
    let maxx = ax.max(bx).max(cx) as usize;
    let miny = ay.min(by).min(cy).max(0) as usize;
    let maxy = ay.max(by).max(cy) as usize;
    for py in miny..=maxy {
        for px in minx..=maxx {
            if point_in_tri(px as isize, py as isize, ax, ay, bx, by, cx, cy) {
                set(buf, w, h, px, py, color);
            }
        }
    }
}

fn point_in_tri(
    px: isize,
    py: isize,
    ax: isize,
    ay: isize,
    bx: isize,
    by: isize,
    cx: isize,
    cy: isize,
) -> bool {
    let d1 = sign(px, py, ax, ay, bx, by);
    let d2 = sign(px, py, bx, by, cx, cy);
    let d3 = sign(px, py, cx, cy, ax, ay);
    let neg = d1 < 0 || d2 < 0 || d3 < 0;
    let pos = d1 > 0 || d2 > 0 || d3 > 0;
    !(neg && pos)
}

fn sign(px: isize, py: isize, ax: isize, ay: isize, bx: isize, by: isize) -> isize {
    (px - bx) * (ay - by) - (ax - bx) * (py - by)
}

fn scatter(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let n = spec.values.len().max(2);
    let max = max_value(&spec.values);
    let plot_w = cw.saturating_sub(16);
    let plot_h = ch.saturating_sub(16);
    rect(buf, w, h, x + 8, y, x + plot_w, y + plot_h, (55, 65, 81));
    for (i, v) in spec.values.iter().enumerate() {
        let px = x + 8 + i * (plot_w - 8) / (n - 1);
        let py = y + plot_h - ((v / max) * plot_h as f64) as usize;
        let color = if i % 2 == 0 { a1 } else { a2 };
        rect(
            buf,
            w,
            h,
            px.saturating_sub(2),
            py.saturating_sub(2),
            px + 3,
            py + 3,
            color,
        );
    }
}

fn heatmap(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let max = max_value(&spec.values);
    let n = spec.values.len().max(1);
    let cell_w = (cw / n).max(8);
    let cell_h = ch / 3;
    for (i, v) in spec.values.iter().enumerate() {
        let intensity = (v / max).clamp(0.0, 1.0) as u8;
        let color = if i % 2 == 0 { a1 } else { a2 };
        let cx = x + (i % 5) * cell_w;
        let cy = y + (i / 5) * cell_h;
        let c = (
            (color.0 as u16 * intensity as u16 / 255).min(255) as u8,
            (color.1 as u16 * intensity as u16 / 255).min(255) as u8,
            (color.2 as u16 * intensity as u16 / 255).min(255) as u8,
        );
        rect(buf, w, h, cx, cy, cx + cell_w - 4, cy + cell_h - 4, c);
    }
}

fn gauge(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let max = max_value(&spec.values);
    let cx = x + cw / 2;
    let cy = y + ch;
    let r = (cw / 2).saturating_sub(8) as f64;
    for (i, v) in spec.values.iter().enumerate() {
        let frac = (v / max).clamp(0.0, 1.0);
        let sweep = frac * std::f64::consts::PI;
        let color = if i % 2 == 0 { a1 } else { a2 };
        let steps = (sweep / 0.05).ceil() as usize;
        for s in 0..=steps {
            let a = sweep * s as f64 / steps as f64;
            let px = cx as f64 - r + r * a.cos();
            let py = cy as f64 - r * a.sin();
            set(buf, w, h, px as usize, py as usize, color);
        }
    }
}

fn stacked(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let n = spec.values.len().max(1);
    let max = max_value(&spec.values);
    let plot_w = cw.saturating_sub(16);
    let plot_h = ch.saturating_sub(32);
    let bar_w = (plot_w / n).clamp(2, 48);
    let gap = ((plot_w.saturating_sub(bar_w * n)) / (n + 1)).max(2);
    for (i, v) in spec.values.iter().enumerate() {
        let frac = v / max;
        let bx = x + 8 + i * (bar_w + gap) + gap;
        let h1 = (frac * plot_h as f64 * 0.6).max(2.0) as usize;
        let h2 = (frac * plot_h as f64 * 0.4).max(2.0) as usize;
        let y1 = y + plot_h - h1;
        let y2 = y1 - h2;
        rect(buf, w, h, bx, y1, bx + bar_w, y + plot_h, a1);
        rect(buf, w, h, bx, y2, bx + bar_w, y1, a2);
    }
}

fn area(
    spec: &ChartSpec,
    buf: &mut [u8],
    w: usize,
    h: usize,
    x: usize,
    y: usize,
    cw: usize,
    ch: usize,
    a1: (u8, u8, u8),
    a2: (u8, u8, u8),
    _t: (u8, u8, u8),
) {
    let n = spec.values.len().max(2);
    let max = max_value(&spec.values);
    let plot_w = cw.saturating_sub(16);
    let plot_h = ch.saturating_sub(16);
    let pts: Vec<(usize, usize)> = spec
        .values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let px = x + 8 + i * (plot_w - 16) / (n - 1);
            let py = y + plot_h - ((v / max) * plot_h as f64) as usize;
            (px, py)
        })
        .collect();
    // fill area under polyline (scanline per x)
    for xc in x + 8..x + plot_w {
        let mut top = y + plot_h;
        for pair in pts.windows(2) {
            let (x0, y0) = pair[0];
            let (x1, y1) = pair[1];
            let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
            if xc >= lo && xc <= hi {
                let span = hi.saturating_sub(lo).max(1);
                let yy = if hi == lo {
                    y0
                } else {
                    (y0 as isize
                        + (y1 as isize - y0 as isize) * (xc as isize - lo as isize) / span as isize)
                        as usize
                };
                if yy < top {
                    top = yy;
                }
            }
        }
        rect(buf, w, h, xc, top, xc + 1, y + plot_h, a2);
    }
    for pair in pts.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        let steps = x1.abs_diff(x0).max(y1.abs_diff(y0)).max(1);
        for s in 0..=steps {
            let t = s as isize;
            let st = steps as isize;
            let px = (x0 as isize + (x1 as isize - x0 as isize) * t / st) as usize;
            let py = (y0 as isize + (y1 as isize - y0 as isize) * t / st) as usize;
            set(buf, w, h, px, py, a1);
        }
    }
}
