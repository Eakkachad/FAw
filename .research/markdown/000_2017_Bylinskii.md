# Research 000: Bylinskii et al. — Structural Mapping & Saliency of Infographics

> **Source:** [Understanding Infographics through Textual and Visual Features](https://arxiv.org/abs/1709.09215) — Zoya Bylinskii, Alireza Aliannejadi, Jeremy Brand, Fredo Durand, Aude Oliva (MIT CSAIL & Adobe Research), Sep 2017.
> **Date:** 2026-08-01
> **Status:** Done
> **Related Research:** 001 (LIDA), 003 (ChartSpark), 004 (ChartGalaxy)
> **Related Plans:** None
> **Classification:** Public

---

## TL;DR

This seminal paper establishes a formal visual and textual element taxonomy for infographics and analyzes how humans read and process complex graphic design layouts. By annotating bounding boxes across 29K infographics and running eye-tracking experiments with 35 participants on 630 designs, the paper proves that human visual entry points are dominated by textual headers and statistical callouts (>67% of initial fixations), while visual charts and pictograms guide post-entry exploration. The paper provides the theoretical basis for coordinate-based bounding box grids ($[x, y, w, h]$ layout region specs) and structural element separation.

**Distilled for FAW (modelless, inference-time):**
Provides the mathematical layout coordinate model ($[x, y, w, h]$ bounding box regions) to decouple editable text spans from underlying graphics. Validates FAW's visual hierarchy priority (Title $\rightarrow$ Metric Callout $\rightarrow$ Section Header $\rightarrow$ Body Paragraph $\rightarrow$ SVG Graphic Canvas).

---

## 1. Paper Core Findings

### 1.1 Structural Element Taxonomy

The authors partition infographic elements into two distinct categories: **Textual Tags** and **Visual Elements**, defining a 22-class fine-grained taxonomy:

```
Infographic Layout
├── Textual Elements (10 classes)
│   ├── Title / Subtitle
│   ├── Section Header
│   ├── Body Paragraph
│   ├── Data Label / Statistical Callout
│   ├── Legend Label / Axis Text
│   └── Source / Credits / Footnote
└── Visual Elements (12 classes)
    ├── Statistical Graphics (Bar, Line, Pie, Scatter, Donut)
    ├── Structural Diagrams (Flowchart, Process Map, Network Tree)
    ├── Pictorial Representation (Pictogram, Illustration, Photo, Icon)
    └── Structural Decorators (Border, Divider Line, Container Box)
```

Each element $e_i$ is parameterized as a tuple:
$$e_i = (c_i, [x_i, y_i, w_i, h_i], z_i, \tau_i)$$
where $c_i$ is the semantic class, $[x_i, y_i, w_i, h_i]$ is the normalized bounding box in $[0, 1]^4$, $z_i$ is the z-index rendering depth, and $\tau_i$ contains textual or path data.

### 1.2 Dataset Construction & Annotation (Visually-29K & Visually-630)

- **Visually-29K:** Scraped 63,000 raw infographics from *Visually* (`visual.ly`), filtered out corrupt or low-resolution images to yield 29,328 clean infographics across 26 topic categories (Business, Economy, Education, Environment, Health, Technology, Transportation, etc.).
- **Visually-630 (Fine-Grained Bounding Box Corpus):** A curated subset of 630 infographics manually annotated with 12,415 element bounding boxes, averaging $19.7 \pm 4.2$ discrete elements per infographic.

```
+-----------------------------------------------------------------------+
|  Visually Dataset Breakdown                                           |
+-----------------------------------+-+---------------------------------+
| Total Raw Scraped                 | | 63,000                          |
| Filtered Clean Infographics       | | 29,328                          |
| Bounding Box Annotated Subset     | | 630                             |
| Total Annotated Bounding Boxes    | | 12,415                          |
| Average Elements per Infographic  | | 19.7                            |
| Eye-Tracking Human Participants   | | 35                              |
+-----------------------------------+-+---------------------------------+
```

### 1.3 Human Eye-Tracking & Visual Saliency Analysis

The authors conducted an eye-tracking experiment using an Eyelink 1000 eye-tracker on 35 human participants viewing 630 infographics for 2000 ms each.

#### Headline Findings:
1. **Entry Point Dominance:** $67.4\%$ of first fixations ($0 - 500\text{ ms}$) land on textual elements—specifically **large titles** ($38.2\%$) and **numerical statistical callouts** ($29.2\%$).
2. **Visual Traversal:** Between $500 - 2000\text{ ms}$, visual fixations shift from text to **charts and diagrams** ($54.1\%$), validating that text establishes the semantic context while visual graphics provide detailed verification.
3. **Saliency Prediction Model:** Trained a deep convolutional saliency predictor (ResNet-50 backbone) to predict fixation heatmaps $S(x,y)$, demonstrating that text-mask channels increase saliency prediction AUC from $0.78$ to $0.89$.

```
Fixation Timeline:
[0ms ------------ 500ms ------------ 1200ms ------------ 2000ms]
  |-- Text Titles (38.2%) --|
  |-- Data Callouts (29.2%)--|
                             |-- Charts & Graphs (54.1%) --|
                             |-- Diagrams & Icons (32.4%) -|
```

### 1.4 Automatic Element Parsing Pipeline

The paper proposes a machine learning pipeline to automatically decompose a flat infographic image into structured element bounding boxes:

1. **Candidate Proposal:** Generates candidate region proposals via Selective Search and Faster R-CNN ($N \approx 2000$ region proposals per image).
2. **Feature Extraction:** Extracts deep visual features using ResNet-50 ($2048$-dim) and textual features via OCR text length, character height, and font-weight ratios.
3. **Multi-Class SVM Classifier:** Classifies each region proposal into one of the 22 element classes with Non-Maximum Suppression (NMS, $\text{IoU} > 0.3$).
4. **Layout Directed Acyclic Graph (DAG):** Reconstructs the reading order by sorting bounding boxes top-to-bottom, left-to-right:
   $$\text{Score}(e_i, e_j) = \alpha \cdot (y_j - y_i) + \beta \cdot (x_j - x_i) + \gamma \cdot \mathbb{I}(\text{same\_container})$$

### 1.5 Empirical Performance

```
+-----------------------------------+-------------------+-------------------+
| Model Architecture                | Element IoU mAP   | Saliency AUC      |
+-----------------------------------+-------------------+-------------------+
| Baseline Faster R-CNN (Visual)    | 58.4%             | 0.78              |
| Dual Visual + Text OCR Pipeline   | 74.6%             | 0.85              |
| ResNet-50 + Spatial Container DAG | 84.2%             | 0.89              |
+-----------------------------------+-------------------+-------------------+
```

---

## 2. Distillation for FAW

### 2.1 Transferable Primitives

1. **Separation of Text Tags and Visual Assets:** Infographic rendering must preserve text tags (titles, metrics, section headers) as distinct, un-baked text nodes rather than flattening them into image pixels.
2. **Saliency-Driven Layout Scaling:** Bounding box sizes for Key Metric Cards ($[x, y, w, h]$) should scale dynamically based on numerical callout prominence to match human eye-tracking entry priorities.
3. **Container-Based Region Specs:** Dividing the canvas into explicit `RegionDef` blocks (Header, Metric Band, Section Grid, Footer) matches the spatial container DAG structure.

### 2.2 System Mapping (`katsvg-engine` Pure-Rust Pipeline)

Implements the spatial container DAG via `RegionDef` and `LayoutDef` in `router.rs`. Layout templates (e.g. `ProcessTimeline`, `StatisticalDashboard`, `ComparisonGrid`) map directly to the structured bounding box layout classes identified in the paper. Elements are isolated as independent vector text nodes and chart paths to preserve visual readability.

---

## 3. Verdict

- **Tier:** Pass
- **Criteria:** Mechanism already ships as the foundation of FAW's declarative layout coordinate system.
- **Routing:** Baseline taxonomy and spatial container bounding-box representation are integrated into `katsvg-engine`.
- **One-Line Reasoning:** The element taxonomy and coordinate bounding-box grid representation are integrated as the core layout data structure of the compositor.
