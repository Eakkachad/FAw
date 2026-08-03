//! F10 gate tests: i18n chrome text (subtitle/footer follow prompt language).

use katsvg_engine::strs::{Lang, Strs, detect_lang};
use katsvg_engine::InfographicIntentRouter;

#[test]
fn detect_lang_routes_thai_and_english() {
    assert_eq!(detect_lang("รายงานการเงิน"), Lang::Th);
    assert_eq!(detect_lang("financial report"), Lang::En);
    assert_eq!(detect_lang("mixed ไทย report"), Lang::Th);
}

#[test]
fn thai_prompt_gets_thai_subtitle_and_footer() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("สร้างไทม์ไลน์การพัฒนาระบบ");
    assert_eq!(spec.lang, Lang::Th);
    assert!(spec.subtitle.as_deref().unwrap().contains("สร้างโดย"), "Thai subtitle");
    assert!(spec.footer_note.as_deref().unwrap().contains("สัญญาอนุญาต"), "Thai footer");
}

#[test]
fn english_prompt_gets_english_chrome() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("Build a timeline");
    assert_eq!(spec.lang, Lang::En);
    assert!(spec.subtitle.as_deref().unwrap().contains("Generated via"), "English subtitle");
}

#[test]
fn strs_table_has_both_languages() {
    assert_ne!(Strs::subtitle(Lang::En), Strs::subtitle(Lang::Th));
    assert_ne!(Strs::footer(Lang::En), Strs::footer(Lang::Th));
}
