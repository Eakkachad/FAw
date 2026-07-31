# katSVG Engine (`katsvg-engine`)

> **High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Engine**

`katsvg-engine` is a pure Rust neuro-symbolic vector layout compositor and document export system. It parses text prompts into strongly-typed layout specifications (`InfographicLayoutSpec`) in **< 0.1 ms**, validates structural guardrails via `ConstraintPruner` (**0.0% Hallucination**), and renders native vector graphics and multi-format document packages (**SVG, PDF 1.7, PNG, PPTX**) in **< 30 ms**.

---

## ⚡ Performance Highlights

| Metric | Measured Benchmark |
| :--- | :--- |
| **Intent Parsing Latency** | **p50 0.004 ms / p99 0.011 ms** (GOAT harness, `cargo run --release --bin bench`) |
| **SVG Vector Render Time** | **0.022 - 0.045 ms** |
| **Multi-Format Export (SVG+PDF+PNG+PPTX)** | **p50 2.85 ms / p99 5.35 ms** (GOAT harness) |
| **RAM Footprint** | **< 3.2 MB Total** |
| **Structural Hallucination** | **0.0% (`ConstraintPruner` Verified)** |
| **Byte Determinism** | **PASS** (identical prompt ⇒ identical spec/SVG/PNG/PPTX/PDF) |

---

## 🚀 Quickstart & Usage

### 1. CLI Command-Line Utility

Build and run the standalone CLI tool:

```bash
# Build release binary
cargo build --release --bin katsvg

# Run text-to-infographic generation
./target/release/katsvg "Build a 4-step AI Agent Deployment Timeline in dark mode" --out ./my_infographic
```

Output files generated in `./my_infographic`:
- `infographic.svg` — Native scalable vector graphics
- `infographic.pdf` — Compliant vector PDF 1.7 document
- `infographic.png` — Pixel map raster image
- `infographic.pptx` — Editable OpenXML PowerPoint slide package

---

### 2. Rust Crate Integration

Add `katsvg-engine` to your `Cargo.toml`:

```rust
use katsvg_engine::{InfographicIntentRouter, ExportManager};
use std::path::Path;

fn main() {
    // 1. Initialize intent router
    let router = InfographicIntentRouter::new();

    // 2. Parse text prompt to structured spec in < 0.1ms
    let prompt = "Generate a Q3 Financial Revenue Dashboard poster in navy blue";
    let spec = router.parse_and_route(prompt);

    // 3. Export all 4 vector & document formats to disk in < 30ms
    let out_dir = Path::new("./output_dashboard");
    let result = ExportManager::export_all(&spec, out_dir).expect("Export failed");

    println!("Infographic exported to {:?}", result.svg_path);
}
```

---

## 🏗️ Architecture

```
User Prompt Input
  │
  ▼
InfographicIntentRouter (corpus-driven, model-less)
  │  retrieve LayoutDef from embedded layout corpus
  │  deterministic parameter extraction (step count, metrics, title)
  ▼
InfographicConstraintPruner (katgpt-core ConstraintPruner trait
  │                        + per-layout bounds, violations()/clamp())
  ▼
InfographicLayoutSpec (Strongly-Typed Latent MCP Target)
  │
  ├── SVGVectorRenderer     ──> infographic.svg
  ├── PDFVectorExporter     ──> infographic.pdf
  ├── PNGRasterExporter     ──> infographic.png
  └── PPTXPresentationExporter ──> infographic.pptx
```

> **katGPT integration:** the engine reuses `katgpt-core`'s `ConstraintPruner`
> trait for structural validity (0.0% hallucination) and follows the katgpt-rs
> modelless-first mandate — no transformer inference at runtime. The routing
> layer is corpus-driven, not an LLM.

---

## 📄 License

MIT License — free for commercial and personal use.
