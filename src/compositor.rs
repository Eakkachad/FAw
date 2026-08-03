//! Region-based layout composition (`katSVG Compositor`).
//!
//! F1: drives composition from `LayoutDef.regions` (unit coordinates) + slot
//! semantics instead of the legacy fixed compositor, so each corpus archetype
//! renders with its own geometry. Falls back to the legacy compositor when a
//! layout id is unknown.

use crate::router::{InfographicLayoutSpec, LayoutDef, PaletteTheme, RegionDef};

/// A resolved pixel rectangle for one region.
#[derive(Debug, Clone, Copy)]
pub struct RegionRect {
    pub slot: Slot,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Slot kinds a region can bind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Title,
    Subtitle,
    Metrics,
    Sections,
    Chart,
    Footer,
}

/// Convert a unit region to pixels given canvas size.
pub fn unit_to_px(r: &RegionDef, width: u32, height: u32) -> (u32, u32, u32, u32) {
    let x = (r.x * width as f64) as u32;
    let y = (r.y * height as f64) as u32;
    let w = (r.width * width as f64) as u32;
    let h = (r.height * height as f64) as u32;
    (x, y, w, h)
}

fn slot_of(s: &str) -> Slot {
    match s {
        "title" => Slot::Title,
        "subtitle" => Slot::Subtitle,
        "metrics" => Slot::Metrics,
        "sections" => Slot::Sections,
        "chart" => Slot::Chart,
        "footer" => Slot::Footer,
        _ => Slot::Sections,
    }
}

/// Resolve the pixel rects for all regions of a layout.
pub fn regions_px(layout: &LayoutDef, width: u32, height: u32) -> Vec<RegionRect> {
    layout
        .regions
        .iter()
        .map(|r| {
            let (x, y, w, h) = unit_to_px(r, width, height);
            RegionRect {
                slot: slot_of(&r.slot),
                x,
                y,
                w,
                h,
            }
        })
        .collect()
}

fn rect_for(regions: &[RegionRect], slot: Slot) -> Option<&RegionRect> {
    regions.iter().find(|r| r.slot == slot)
}

/// Theme color roles (bg, cardBg, accent1, accent2, text).
fn colors(
    theme: PaletteTheme,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    theme.colors()
}

fn escape_svg(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Truncate a string to a pixel budget using the real font rasterizer width
/// measurement (F6) so text never spills past its region. Deterministic.
fn fit_text(text: &str, px: f32, budget_px: u32) -> String {
    static RENDERER: std::sync::OnceLock<crate::text::TextRenderer> = std::sync::OnceLock::new();
    RENDERER
        .get_or_init(crate::text::TextRenderer::new)
        .truncate_to_fit(px, budget_px as f32, text)
}

/// Render an SVG from the region layout. Returns the SVG string.
pub fn render_svg_regions(layout: &LayoutDef, spec: &InfographicLayoutSpec) -> Option<String> {
    let (width, height) = spec.aspect_ratio.dimensions();
    let regions = regions_px(layout, width, height);
    if regions.is_empty() {
        return None;
    }
    let (bg, card_bg, accent1, accent2, text) = colors(spec.theme);

    let mut svg = String::with_capacity(8192);
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">\n",
        width, height, width, height
    ));
    svg.push_str("<defs>\n  <style>\n");
    let has_thai = crate::font::has_non_ascii(&spec.title)
        || spec
            .subtitle
            .as_deref()
            .is_some_and(crate::font::has_non_ascii)
        || spec
            .metrics
            .iter()
            .any(|m| crate::font::has_non_ascii(&m.label) || crate::font::has_non_ascii(&m.value))
        || spec
            .sections
            .iter()
            .any(|s| crate::font::has_non_ascii(&s.title));
    svg.push_str(&crate::font::font_style_block(has_thai));
    svg.push_str(&format!(
        "    text {{ font-family: {}; }}\n",
        crate::font::font_stack(has_thai)
    ));
    svg.push_str(&format!(
        "    .title {{ font-size: 26px; font-weight: 800; fill: {}; }}\n",
        text
    ));
    svg.push_str("    .subtitle { font-size: 14px; font-weight: 400; fill: #9CA3AF; }\n");
    svg.push_str(&format!(
        "    .card-title {{ font-size: 15px; font-weight: 600; fill: {}; }}\n",
        text
    ));
    svg.push_str("    .card-desc { font-size: 11px; font-weight: 400; fill: #9CA3AF; }\n");
    svg.push_str(&format!(
        "    .metric-val {{ font-size: 22px; font-weight: 800; fill: {}; }}\n",
        accent1
    ));
    svg.push_str("    .metric-lbl { font-size: 10px; font-weight: 600; fill: #9CA3AF; letter-spacing: 0.5px; }\n");
    svg.push_str(&format!(
        "    .badge {{ font-size: 11px; font-weight: 800; fill: {}; }}\n",
        bg
    ));
    svg.push_str("  </style>\n");
    svg.push_str(&format!(
        "  <linearGradient id=\"bg-grad\" x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"100%\">\n    <stop offset=\"0%\" stop-color=\"{}\" />\n    <stop offset=\"100%\" stop-color=\"{}\" />\n  </linearGradient>\n</defs>\n",
        bg, card_bg
    ));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"url(#bg-grad)\" />\n");

    // Title + subtitle
    if let Some(r) = rect_for(&regions, Slot::Title) {
        let t = fit_text(&spec.title, 26.0, r.w.saturating_sub(16));
        svg.push_str(&format!(
            "<g transform=\"translate({}, {})\">\n  <rect x=\"0\" y=\"0\" width=\"8\" height=\"40\" rx=\"4\" fill=\"{}\" />\n  <text x=\"20\" y=\"28\" class=\"title\">{}</text>\n</g>\n",
            r.x, r.y, accent1, escape_svg(&t)
        ));
    }
    if let Some(r) = rect_for(&regions, Slot::Subtitle)
        && let Some(sub) = &spec.subtitle
    {
        let s = fit_text(sub, 14.0, r.w.saturating_sub(8));
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" class=\"subtitle\">{}</text>\n",
            r.x,
            r.y + 18,
            escape_svg(&s)
        ));
    }

    // Metrics band: cards laid out horizontally within the region.
    if let Some(r) = rect_for(&regions, Slot::Metrics)
        && !spec.metrics.is_empty()
    {
        let n = spec.metrics.len() as u32;
        let gap = 10u32;
        let card_w = (r.w.saturating_sub((n.saturating_sub(1)) * gap)) / n;
        for (i, m) in spec.metrics.iter().enumerate() {
            let x = r.x + i as u32 * (card_w + gap);
            let icon_svg = crate::icon::IconRenderer::render(&m.icon, accent2);
            svg.push_str(&format!(
                    "<g transform=\"translate({}, {})\">\n  <rect width=\"{}\" height=\"{}\" rx=\"10\" fill=\"{}\" stroke=\"#1F2937\" stroke-width=\"1\" />\n  <text x=\"12\" y=\"26\" class=\"metric-val\">{}</text>\n  <text x=\"12\" y=\"44\" class=\"metric-lbl\">{}</text>\n  <g transform=\"translate({}, 8)\">{}</g>\n</g>\n",
                    x, r.y, card_w, r.h, card_bg, escape_svg(&m.value), escape_svg(&m.label.to_uppercase()), card_w.saturating_sub(26), icon_svg
                ));
        }
    }

    // Chart region.
    if let Some(r) = rect_for(&regions, Slot::Chart)
        && let Some(chart) = &spec.chart
    {
        let c = crate::chart::ChartColors {
            bg,
            card_bg,
            accent1,
            accent2,
            text,
        };
        svg.push_str(&crate::chart::ChartGlyphRenderer::render(
            chart, &c, r.x, r.y, r.w, r.h,
        ));
    }

    // Sections: stacked cards within the region.
    if let Some(r) = rect_for(&regions, Slot::Sections) {
        let n = spec.sections.len().max(1) as u32;
        let gap = 8u32;
        let sec_h = r.h.saturating_sub((n.saturating_sub(1)) * gap) / n;
        for (i, s) in spec.sections.iter().enumerate() {
            let y = r.y + i as u32 * (sec_h + gap);
            svg.push_str(&format!(
                "<g transform=\"translate({}, {})\">\n  <rect width=\"{}\" height=\"{}\" rx=\"10\" fill=\"{}\" stroke=\"#1F2937\" stroke-width=\"1\" />\n  <circle cx=\"22\" cy=\"{}\" r=\"12\" fill=\"{}\" />\n  <text x=\"22\" y=\"{}\" text-anchor=\"middle\" class=\"badge\">{}</text>\n  <text x=\"44\" y=\"22\" class=\"card-title\">{}</text>\n  <text x=\"44\" y=\"40\" class=\"card-desc\">{}</text>\n</g>\n",
                r.x, y, r.w, sec_h, card_bg, sec_h / 2, accent1, sec_h / 2 + 4, s.step_number, fit_text(&s.title, 15.0, r.w.saturating_sub(56)), fit_text(&s.description, 11.0, r.w.saturating_sub(56))
            ));
        }
    }

    // Footer.
    if let Some(r) = rect_for(&regions, Slot::Footer)
        && let Some(footer) = &spec.footer_note
    {
        let cx = r.x + r.w / 2;
        svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#6B7280\">{}</text>\n",
                cx, r.y + r.h / 2, escape_svg(&fit_text(footer, 11.0, r.w))
            ));
    }

    svg.push_str("</svg>");
    Some(svg)
}
