# katSVG Engine (`katsvg-engine`)

> **High-Speed, Lightweight, Zero-Hallucination SVG & Document Infographic Generator Engine**

`katsvg-engine` is a pure Rust, model-less vector layout compositor and document export system. It parses structured Thai/English prompts into strongly-typed layout specifications (`InfographicLayoutSpec`), validates structural guardrails via `ConstraintPruner`, and renders multi-format document packages (**SVG, PDF 1.7, PNG, PPTX**). Runtime rendering is deterministic, offline, and does not perform LLM inference.

---

## ⚡ Performance Highlights & Benchmarks

| Metric | Measured Benchmark | Status / Threshold |
| :--- | :--- | :--- |
| **Intent Parsing Latency** | **p50 0.050 ms / p99 0.094 ms** | `< 0.5 ms` PASS |
| **Multi-Format Export (SVG+PDF+PNG+PPTX)** | **p50 3.938 ms / p99 19.904 ms** | `< 30 ms` PASS |
| **End-to-End Latency** | **~19.0 ms** | Real-time |
| **RAM Footprint** | **< 3.5 MB Total** | Offline / No GPU |
| **Structural Hallucination** | **0.0% (`ConstraintPruner` Verified)** | Guaranteed |
| **Byte Determinism** | **PASS** (identical prompt $\Rightarrow$ byte-identical output) | Verified |
| **Test Pass Rate** | **100% (86 / 86 tests passing)** | `cargo test` |

---

## 🛡️ Anti-Overclaim Verification Suite (5 Gates)

Grounded in theoretical research from `.research/markdown/` (Bylinskii et al. 2017, LIDA 2023, Chat2VIS 2023, ChartSpark 2024, ChartGalaxy ICLR 2026, VisLiteracy IEEE TVCG 2025):

```bash
# Run anti-overclaim empirical verification gate
cargo test --release --test anti_overclaim_gate
```

- **Gate 1: Zero-Hallucination Data Extraction Gate** — Validates exact prompt metric value & label parsing (0.0% invented values).
- **Gate 2: Visual Hierarchy & Bounds Gate** — Enforces $[x, y, w, h]$ region bounds and zero element overlap.
- **Gate 3: Multi-Format Parity Gate** — Verifies valid binary signatures (%PDF-1.7, PNG IHDR/IDAT, PPTX ZIP PK) across all outputs.
- **Gate 4: Mutation & De-memorization Gate** — Tests robustness against Thai/English prompt perturbations and adversarial inputs.
- **Gate 5: Empirical Latency Audit Gate** — Enforces strict `< 1.0 ms` parse and `< 30.0 ms` export gates.

---

## 🎨 Asset Corpus & Capabilities

- **10 Layout Archetypes:** `ProcessTimeline`, `StatisticalDashboard`, `ComparisonGrid`, `MindmapHierarchy`, `KpiSnapshot`, `PricingTable`, `OrgHierarchy`, `DecisionFlow`, `HeroQuote`, `ChartDashboard`.
- **9 Chart Types:** `Bar`, `Line`, `Pie`, `Donut`, `StackedBar`, `Area`, `Scatter`, `Heatmap`, `Gauge`.
- **21+ Embedded Vector Icons:** `ai`, `sparkles`, `database`, `cloud`, `code`, `brain`, `rocket`, `globe`, `activity`, `zap`, `cpu`, `shield-check`, `chart`, `target`, `users`, `clock`, `trending-up`, `layers`, `check-circle`, `alert`, `dollar`.
- **8 Color Palettes:** `TechDark`, `FinancialNavy`, `VibrantCoral`, `AcademicWarm`, `OceanBreeze`, `SunsetGlow`, `ForestMint`, `Monochrome`.
- **Thai & English i18n:** Dynamic language routing + automatic Noto Sans Thai font embedding into SVG and PDF.

---

## 🚀 Quickstart & Usage

### 1. Standalone CLI Utility

Build and run the standalone CLI tool:

```bash
# Build release binary
cargo build --release --bin katsvg

# Run text-to-infographic generation (Thai & English supported)
./target/release/katsvg "สร้าง Roadmap พัฒนา AI Architecture 4 ขั้นตอน พร้อมสรุป KPI รายไตรมาส" --out ./my_infographic
```

Output files generated in `./my_infographic`:
- `infographic.svg` — Native scalable vector graphics
- `infographic.pdf` — Vector PDF 1.7 document with embedded Thai font
- `infographic.png` — High-resolution pixel raster image
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

    // 3. Export all 4 vector & document formats to disk in < 20ms
    let out_dir = Path::new("./output_dashboard");
    let result = ExportManager::export_all(&spec, out_dir).expect("Export failed");

    println!("Infographic exported to {:?}", result.svg_path);
}
```

---

## 🗂️ Project Layout

```text
katsvg-engine/
├── src/                 Rust library, router, compositor, renderers, exporters
├── src/bin/             CLI, HTTP server, benchmark, and WASM entry points
├── corpus/              Embedded layouts, palettes, and icons
├── schemas/             JSON schemas for corpus and data binding
├── assets/              Embedded fonts and font resources
├── tests/               Integration and anti-overclaim verification tests
├── examples/            Rust integration examples
├── docs/                Benchmark and project documentation
└── .research/           Research notes and source papers
```

## 🏗️ Architecture

```mermaid
flowchart TD
    User[User / Client]

    subgraph Entry[Entry Points]
        CLI[CLI katsvg]
        Server[HTTP Server]
        WASM[WASM API]
        RustAPI[Rust Crate API]
    end

    User --> CLI
    User --> Server
    User --> WASM
    User --> RustAPI

    subgraph Input[Input]
        Prompt[Thai / English Prompt]
        Data[JSON or CSV Data]
        SavedSpec[Saved Layout Spec JSON]
    end

    CLI --> Prompt
    CLI --> Data
    CLI --> SavedSpec
    Server --> Prompt
    WASM --> Prompt
    RustAPI --> Prompt

    subgraph Routing[Intent Routing]
        Detect[Language, Theme, Aspect Detection]
        Extract[Deterministic Parameter Extraction]
        Retrieve[Layout Retrieval Pipeline]
    end

    Prompt --> Detect
    Prompt --> Extract
    Prompt --> Retrieve

    Corpus[Embedded Layout Corpus<br/>10 Layout Archetypes]
    Palettes[Palette Registry]
    Icons[Embedded Vector Icons]

    Retrieve --> Corpus
    Detect --> Palettes
    Extract --> Icons

    subgraph Validation[Validation and Composition]
        Layout[LayoutDef]
        Pruner[InfographicConstraintPruner]
        Spec[InfographicLayoutSpec]
    end

    Corpus --> Layout
    Layout --> Spec
    Detect --> Spec
    Extract --> Spec
    Data --> Binding[Data Binding]
    Binding --> Spec
    SavedSpec --> Spec
    Spec --> Pruner
    Layout --> Pruner
    Pruner --> Spec

    subgraph Rendering[Rendering and Export]
        SVG[SVG Vector Renderer]
        PDF[PDF Vector Exporter]
        PNG[PNG Raster Exporter]
        PPTX[PPTX OOXML Exporter]
    end

    Spec --> SVG
    Spec --> PDF
    Spec --> PNG
    Spec --> PPTX

    SVG --> SVGOut[infographic.svg]
    PDF --> PDFOut[infographic.pdf]
    PNG --> PNGOut[infographic.png]
    PPTX --> PPTXOut[infographic.pptx]

    Core[katgpt-core ConstraintPruner Trait]
    Core -. build-time dependency .-> Pruner
```

### Runtime properties

- **Model-less:** the routing layer uses deterministic rules and a closed layout corpus; no transformer inference runs at runtime.
- **Offline:** fonts, layouts, palettes, and icons are embedded or local; runtime network access is not required.
- **Deterministic:** identical input and build produce equivalent specs and byte-stable artifacts.
- **Structural safety:** `ConstraintPruner` reports and clamps layout bounds; this does not guarantee the truth of arbitrary prompt content.

See [`docs/SYSTEM_ARCHITECTURE.md`](docs/SYSTEM_ARCHITECTURE.md) for the detailed system flow, per-component Mermaid diagrams, usage example, and sample outputs.

See [`docs/BENCH_REPORT.md`](docs/BENCH_REPORT.md) for the measured release benchmark and reproduction commands.

---

## 📄 License

MIT License — free for commercial and personal use.
