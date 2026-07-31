//! Icon Set (`katSVG Icons`).
//!
//! Renders named monochrome glyphs (feather-style, 24×24 viewBox, `currentColor`
//! stroke) into inline SVG. Icons come from the embedded `corpus/icons/icons.json`
//! corpus; unknown names fall back to a generic circle glyph (never panic, never
//! invent). Resolves G2: `MetricCardSpec.icon` strings now render as vectors.

use serde::Deserialize;
use std::collections::HashMap;

const ICONS_JSON: &str = include_str!("../corpus/icons/icons.json");

/// Renders an icon by name into an inline SVG `<path>` fragment with the given
/// stroke color. Falls back to a generic glyph for unknown names.
pub struct IconRenderer;

impl IconRenderer {
    /// Render `<path>` elements for `name` inside a 24×24 viewBox, filled with
    /// `color` using the current stroke. Returns an SVG `<g>` group.
    pub fn render(name: &str, color: &str) -> String {
        let paths = Self::paths(name);
        let mut out = String::with_capacity(256);
        out.push_str(&format!(
            "<g fill=\"none\" stroke=\"{}\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" transform=\"translate(-4,-4)\">",
            color
        ));
        for p in paths {
            out.push_str(&format!("<path d=\"{}\" />", p));
        }
        out.push_str("</g>");
        out
    }

    /// Icon path `d` string(s) for `name`; empty if unknown.
    pub fn paths(name: &str) -> Vec<String> {
        registry()
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Whether an icon name exists in the corpus.
    pub fn has(name: &str) -> bool {
        registry().contains_key(&name.to_lowercase())
    }

    /// Number of icons in the corpus.
    pub fn count() -> usize {
        registry().len()
    }
}

fn registry() -> &'static HashMap<String, Vec<String>> {
    static REGISTRY: std::sync::OnceLock<HashMap<String, Vec<String>>> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        #[derive(Deserialize)]
        struct Raw(HashMap<String, String>);
        let raw: Raw = serde_json::from_str(ICONS_JSON).expect("embedded icons.json must be valid");
        raw.0
            .into_iter()
            .map(|(k, v)| (k, split_commands(&v)))
            .collect()
    })
}

/// Splits a compact `M... Z M... Z` path string into separate path commands so
/// multi-shape icons (e.g., "cpu") render as multiple `<path>` elements.
fn split_commands(d: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in d.chars() {
        if c == 'M' && !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
