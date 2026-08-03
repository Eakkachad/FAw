//! Localized strings (`katSVG i18n`).
//!
//! F10: routes generated chrome text (subtitle, footer) through a per-language
//! table keyed on the detected prompt language, instead of hardcoding English.
//! Only `th` and `en` are shipped; anything else falls back to English.

/// Supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    En,
    Th,
}

/// Detect language from prompt content: Thai characters → Thai, else English.
pub fn detect_lang(text: &str) -> Lang {
    if text.chars().any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c)) {
        Lang::Th
    } else {
        Lang::En
    }
}

/// A small keyed string table.
pub struct Strs;

impl Strs {
    /// Subtitle shown under the title.
    pub fn subtitle(lang: Lang) -> &'static str {
        match lang {
            Lang::En => "Generated via katSVG Neuro-Symbolic Vector Layout Engine",
            Lang::Th => "สร้างโดย katSVG Neuro-Symbolic Vector Layout Engine",
        }
    }

    /// Footer note.
    pub fn footer(lang: Lang) -> &'static str {
        match lang {
            Lang::En => "katSVG Engine • MIT License",
            Lang::Th => "katSVG Engine • สัญญาอนุญาต MIT",
        }
    }
}
