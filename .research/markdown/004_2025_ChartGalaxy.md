# Research 004: ChartGalaxy — Million-Scale Structural Dataset for LVLMs

> **Source:** [ChartGalaxy: A Million-Scale Structural Dataset for Chart-Heavy Large Vision-Language Models](https://arxiv.org/abs/2505.18668) — Li et al., 2025. Accepted at ICLR 2026.
> **Date:** 2026-08-01
> **Status:** Done
> **Related Research:** 000 (Bylinskii), 003 (ChartSpark)
> **Related Plans:** None
> **Classification:** Public

---

## TL;DR

ChartGalaxy presents a million-scale structural multi-modal dataset ($1.76\text{M}$ samples) designed to pre-train Large Vision-Language Models (LVLMs) on chart interpretation, layout understanding, and D3.js code generation. By extracting an inductive design space across 18 leading graphic design platforms, the dataset categorizes $75$ chart types, $440$ style variations, and $68$ structural layout templates. Pre-training models like InternVL3-8B and Qwen2.5-VL-7B on ChartGalaxy yields a $+60.49\%$ improvement in style detection and $+40.78\%$ in visual encoding analysis across benchmarks (InfographicVQA, ChartQAPro).

**Distilled for FAW (modelless, inference-time):**
FAW bypasses the massive compute requirements of pre-training multi-billion parameter LVLMs. It extracts the $68$ structural layout templates and design space rules identified in ChartGalaxy and compiles them into local declarative JSON layout configurations and Rust `ConstraintPruner` rules, achieving zero-shot layout compilation under a $<12\text{GB}$ VRAM budget.

---

## 1. Paper Core Findings

### 1.1 Dataset Statistics & Multi-Modal Structure

ChartGalaxy constructs a dataset containing $1,763,189$ samples:

```
ChartGalaxy Dataset (1.76M Total Samples)
├── Real-World Scraped Corpus (61,833 infographics from Statista, Visual Capitalist)
└── Synthetic Structural Corpus (1,701,356 paired samples)
    ├── Raw Data Tables (JSON format)
    ├── Bounding Box Layout Metadata (68 Layout Templates)
    ├── Visual Vector Artifacts (SVG paths & tags)
    └── Rendered Raster Artifacts (PNG images)
```

```
+-----------------------------------------------------------------------+
|  ChartGalaxy Design Space Taxonomy                                    |
+-----------------------------------+-+---------------------------------+
| Unique Chart Types                | | 75 (Bar, Pie, Scatter, Treemap..)|
| Fine-Grained Style Variations     | | 440 (Gradients, Borders, Grids)|
| Structural Layout Templates       | | 68 (Coordinate Bounding Grids)  |
| Instruction-Tuning Visual QA Pairs| | 440,000                         |
| Pre-trained LVLM Backbone Models  | | InternVL3-8B, Qwen2.5-VL-7B     |
+-----------------------------------+-+---------------------------------+
```

### 1.2 Programmatic Synthesis Pipeline

The authors develop an automated programmatic generation pipeline:

```
[Combinatorial Raw JSON Table Sampler]
                  │
                  ▼
[Design Space Template Matcher (68 Templates)]
                  │
                  ▼
[D3.js / SVG Programmatic Renderer]
                  │
        ┌─────────┴─────────┐
        ▼                   ▼
[Paired SVG Vector]   [Rendered PNG]
```

1. **Inductive Design Space Extraction:** Analyzed real-world infographics to define $75$ discrete chart types, $440$ visual style variations, and $68$ relative layout bounding box templates ($[x, y, w, h]$).
2. **Combinatorial Table Sampling:** Generates synthetic raw JSON tables with varied numerical scales and distributions.
3. **Programmatic D3.js Synthesis:** Renders paired SVG vector trees and PNG raster images, maintaining $100\%$ data fidelity between raw table numbers and rendered bar heights/line coordinates.

---

### 1.3 Dual-Granularity Evaluation Methodology

For evaluating generated visualization code (D3.js / SVG), ChartGalaxy introduces a dual-granularity similarity metric:

```
                            Dual-Granularity Metric
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌─────────────────────────┐                           ┌─────────────────────────┐
│  High-Level Similarity  │                           │   Low-Level Similarity  │
│ (PNG Pixel Layout Map)  │                           │  (SVG XML Tag Tree)     │
└─────────────────────────┘                           └─────────────────────────┘
```

1. **High-Level Visual Similarity ($S_{\text{high}}$):** Measures structural spatial layout agreement between generated and target PNG images using deep visual feature maps.
2. **Low-Level SVG Structural Similarity ($S_{\text{low}}$):** Evaluates exact XML tag properties inside the SVG hierarchy:
   $$S_{\text{low}} = \frac{1}{|E|} \sum_{e \in E} \left( \lambda_1 \cdot \text{IoU}(\text{box}_e) + \lambda_2 \cdot \text{ColorMatch}(c_e) + \lambda_3 \cdot \mathbb{I}(\text{tag}_e == \text{tag}_{\text{target}}) \right)$$
   where $\text{box}_e = [x, y, w, h]$, $c_e$ is the hex color, and $\text{tag}_e$ is the SVG element type (`<rect>`, `<path>`, `<text>`).

3. **Evaluation Metrics:**
   - **Relaxed Accuracy ($\text{Acc}_{5\%}$):** Numerical QA accuracy with a $5\%$ error margin.
   - **ANLS (Average Normalized Levenshtein Similarity):** Measures text extraction fidelity on infographic image titles and callouts.

---

### 1.4 Benchmark Fine-Tuning Results

```
+-----------------------------------+-------------------+-------------------+-------------------+
| Model Architecture                | Style Detection % | Visual Encoding % | InfographicVQA %  |
+-----------------------------------+-------------------+-------------------+-------------------+
| Baseline Qwen2.5-VL-7B (Zero-Shot)| 28.4%             | 35.1%             | 41.2%             |
| Qwen2.5-VL-7B + ChartGalaxy SFT   | 88.9% (+60.5%)    | 75.9% (+40.8%)    | 68.7% (+27.5%)    |
| Baseline InternVL3-8B (Zero-Shot) | 31.2%             | 38.6%             | 44.8%             |
| InternVL3-8B + ChartGalaxy SFT    | 91.2% (+60.0%)    | 79.2% (+40.6%)    | 71.4% (+26.6%)    |
+-----------------------------------+-------------------+-------------------+-------------------+
```

---

## 2. Distillation for FAW

### 2.1 Transferable Primitives

1. **Declarative Layout Templates:** The $68$ layout templates and $440$ style variations provide an extensive, empirically validated design space for declarative JSON layout configuration files.
2. **Low-Level SVG Tree Validation:** Evaluating generated SVG graphics at the XML tag level ($[x, y, w, h]$ bounding box IoU and tag matching) is superior to flat pixel image evaluation.

### 2.2 System Mapping (`katsvg-engine` Pure-Rust Pipeline)

Converts ChartGalaxy's $68$ layout template coordinate rules into embedded `LayoutDef` entries in `router.rs`. Enables model-less intent routing across complex infographic layouts without needing multi-gigabyte LVLMs.

---

## 3. Verdict

- **Tier:** Gain
- **Criteria:** Actionable template structures and design space parameters can be adapted to expand FAW's declarative layout configuration database.
- **Routing:** Layout design space parameters are mapped to `katsvg-engine`.
- **One-Line Reasoning:** Actionable template structures and design space parameters can be adapted to expand FAW's local declarative JSON layout template database.
