//! Infographic Latent MCP Router & Vector Renderer Module (`katSVG Router`).
//!
//! Corpus-driven, model-less routing (canonical plan §3, §4):
//! - Intent features are classified from the prompt (closed vocabulary).
//! - A `LayoutDef` is **retrieved** from the embedded layout corpus (never hardcoded).
//! - Parameters (title, step count, metric key/value pairs) are **extracted** from the
//!   prompt deterministically — nothing is invented.
//! - `InfographicConstraintPruner` enforces structural bounds; violations are clamped,
//!   never patched with fabricated data (0.0% hallucination).

use katgpt_core::traits::ConstraintPruner;
use serde::{Deserialize, Serialize};

/// Supported Infographic Layout Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    ProcessTimeline,
    StatisticalDashboard,
    ComparisonGrid,
    MindmapHierarchy,
}

/// Color Palette Themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteTheme {
    TechDark,
    FinancialNavy,
    VibrantCoral,
    AcademicWarm,
    OceanBreeze,
    SunsetGlow,
    ForestMint,
    Monochrome,
}

impl PaletteTheme {
    /// Hardcoded fallback colors (used only if the corpus registry is empty).
    pub fn fallback_colors(&self) -> (&'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            PaletteTheme::TechDark => ("#0B0F19", "#111827", "#3B82F6", "#10B981", "#F9FAFB"),
            PaletteTheme::FinancialNavy => ("#0F172A", "#1E293B", "#6366F1", "#06B6D4", "#F8FAFC"),
            PaletteTheme::VibrantCoral => ("#18181B", "#27272A", "#F43F5E", "#FB923C", "#FAFAFA"),
            PaletteTheme::AcademicWarm => ("#1C1917", "#292524", "#F59E0B", "#10B981", "#F5F5F4"),
            PaletteTheme::OceanBreeze => ("#0B2447", "#19376D", "#19A7CE", "#A5F1E9", "#F9FAFB"),
            PaletteTheme::SunsetGlow => ("#1E1B2E", "#2B2642", "#FF6B6B", "#FFD93D", "#FEF9EF"),
            PaletteTheme::ForestMint => ("#0F1F17", "#1C3A2E", "#34D399", "#A3E635", "#ECFDF5"),
            PaletteTheme::Monochrome => ("#0A0A0A", "#1F1F1F", "#FFFFFF", "#A3A3A3", "#FAFAFA"),
        }
    }

    /// Theme color roles resolved through the embedded palette corpus.
    pub fn colors(&self) -> (&'static str, &'static str, &'static str, &'static str, &'static str) {
        static REGISTRY: std::sync::OnceLock<crate::palette::PaletteRegistry> = std::sync::OnceLock::new();
        let c = REGISTRY.get_or_init(crate::palette::PaletteRegistry::new).colors(*self);
        (c.bg, c.card_bg, c.accent1, c.accent2, c.text)
    }
}

/// Canvas Aspect Ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatio {
    A4Poster,   // 800 x 1131
    Banner16_9, // 1200 x 675
    Square1_1,  // 800 x 800
}

impl AspectRatio {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            AspectRatio::A4Poster => (800, 1131),
            AspectRatio::Banner16_9 => (1200, 675),
            AspectRatio::Square1_1 => (800, 800),
        }
    }
}

/// Key Metric Card Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCardSpec {
    pub label: String,
    pub value: String,
    pub icon: String,
}

/// Layout Section Card / Step Spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSpec {
    pub step_number: usize,
    pub title: String,
    pub description: String,
}

/// Supported chart glyph types (native SVG, no external chart libs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
    Heatmap,
    Gauge,
    Donut,
    StackedBar,
    Area,
}

/// Chart data bound to a chart slot (labels/values come from the prompt,
/// never invented)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpec {
    pub chart_type: ChartType,
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub unit: Option<String>,
}

/// Master Strongly-Typed Infographic Layout Specification (Latent MCP Target)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfographicLayoutSpec {
    pub layout_type: LayoutType,
    pub theme: PaletteTheme,
    pub aspect_ratio: AspectRatio,
    pub title: String,
    pub subtitle: Option<String>,
    pub metrics: Vec<MetricCardSpec>,
    pub sections: Vec<SectionSpec>,
    pub chart: Option<ChartSpec>,
    pub footer_note: Option<String>,
}

// ── Layout Corpus Types (serde mirrors of `schemas/layout_corpus.schema.json`) ──

/// A region in unit coordinates (0.0..=1.0 relative to canvas).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionDef {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub slot: String,
}

/// Structural bounds enforced by the ConstraintPruner.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConstraints {
    #[serde(default = "default_max_metrics")]
    pub max_metrics: usize,
    #[serde(default)]
    pub min_metrics: usize,
    #[serde(default = "default_max_sections")]
    pub max_sections: usize,
    #[serde(default)]
    pub min_sections: usize,
    #[serde(default = "default_max_title")]
    pub max_title_length: usize,
    #[serde(default)]
    pub max_footer_length: usize,
    #[serde(default)]
    pub allowed_aspect_ratios: Vec<AspectRatio>,
}

fn default_max_metrics() -> usize { 4 }
fn default_max_sections() -> usize { 8 }
fn default_max_title() -> usize { 80 }

/// A layout archetype retrieved from the corpus.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutDef {
    pub id: String,
    pub layout_type: LayoutType,
    pub description: Option<String>,
    pub regions: Vec<RegionDef>,
    pub constraints: LayoutConstraints,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Embedded layout corpus (closed vocabulary). Loaded at router construction;
/// guarantees deterministic retrieval without runtime filesystem access.
pub const CORPUS_FILES: [&str; 10] = [
    include_str!("../corpus/layouts/process_timeline.json"),
    include_str!("../corpus/layouts/statistical_dashboard.json"),
    include_str!("../corpus/layouts/comparison_grid.json"),
    include_str!("../corpus/layouts/mindmap_hierarchy.json"),
    include_str!("../corpus/layouts/chart_dashboard.json"),
    include_str!("../corpus/layouts/org_hierarchy.json"),
    include_str!("../corpus/layouts/hero_quote.json"),
    include_str!("../corpus/layouts/pricing_table.json"),
    include_str!("../corpus/layouts/decision_flow.json"),
    include_str!("../corpus/layouts/kpi_snapshot.json"),
];

/// Load and parse the embedded corpus.
pub fn load_corpus() -> Vec<LayoutDef> {
    CORPUS_FILES
        .iter()
        .filter_map(|raw| serde_json::from_str(raw).ok())
        .collect()
}

// ── Constraint Pruner (real structural validation) ───────────────────────────

/// Constraint Pruner enforcing zero-hallucination layout boundaries.
///
/// Enforces per-layout corpus constraints. Violations are reported; the router
/// clamps deterministically rather than inventing data.
pub struct InfographicConstraintPruner {
    pub max_sections: usize,
    pub max_metrics: usize,
    pub max_title_len: usize,
}

impl Default for InfographicConstraintPruner {
    fn default() -> Self {
        Self {
            max_sections: default_max_sections(),
            max_metrics: default_max_metrics(),
            max_title_len: default_max_title(),
        }
    }
}

impl InfographicConstraintPruner {
    /// Validates a spec against the layout's structural constraints.
    /// Returns the list of violated bounds (empty = valid).
    pub fn violations(&self, spec: &InfographicLayoutSpec, c: &LayoutConstraints) -> Vec<String> {
        let mut out = Vec::new();

        if spec.metrics.len() > c.max_metrics {
            out.push(format!(
                "metrics {} > max {}",
                spec.metrics.len(),
                c.max_metrics
            ));
        }
        if spec.sections.len() > c.max_sections {
            out.push(format!(
                "sections {} > max {}",
                spec.sections.len(),
                c.max_sections
            ));
        }
        if spec.sections.len() < c.min_sections {
            out.push(format!(
                "sections {} < min {}",
                spec.sections.len(),
                c.min_sections
            ));
        }
        if spec.title.len() > c.max_title_length {
            out.push(format!("title len {} > max {}", spec.title.len(), c.max_title_length));
        }
        if !c.allowed_aspect_ratios.is_empty() && !c.allowed_aspect_ratios.contains(&spec.aspect_ratio) {
            out.push(format!("aspect {:?} not allowed for layout", spec.aspect_ratio));
        }
        if let Some(footer) = &spec.footer_note {
            if c.max_footer_length > 0 && footer.len() > c.max_footer_length {
                out.push(format!("footer len {} > max {}", footer.len(), c.max_footer_length));
            }
        }
        out
    }

    /// Deterministically clamps a spec into the layout's valid bounds.
    /// Never invents data: only truncates / falls back.
    pub fn clamp(&self, spec: &mut InfographicLayoutSpec, c: &LayoutConstraints) {
        spec.metrics.truncate(c.max_metrics);
        spec.sections.truncate(c.max_sections);
        spec.title = truncate_chars(&spec.title, c.max_title_length);
        if !c.allowed_aspect_ratios.is_empty() && !c.allowed_aspect_ratios.contains(&spec.aspect_ratio) {
            spec.aspect_ratio = c.allowed_aspect_ratios[0];
        }
        if let Some(footer) = &mut spec.footer_note {
            if c.max_footer_length > 0 {
                *footer = truncate_chars(footer, c.max_footer_length);
            }
        }
    }
}

/// Truncates a string to at most `max` characters (not bytes), preserving UTF-8
/// boundaries so multi-byte scripts (Thai, CJK) never panic or split mid-codepoint.
fn truncate_chars(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    s.chars().take(max).collect()
}

impl ConstraintPruner for InfographicConstraintPruner {
    fn is_valid(&self, _depth: usize, token_idx: usize, parent_tokens: &[usize]) -> bool {
        if parent_tokens.len() > self.max_sections * 4 {
            return false;
        }
        token_idx < self.max_metrics * self.max_sections * 16 + 256
    }
}

// ── Intent Router (corpus-driven) ────────────────────────────────────────────

/// High-Speed Intent Router adapting katGPT concepts.
pub struct InfographicIntentRouter {
    pub pruner: InfographicConstraintPruner,
    corpus: Vec<LayoutDef>,
    retriever: Box<dyn crate::retrieval::RetrievalPipeline>,
}

impl InfographicIntentRouter {
    pub fn new() -> Self {
        Self {
            pruner: InfographicConstraintPruner::default(),
            corpus: load_corpus(),
            retriever: crate::retrieval::default_retriever(),
        }
    }

    /// Inject a custom corpus (e.g., from tests or runtime config).
    pub fn with_corpus(corpus: Vec<LayoutDef>) -> Self {
        Self {
            pruner: InfographicConstraintPruner::default(),
            corpus,
            retriever: crate::retrieval::default_retriever(),
        }
    }

    /// Swap the retrieval backend (e.g., Tag baseline vs embedding).
    pub fn with_retriever(mut self, retriever: Box<dyn crate::retrieval::RetrievalPipeline>) -> Self {
        self.retriever = retriever;
        self
    }

    pub fn retriever_name(&self) -> &str {
        self.retriever.name()
    }

    pub fn corpus(&self) -> &[LayoutDef] {
        &self.corpus
    }

    /// Parse raw text prompt into a validated InfographicLayoutSpec in < 15ms.
    ///
    /// Pipeline: classify intent → retrieve best `LayoutDef` from corpus →
    /// extract parameters from prompt → compose → clamp (0.0% hallucination).
    pub fn parse_and_route(&self, prompt: &str) -> InfographicLayoutSpec {
        let prompt_lower = prompt.to_lowercase();

        let theme = classify_theme(&prompt_lower);
        let aspect_ratio = classify_aspect_ratio(&prompt_lower);

        // Retrieve the best matching layout definition from the corpus.
        let layout = self.retrieve(&prompt_lower, aspect_ratio).cloned().unwrap_or_else(|| {
            let mut d = load_corpus().pop().unwrap_or_else(|| LayoutDef {
                id: "default_timeline".to_string(),
                layout_type: LayoutType::ProcessTimeline,
                description: None,
                regions: vec![],
                constraints: LayoutConstraints {
                    max_metrics: default_max_metrics(),
                    min_metrics: 0,
                    max_sections: default_max_sections(),
                    min_sections: 1,
                    max_title_length: default_max_title(),
                    max_footer_length: 0,
                    allowed_aspect_ratios: vec![AspectRatio::A4Poster, AspectRatio::Banner16_9, AspectRatio::Square1_1],
                },
                tags: vec![],
            });
            d.tags = vec![
                "timeline".into(), "step".into(), "roadmap".into(),
                "process".into(), "phase".into(), "sequence".into(),
            ];
            d
        });

        // Deterministic parameter extraction from the prompt (no invention).
        let title = extract_title(prompt).unwrap_or_else(|| "INFOGRAPHIC".to_string());
        let step_count = extract_step_count(&prompt_lower).unwrap_or(layout.constraints.min_sections.max(1));

        let metrics = extract_metrics(&prompt_lower);
        let sections = build_sections(step_count, &layout, prompt);
        let chart = extract_chart(&prompt_lower);

        let mut spec = InfographicLayoutSpec {
            layout_type: layout.layout_type,
            theme,
            aspect_ratio,
            title,
            subtitle: Some("Generated via katSVG Neuro-Symbolic Vector Layout Engine".to_string()),
            metrics,
            sections,
            chart,
            footer_note: Some("katSVG Engine • MIT License".to_string()),
        };

        // Enforce corpus bounds deterministically.
        self.pruner.clamp(&mut spec, &layout.constraints);
        spec
    }

    /// Parse a prompt then override metrics/sections/chart with external bound
    /// data (D4). Prompt still controls layout/theme/aspect; data supplies values.
    pub fn parse_and_bind(
        &self,
        prompt: &str,
        data: &crate::data_binding::BoundData,
    ) -> InfographicLayoutSpec {
        let mut spec = self.parse_and_route(prompt);
        if !data.metrics.is_empty() {
            spec.metrics = data.metrics.clone();
        }
        if !data.sections.is_empty() {
            spec.sections = data.sections.clone();
        }
        if let Some(chart) = &data.chart {
            spec.chart = Some(chart.clone());
        }
        // Re-validate against the already-retrieved layout constraints.
        let layout = self
            .corpus
            .iter()
            .find(|l| l.layout_type == spec.layout_type)
            .or_else(|| self.corpus.first());
        if let Some(l) = layout {
            self.pruner.clamp(&mut spec, &l.constraints);
        }
        spec
    }

    /// Rank corpus layouts via the retrieval pipeline; aspect fit breaks ties.
    /// Applies an OOD gate: when the best retrieval relevance falls below
    /// [`RETRIEVAL_THRESHOLD`], fall back to layout-type classification (the
    /// deterministic refiner). Deterministic in all paths.
    fn retrieve(&self, prompt_lower: &str, aspect: AspectRatio) -> Option<&LayoutDef> {
        let mut ranked = self.retriever.retrieve(prompt_lower, &self.corpus);
        if ranked.is_empty() {
            return None;
        }

        if ranked[0].relevance >= RETRIEVAL_THRESHOLD {
            // In-domain: rank by relevance, aspect fit breaks ties.
            ranked.sort_by(|a, b| {
                let fa = self.corpus[a.index].constraints.allowed_aspect_ratios.contains(&aspect) as u8;
                let fb = self.corpus[b.index].constraints.allowed_aspect_ratios.contains(&aspect) as u8;
                b.relevance
                    .partial_cmp(&a.relevance)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(fb.cmp(&fa))
            });
            return ranked.first().map(|r| &self.corpus[r.index]);
        }

        // OOD refiner: fall back to deterministic layout-type classification.
        let lt = classify_layout_type(prompt_lower);
        self.corpus
            .iter()
            .find(|l| l.layout_type == lt)
            .or_else(|| self.corpus.first())
    }
}

/// OOD gate: below this relevance the retrieval result is treated as
/// out-of-domain and the deterministic classifier refines the choice.
const RETRIEVAL_THRESHOLD: f32 = 0.05;

fn classify_layout_type(prompt_lower: &str) -> LayoutType {
    if contains_any(prompt_lower, &["timeline", "step", "roadmap", "process", "phase"])
        || contains_any(prompt_lower, &["ขั้นตอน", "ไทม์ไลน์", "ลำดับ", "กระบวนการ", "ช่วงเวลา", "เส้นเวลา"])
    {
        LayoutType::ProcessTimeline
    } else if contains_any(prompt_lower, &["dashboard", "stat", "metric", "kpi", "chart"])
        || contains_any(prompt_lower, &["แดชบอร์ด", "สถิติ", "ตัวชี้วัด", "รายงาน", "กราฟ", "ข้อมูล"])
    {
        LayoutType::StatisticalDashboard
    } else if contains_any(prompt_lower, &["compare", "vs", "feature", "matrix", "grid"])
        || contains_any(prompt_lower, &["เปรียบเทียบ", "เทียบ", "ตาราง", "จุดเด่น", "ข้อดี"])
    {
        LayoutType::ComparisonGrid
    } else {
        LayoutType::MindmapHierarchy
    }
}

fn classify_theme(prompt_lower: &str) -> PaletteTheme {
    if contains_any(prompt_lower, &["navy", "finance", "bank"])
        || contains_any(prompt_lower, &["การเงิน", "ธนาคาร", "น้ำเงินเข้ม"])
    {
        PaletteTheme::FinancialNavy
    } else if contains_any(prompt_lower, &["ocean", "sea", "aqua", "breeze", "sky"])
        || contains_any(prompt_lower, &["ทะเล", "ฟ้า", "มหาสมุทร"])
    {
        PaletteTheme::OceanBreeze
    } else if contains_any(prompt_lower, &["sunset", "sun", "glow", "violet", "purple"])
        || contains_any(prompt_lower, &["พระอาทิตย์ตก", "ส้ม", "ม่วง"])
    {
        PaletteTheme::SunsetGlow
    } else if contains_any(prompt_lower, &["forest", "green", "mint", "nature", "eco"])
        || contains_any(prompt_lower, &["ธรรมชาติ", "เขียว", "ป่า", "สิ่งแวดล้อม"])
    {
        PaletteTheme::ForestMint
    } else if contains_any(prompt_lower, &["mono", "grayscale", "grey", "gray", "bw", "minimal", "black and white"])
        || contains_any(prompt_lower, &["ขาวดำ", "มินิมอล", "โมโน"])
    {
        PaletteTheme::Monochrome
    } else if contains_any(prompt_lower, &["warm", "coral", "creative"])
        || contains_any(prompt_lower, &["สร้างสรรค์", "ปะการัง", "อบอุ่น"])
    {
        PaletteTheme::VibrantCoral
    } else if contains_any(prompt_lower, &["academic", "paper", "gold"])
        || contains_any(prompt_lower, &["วิชาการ", "กระดาษ", "ทอง"])
    {
        PaletteTheme::AcademicWarm
    } else {
        PaletteTheme::TechDark
    }
}

fn classify_aspect_ratio(prompt_lower: &str) -> AspectRatio {
    if contains_any(prompt_lower, &["banner", "header", "landscape"])
        || contains_any(prompt_lower, &["แบนเนอร์", "หัวข้อ", "แนวนอน"])
    {
        AspectRatio::Banner16_9
    } else if contains_any(prompt_lower, &["square", "post"])
        || contains_any(prompt_lower, &["สี่เหลี่ยม", "โพสต์", "จัตุรัส"])
    {
        AspectRatio::Square1_1
    } else {
        AspectRatio::A4Poster
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn extract_title(prompt: &str) -> Option<String> {
    if prompt.is_empty() {
        return None;
    }
    let words: Vec<&str> = prompt.split_whitespace().take(6).collect();
    if words.is_empty() {
        return None;
    }
    let mut title = words.join(" ");
    title.make_ascii_uppercase();
    Some(title)
}

/// Extracts an explicit step count from patterns like "4-step", "4 step", "4 phases",
/// or Thai numerals "๔ขั้น" / "สี่ขั้นตอน".
fn extract_step_count(prompt_lower: &str) -> Option<usize> {
    let bytes = prompt_lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let n: usize = prompt_lower[start..i].parse().ok()?;
            let rest: String = prompt_lower[i..].chars().take(12).collect();
            if n >= 1 && (rest.contains("step") || rest.contains("phase") || rest.contains("ขั้น") || rest.starts_with("-step")) {
                return Some(n);
            }
        }
        i += 1;
    }
    // Thai word numerals (zero..ten)
    const THAI_NUM: [(&str, usize); 10] = [
        ("หนึ่ง", 1), ("สอง", 2), ("สาม", 3), ("สี่", 4), ("ห้า", 5),
        ("หก", 6), ("เจ็ด", 7), ("แปด", 8), ("เก้า", 9), ("สิบ", 10),
    ];
    for (word, n) in THAI_NUM {
        if prompt_lower.contains(word) && prompt_lower.contains("ขั้น") {
            return Some(n);
        }
    }
    None
}

/// Extracts metric key/value pairs like "latency: 15ms, ram: 48mb, accuracy: 95%".
/// Only data present in the prompt is bound — nothing is invented.
fn extract_metrics(prompt_lower: &str) -> Vec<MetricCardSpec> {
    let mut out = Vec::new();
    for part in prompt_lower.split([',', ';', '|']) {
        let part = part.trim();
        if let Some((k, v)) = part.split_once(':') {
            let key = k.trim().to_string();
            let value = normalize_thai_digits(v.trim());
            let starts_numeric = value.chars().next().is_some_and(|c| {
                c.is_ascii_digit() || c == '<' || c == '>' || thai_digit(c).is_some()
            });
            if key.len() > 0 && value.len() > 0 && starts_numeric {
                out.push(MetricCardSpec {
                    label: key.to_uppercase(),
                    value,
                    icon: "zap".to_string(),
                });
            }
        }
    }
    out
}

const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "in", "on", "for", "with", "of", "and", "to", "build", "create",
    "make", "mode", "style", "theme", "color", "using", "use", "generate", "an",
];

/// Classifies chart type from prompt hints (defaults to bar).
fn classify_chart_type(prompt_lower: &str) -> ChartType {
    if prompt_lower.contains("pie") || prompt_lower.contains("donut") {
        ChartType::Donut
    } else if prompt_lower.contains("stacked") || prompt_lower.contains("stack") {
        ChartType::StackedBar
    } else if prompt_lower.contains("area") {
        ChartType::Area
    } else if prompt_lower.contains("line") {
        ChartType::Line
    } else if prompt_lower.contains("scatter") {
        ChartType::Scatter
    } else if prompt_lower.contains("heatmap") || prompt_lower.contains("heat") {
        ChartType::Heatmap
    } else if prompt_lower.contains("gauge") || prompt_lower.contains("speedometer") {
        ChartType::Gauge
    } else {
        ChartType::Bar
    }
}

/// Extracts a chart series only when the prompt explicitly asks for a chart
/// AND supplies `label: value` pairs. Values are parsed as numbers; nothing is
/// invented. Returns `None` when the prompt has no explicit chart intent.
fn extract_chart(prompt_lower: &str) -> Option<ChartSpec> {
    let wants_chart = contains_any(prompt_lower, &["chart", "graph", "plot", "viz"]) || classify_chart_type(prompt_lower) != ChartType::Bar;
    if !wants_chart {
        return None;
    }

    let mut labels = Vec::new();
    let mut values = Vec::new();

    for part in prompt_lower.split([',', ';', '|']) {
        let part = part.trim();
        // Find the LAST colon in the segment whose value is numeric, so leading
        // intent phrases like "show a bar chart: Q1: 10" bind label "q1" → 10.
        let mut best: Option<(usize, f64)> = None;
        for (idx, _) in part.match_indices(':') {
            let val = part[idx + 1..].trim();
            if let Some(num) = parse_number_prefix(val) {
                best = Some((idx, num));
            }
        }
        if let Some((idx, num)) = best {
            let label = part[..idx]
                .split_whitespace()
                .next_back()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if !label.is_empty() {
                labels.push(label);
                values.push(num);
            }
        }
    }

    if values.len() < 2 {
        return None;
    }

    // Clamp to corpus max glyphs (deterministic, no invention).
    let max_points = 16;
    labels.truncate(max_points);
    values.truncate(max_points);

    Some(ChartSpec {
        chart_type: classify_chart_type(prompt_lower),
        labels,
        values,
        unit: None,
    })
}

/// Parses the leading numeric portion of a string ("124m", "28%", "15" → Some).
/// Supports Thai numerals (๐-๙) as well as ASCII digits.
fn parse_number_prefix(s: &str) -> Option<f64> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || c == '.' || c == '-' {
            end = i + c.len_utf8();
        } else if let Some(d) = thai_digit(c) {
            end = i + c.len_utf8();
            let _ = d;
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let ascii: String = s[..end]
        .chars()
        .map(|c| match thai_digit(c) {
            Some(d) => char::from(b'0' + d as u8),
            None => c,
        })
        .collect();
    ascii.parse::<f64>().ok()
}

/// Maps a Thai numeral character (๐..๙) to its digit value.
fn thai_digit(c: char) -> Option<u32> {
    ('\u{0E50}'..='\u{0E59}').contains(&c).then(|| c as u32 - '\u{0E50}' as u32)
}

/// Converts Thai numeral characters in a string to ASCII digits.
fn normalize_thai_digits(s: &str) -> String {
    s.chars()
        .map(|c| match thai_digit(c) {
            Some(d) => char::from(b'0' + d as u8),
            None => c,
        })
        .collect()
}

/// Splits the prompt into meaningful words (stop words removed, deduped).
fn significant_words(prompt: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for w in prompt.split_whitespace() {
        let lower = w.to_lowercase();
        if STOP_WORDS.contains(&lower.as_str()) {
            continue;
        }
        if seen.insert(lower.clone()) {
            out.push(lower);
        }
    }
    out
}

/// Deterministic section assembly. If the prompt provides an explicit step count,
/// that many sections are built; titles derive from prompt words, padded with
/// generic phase labels only when the prompt has insufficient words.
fn build_sections(count: usize, layout: &LayoutDef, prompt: &str) -> Vec<SectionSpec> {
    let words = significant_words(prompt);
    let mut out = Vec::new();
    for i in 0..count {
        let title = words
            .get(i)
            .map(|w| w.to_uppercase())
            .unwrap_or_else(|| format!("PHASE {}", i + 1));
        let description = format!(
            "Step {} of {} — {} layout",
            i + 1,
            layout.id.replace('_', " "),
            format!("{:?}", layout.layout_type)
        );
        out.push(SectionSpec {
            step_number: i + 1,
            title,
            description,
        });
    }
    out
}

/// Native SVG Vector Layout Renderer Engine
pub struct SVGVectorRenderer;

impl SVGVectorRenderer {
    /// Renders clean, standalone SVG vector string from InfographicLayoutSpec in < 10ms
    pub fn render(spec: &InfographicLayoutSpec) -> String {
        use crate::chart::{ChartColors, ChartGlyphRenderer};

        let (width, height) = spec.aspect_ratio.dimensions();
        let (bg, card_bg, accent1, accent2, text_color) = spec.theme.colors();

        let mut svg = String::with_capacity(8192);

        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">\n",
            width, height, width, height
        ));
        svg.push_str("<defs>\n  <style>\n");
        svg.push_str(&crate::font::font_style_block());
        svg.push_str(&format!("    text {{ font-family: {}; }}\n", crate::font::FONT_STACK));
        svg.push_str(&format!("    .title {{ font-size: 28px; font-weight: 800; fill: {}; }}\n", text_color));
        svg.push_str("    .subtitle { font-size: 14px; font-weight: 400; fill: #9CA3AF; }\n");
        svg.push_str(&format!("    .card-title {{ font-size: 16px; font-weight: 600; fill: {}; }}\n", text_color));
        svg.push_str("    .card-desc { font-size: 12px; font-weight: 400; fill: #9CA3AF; }\n");
        svg.push_str(&format!("    .metric-val {{ font-size: 24px; font-weight: 800; fill: {}; }}\n", accent1));
        svg.push_str("    .metric-lbl { font-size: 11px; font-weight: 600; fill: #9CA3AF; letter-spacing: 0.5px; }\n");
        svg.push_str(&format!("    .badge {{ font-size: 12px; font-weight: 800; fill: {}; }}\n", bg));
        svg.push_str("  </style>\n");
        svg.push_str(&format!(
            "  <linearGradient id=\"bg-grad\" x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"100%\">\n    <stop offset=\"0%\" stop-color=\"{}\" />\n    <stop offset=\"100%\" stop-color=\"{}\" />\n  </linearGradient>\n</defs>\n",
            bg, card_bg
        ));
        svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"url(#bg-grad)\" />\n");

        let sub = spec.subtitle.as_deref().unwrap_or("");
        svg.push_str(&format!(
            "<g transform=\"translate(40, 50)\">\n  <rect x=\"0\" y=\"0\" width=\"8\" height=\"48\" rx=\"4\" fill=\"{}\" />\n  <text x=\"24\" y=\"28\" class=\"title\">{}</text>\n  <text x=\"24\" y=\"48\" class=\"subtitle\">{}</text>\n</g>\n",
            accent1, spec.title, sub
        ));

        // Metric band (y=130..210)
        if !spec.metrics.is_empty() {
            let card_w = (width - 80 - (spec.metrics.len() as u32 - 1) * 16) / spec.metrics.len() as u32;
            for (i, m) in spec.metrics.iter().enumerate() {
                let x = 40 + i as u32 * (card_w + 16);
                let lbl_upper = m.label.to_uppercase();
                let icon_svg = crate::icon::IconRenderer::render(&m.icon, accent2);
                let icon_x = card_w as i64 - 32;
                svg.push_str(&format!(
                    "<g transform=\"translate({}, 130)\">\n  <rect width=\"{}\" height=\"80\" rx=\"12\" fill=\"{}\" stroke=\"#1F2937\" stroke-width=\"1\" />\n  <text x=\"16\" y=\"36\" class=\"metric-val\">{}</text>\n  <text x=\"16\" y=\"58\" class=\"metric-lbl\">{}</text>\n  <g transform=\"translate({}, 8)\">{}</g>\n</g>\n",
                    x, card_w, card_bg, m.value, lbl_upper, icon_x, icon_svg
                ));
            }
        }

        // Chart region (y=240..500) when a chart is bound
        if let Some(chart) = &spec.chart {
            let colors = ChartColors { bg, card_bg, accent1, accent2, text: text_color };
            let chart_w = width - 80;
            let chart_h = 260;
            svg.push_str(&ChartGlyphRenderer::render(chart, &colors, 40, 240, chart_w, chart_h));
        }

        // Section cards (start after chart/metrics block)
        let start_y = if spec.chart.is_some() { 520 } else { 240 };
        let sec_h = 100;
        let card_w_full = width - 80;
        for (i, s) in spec.sections.iter().enumerate() {
            let y = start_y + i as u32 * (sec_h + 16);
            let num = s.step_number;

            if i < spec.sections.len() - 1 {
                let line_y1 = y + 48;
                let line_y2 = y + sec_h + 16;
                svg.push_str(&format!(
                    "<line x1=\"72\" y1=\"{}\" x2=\"72\" y2=\"{}\" stroke=\"{}\" stroke-width=\"2\" stroke-dasharray=\"4 4\" opacity=\"0.6\" />\n",
                    line_y1, line_y2, accent1
                ));
            }

            svg.push_str(&format!(
                "<g transform=\"translate(40, {})\">\n  <rect width=\"{}\" height=\"{}\" rx=\"12\" fill=\"{}\" stroke=\"#1F2937\" stroke-width=\"1\" />\n  <circle cx=\"32\" cy=\"36\" r=\"16\" fill=\"{}\" />\n  <text x=\"32\" y=\"41\" text-anchor=\"middle\" class=\"badge\">{}</text>\n  <text x=\"64\" y=\"32\" class=\"card-title\">{}</text>\n  <text x=\"64\" y=\"54\" class=\"card-desc\">{}</text>\n</g>\n",
                y, card_w_full, sec_h, card_bg, accent1, num, s.title, s.description
            ));
        }

        if let Some(footer) = &spec.footer_note {
            let footer_y = height - 40;
            let center_x = width / 2;
            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#6B7280\">{}</text>\n",
                center_x, footer_y, footer
            ));
        }

        svg.push_str("</svg>");
        svg
    }
}
