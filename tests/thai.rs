//! P6 gate tests: Thai-language intent parsing.

use katsvg_engine::router::{AspectRatio, LayoutType, PaletteTheme};
use katsvg_engine::InfographicIntentRouter;

#[test]
fn thai_layout_classification() {
    let r = InfographicIntentRouter::new();
    assert_eq!(
        r.parse_and_route("สร้างไทม์ไลน์การพัฒนาระบบ 4 ขั้นตอน").layout_type,
        LayoutType::ProcessTimeline
    );
    assert_eq!(
        r.parse_and_route("แดชบอร์ดแสดงสถิติยอดขายรายเดือน").layout_type,
        LayoutType::StatisticalDashboard
    );
    assert_eq!(
        r.parse_and_route("เปรียบเทียบฟีเจอร์ระหว่างสองผลิตภัณฑ์").layout_type,
        LayoutType::ComparisonGrid
    );
}

#[test]
fn thai_theme_classification() {
    let r = InfographicIntentRouter::new();
    assert_eq!(r.parse_and_route("รายงานการเงินสีน้ำเงินเข้ม").theme, PaletteTheme::FinancialNavy);
    assert_eq!(r.parse_and_route("โปสเตอร์ธรรมชาติสีเขียว").theme, PaletteTheme::ForestMint);
    assert_eq!(r.parse_and_route("โปสเตอร์สร้างสรรค์สีปะการัง").theme, PaletteTheme::VibrantCoral);
    assert_eq!(r.parse_and_route("เอกสารวิชาการโทนทอง").theme, PaletteTheme::AcademicWarm);
}

#[test]
fn thai_aspect_classification() {
    let r = InfographicIntentRouter::new();
    // timeline layout allows banner → preserved
    assert_eq!(r.parse_and_route("สร้างแบนเนอร์ไทม์ไลน์แนวนอน").aspect_ratio, AspectRatio::Banner16_9);
    // square → Square1_1
    assert_eq!(r.parse_and_route("โพสต์สี่เหลี่ยมจัตุรัส").aspect_ratio, AspectRatio::Square1_1);
}

#[test]
fn thai_numeric_step_count() {
    let r = InfographicIntentRouter::new();
    // Thai word numeral "สี่ขั้น" → 4 sections
    let spec = r.parse_and_route("สร้างไทม์ไลน์การพัฒนาระบบสี่ขั้น");
    assert_eq!(spec.sections.len(), 4, "Thai word numeral should yield 4 sections");
}

#[test]
fn thai_metric_extraction() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("รายงานรายได้: ๑๒๔ล้าน, ผู้ใช้: ๑๒ล้าน");
    // Thai numerals parsed into metric values (124M, 12M)
    assert!(spec.metrics.len() >= 1, "should extract metrics from Thai numeral pairs");
    let values: Vec<&str> = spec.metrics.iter().map(|m| m.value.as_str()).collect();
    assert!(values.iter().any(|v| v.contains("124")), "expected 124 from ๑๒๔, got {values:?}");
}

#[test]
fn thai_mixed_english_prompt() {
    let r = InfographicIntentRouter::new();
    let spec = r.parse_and_route("show bar chart ขายรายเดือน: มกราคม: 10, กุมภาพันธ์: 25");
    assert!(spec.chart.is_some(), "mixed TH/EN chart prompt should bind a chart");
}
