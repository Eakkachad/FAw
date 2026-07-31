# Research 003: ChartSpark — Semantic Chart Re-rendering via Diffusion

> **Source:** [ChartSpark: Re-rendering Canva Layouts with Generative AI](https://arxiv.org/abs/2304.14630) — Zichun Xiao et al., 2024.
> **Date:** 2026-08-01
> **Status:** Done
> **Related Research:** 000 (Bylinskii), 004 (ChartGalaxy)
> **Related Plans:** None
> **Classification:** Public

---

## TL;DR

ChartSpark proposes a generative diffusion framework to infuse semantic artistic themes, textures, and stylized visual backgrounds into mathematically exact data charts (bar charts, pie charts, line graphs). By applying ControlNet edge guidance (Canny/Hough line masks) combined with spatial cross-attention masking, ChartSpark achieves a $96.8\%$ Data Preservation Index (DPI) while transforming plain vector charts into stylized graphics. However, because ChartSpark operates entirely within the latent pixel space of U-Net diffusion models, the output is baked into a flat raster image, destroying SVG vector paths and making text labels non-editable.

**Distilled for FAW (modelless, inference-time):**
FAW explicitly rejects ChartSpark's flat image-baking paradigm. Instead, FAW restricts latent diffusion models (SD-Turbo / Flux-Schnell) to generating standalone background/icon assets as isolated SVG image blocks, while maintaining all chart paths and Thai text layers as native, fully-editable vector nodes.

---

## 1. Paper Core Findings

### 1.1 Controlled Latent Diffusion Architecture

ChartSpark blends natural language style prompts with exact numerical chart layouts:

```
[Raw Vector Chart SVG]
         │
         ▼ (Extract Canny Edge Map & Data Mask)
┌────────────────────────────────────────────────────────┐
│ Spatial Control Inputs:                                │
│ - Edge Map C(x, y) via Canny edge detector             │
│ - Binary Data Mask M_chart(x, y) for chart bar boundaries│
└────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────┐
│ U-Net Latent Diffusion (Stable Diffusion + ControlNet):│
│ Cross-Attention Masking:                               │
│ A_final = Softmax(Q K^T / √d) V · M_chart              │
│           + A_style · (1 - M_chart)                    │
└────────────────────────────────────────────────────────┘
         │
         ▼
[Stylized Flat Raster Image (512x512 / 1024x1024)]
```

### 1.2 Spatial Cross-Attention Control Equation

To prevent the diffusion model from altering the height of bars or the angles of pie slices, ChartSpark modifies the self/cross-attention maps inside the U-Net decoder blocks:

$$A_{\text{controlled}}(Q, K, V) = \text{Softmax}\left(\frac{Q K^T}{\sqrt{d_k}}\right) V \odot M_{\text{chart}} + A_{\text{style}} \odot (\mathbf{1} - M_{\text{chart}})$$

where:
- $M_{\text{chart}} \in \{0, 1\}^{H \times W}$ is a spatial binary mask where $1$ covers data-encoding chart regions (e.g. bar rectangles, line points) and $0$ covers non-data background regions.
- $A_{\text{style}}$ is the unconstrained text-conditioned cross-attention output driven by the style prompt (e.g., *"cyberpunk neon city, 3D metallic texture"*).

### 1.3 Empirical Evaluation & Performance Metrics

ChartSpark introduces two evaluation metrics for generative chart re-rendering:

1. **Data Preservation Index (DPI):** Measures the structural correlation between bounding box heights of original chart bars $H_{\text{orig}}$ and segmented bars in the generated image $H_{\text{gen}}$:
   $$\text{DPI} = 1 - \frac{1}{N} \sum_{i=1}^N \frac{|H_{\text{orig}, i} - H_{\text{gen}, i}|}{H_{\text{orig}, i}}$$
2. **Fréchet Inception Distance (FID):** Measures visual rendering quality against professional design datasets.

```
+-----------------------------------+-------------------+-------------------+
| Architecture / Method             | Data Preservation (DPI) | FID (Visual Quality) |
+-----------------------------------+-------------------+-------------------+
| Unconstrained Stable Diffusion    | 42.1%             | 34.2              |
| ControlNet (Canny Edge Only)      | 81.5%             | 22.6              |
| ChartSpark (ControlNet + Masking) | 96.8%             | 18.4              |
| FAW SVG Vector Renderer           | 100.0% (Native)   | N/A (Vector Paths)|
+-----------------------------------+-------------------+-------------------+
```

### 1.4 Technical Failures & Infographic Limitations

- **Text Rasterization & Gibberish:** Text elements (titles, axis labels) inside the diffusion latent space undergo pixel smearing, resulting in unreadable or hallucinated glyphs.
- **Loss of Vector Scalability:** Converting clean SVG vector paths into fixed-resolution raster images ($512 \times 512$ or $1024 \times 1024$) destroys zoom scalability and prevents post-generation typography fixes.

---

## 2. Distillation for FAW

### 2.1 Transferable Primitives

1. **Isolating Generative Diffusion to Asset Layers:** Latent diffusion should be used strictly for generating non-data decorative visual assets (icons, background patterns) rather than rendering text or data charts.
2. **Theme-Based Styling Profiles:** Mapping semantic style prompts to explicit SVG color palette tokens (e.g. `TechDark`, `FinancialNavy`, `VibrantCoral`) rather than relying on generative model latent features.

### 2.2 System Mapping (`katsvg-engine` Pure-Rust Pipeline)

Completely replaces latent diffusion styling with deterministic CSS palette themes (`PaletteTheme` in `router.rs`). Renders exact chart glyphs (`ChartGlyphRenderer` in `chart.rs`) natively in pure Rust to maintain $100\%$ data preservation and vector scalability.

---

## 3. Verdict

- **Tier:** Pass
- **Criteria:** Rejects flat image baking of charts and text; FAW isolates generative assets as background/icon layers while keeping text and data paths editable in SVG.
- **Routing:** Layer isolation principles are implemented in `katsvg-engine`.
- **One-Line Reasoning:** Rejects flat image baking of charts and text; FAW isolates generative assets as background/icon layers while keeping text and data paths editable in SVG.
