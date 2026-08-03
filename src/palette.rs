#![allow(clippy::too_many_arguments)]
//! Palette Library Registry (`katSVG Palettes`).
//!
//! Loads palette entries from the embedded `corpus/palettes/` JSON corpus into a
//! deterministic registry, exposing named color roles (bg, cardBg, accent1,
//! accent2, text) plus contrast metadata. `PaletteTheme` remains the typed enum
//! in the spec; this module bridges it to corpus data with a hardcoded fallback
//! for safety (G3 resolution).

use crate::router::PaletteTheme;
use serde::Deserialize;

/// Color roles of a palette entry (5 named slots).
#[derive(Debug, Clone, Copy)]
pub struct PaletteColors {
    pub bg: &'static str,
    pub card_bg: &'static str,
    pub accent1: &'static str,
    pub accent2: &'static str,
    pub text: &'static str,
}

/// A contrast pair with a pre-computed WCAG ratio.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContrastPair {
    pub foreground: String,
    pub background: String,
    pub ratio: f64,
}

/// One corpus palette entry (serde mirror of `schemas/palette_library.schema.json`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteEntry {
    pub id: String,
    pub palette_type: PaletteTheme,
    pub description: Option<String>,
    pub roles: PaletteRoles,
    pub contrast_pairs: Vec<ContrastPair>,
    pub color_blind_safe: Option<bool>,
    pub dark_mode: Option<bool>,
}

/// Named color roles (hex strings from corpus).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteRoles {
    pub bg: String,
    pub card_bg: String,
    pub accent1: String,
    pub accent2: String,
    pub text: String,
}

/// Embedded palette corpus (8 entries). Loaded at registry construction;
/// deterministic and offline.
pub const PALETTE_FILES: [&str; 8] = [
    include_str!("../corpus/palettes/tech_dark.json"),
    include_str!("../corpus/palettes/financial_navy.json"),
    include_str!("../corpus/palettes/vibrant_coral.json"),
    include_str!("../corpus/palettes/academic_warm.json"),
    include_str!("../corpus/palettes/ocean_breeze.json"),
    include_str!("../corpus/palettes/sunset_glow.json"),
    include_str!("../corpus/palettes/forest_mint.json"),
    include_str!("../corpus/palettes/monochrome.json"),
];

/// Deterministic registry over the embedded palette corpus.
pub struct PaletteRegistry {
    entries: Vec<PaletteEntry>,
}

impl PaletteRegistry {
    pub fn new() -> Self {
        Self {
            entries: PALETTE_FILES
                .iter()
                .filter_map(|raw| serde_json::from_str(raw).ok())
                .collect(),
        }
    }

    /// Number of loaded palette entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up a palette entry by theme.
    pub fn get(&self, theme: PaletteTheme) -> Option<&PaletteEntry> {
        self.entries.iter().find(|e| e.palette_type == theme)
    }

    /// All entries in corpus order.
    pub fn entries(&self) -> &[PaletteEntry] {
        &self.entries
    }

    /// Resolve the 5 color roles for a theme: corpus value if present, else the
    /// hardcoded fallback (guarantees rendering even if a corpus entry is missing).
    pub fn colors(&self, theme: PaletteTheme) -> PaletteColors {
        if let Some(e) = self.get(theme) {
            return PaletteColors {
                bg: leak(&e.roles.bg),
                card_bg: leak(&e.roles.card_bg),
                accent1: leak(&e.roles.accent1),
                accent2: leak(&e.roles.accent2),
                text: leak(&e.roles.text),
            };
        }
        let (bg, card_bg, a1, a2, text) = theme.fallback_colors();
        PaletteColors {
            bg,
            card_bg,
            accent1: a1,
            accent2: a2,
            text,
        }
    }
}

impl Default for PaletteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Leaks a String into a &'static str. Palette strings come from embedded
/// `include_str!` JSON which is alive for the program lifetime; the corpus is a
/// fixed closed set, so a bounded leak is safe and deterministic.
fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}
