//! P1 gate tests: palette library corpus loading + theme resolution.

use katsvg_engine::PaletteRegistry;
use katsvg_engine::router::PaletteTheme;

#[test]
fn registry_loads_eight_palettes() {
    let reg = PaletteRegistry::new();
    assert_eq!(reg.len(), 8, "expected 8 palette corpus entries");
    assert!(!reg.is_empty());
}

#[test]
fn all_theme_variants_resolve_colors() {
    let reg = PaletteRegistry::new();
    for theme in [
        PaletteTheme::TechDark,
        PaletteTheme::FinancialNavy,
        PaletteTheme::VibrantCoral,
        PaletteTheme::AcademicWarm,
        PaletteTheme::OceanBreeze,
        PaletteTheme::SunsetGlow,
        PaletteTheme::ForestMint,
        PaletteTheme::Monochrome,
    ] {
        let c = reg.colors(theme);
        assert!(
            c.bg.starts_with('#') && c.bg.len() == 7,
            "{theme:?} bg {:?}",
            c.bg
        );
        assert!(c.accent1.starts_with('#'), "{theme:?} accent1");
        assert!(c.accent2.starts_with('#'), "{theme:?} accent2");
        assert!(c.text.starts_with('#'), "{theme:?} text");
    }
}

#[test]
fn corpus_colors_match_fallback_for_existing_themes() {
    let reg = PaletteRegistry::new();
    for theme in [
        PaletteTheme::TechDark,
        PaletteTheme::FinancialNavy,
        PaletteTheme::VibrantCoral,
        PaletteTheme::AcademicWarm,
    ] {
        let c = reg.colors(theme);
        let (bg, card, a1, a2, text) = theme.fallback_colors();
        assert_eq!(
            (c.bg, c.card_bg, c.accent1, c.accent2, c.text),
            (bg, card, a1, a2, text)
        );
    }
}

#[test]
fn new_themes_differ_from_existing() {
    let reg = PaletteRegistry::new();
    let tech = reg.colors(PaletteTheme::TechDark);
    for theme in [
        PaletteTheme::OceanBreeze,
        PaletteTheme::SunsetGlow,
        PaletteTheme::ForestMint,
        PaletteTheme::Monochrome,
    ] {
        let c = reg.colors(theme);
        assert_ne!(c.bg, tech.bg, "{theme:?} bg should differ from TechDark");
    }
}

#[test]
fn theme_classification_reaches_new_palettes() {
    use katsvg_engine::InfographicIntentRouter;
    let r = InfographicIntentRouter::new();
    assert_eq!(
        r.parse_and_route("Ocean breeze report").theme,
        PaletteTheme::OceanBreeze
    );
    assert_eq!(
        r.parse_and_route("Sunset glow campaign poster").theme,
        PaletteTheme::SunsetGlow
    );
    assert_eq!(
        r.parse_and_route("Forest eco sustainability poster").theme,
        PaletteTheme::ForestMint
    );
    assert_eq!(
        r.parse_and_route("Minimal monochrome print").theme,
        PaletteTheme::Monochrome
    );
}
