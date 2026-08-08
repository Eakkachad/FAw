# BENCH_REPORT.md — katSVG Performance & Baseline (D8)

> Measured 2026-07-31 on Apple Silicon (macOS), release build. GOAT harness:
> `cargo run --release --bin bench -- 50` (200 samples across 4 prompt types).

## 1. Latency (p50 / p99, release)

| Stage | p50 | p99 | Gate |
|---|---|---|---|
| Intent parse (prompt → validated spec) | **0.050 ms** | 0.078 ms | < 0.5 ms ✓ |
| Multi-format export (SVG+PDF+PNG+PPTX) | **3.738 ms** | 7.528 ms | < 30 ms ✓ |
| End-to-end (CLI) | **~10.5 ms** | — | — |

All GOAT gates PASS. Byte-determinism verified (identical prompt → identical spec).

## 2. Output artifact sizes (Q3 KPI dashboard prompt)

| Format | Size |
|---|---|
| `infographic.svg` | 31.2 KB (includes embedded Inter font) |
| `infographic.png` | 27.7 KB (800×1131, text rasterized) |
| `infographic.pptx` | 11.8 KB (valid OOXML) |
| `infographic.pdf` | 1.3 KB (PDF 1.7) |
| Binary `katsvg` | 1.58 MB (2 embedded OFL fonts + chart engines) |

## 3. Resource footprint

| Metric | Value |
|---|---|
| RAM | < 5 MB (embedded corpus, no network) |
| CPU | single-core scalar; no GPU |
| Network | **zero** at runtime (fonts/corpus embedded) |

## 4. Comparison vs LLM-based generation (context)

| Metric | katSVG | Typical LLM (GPT/DALL·E) text-to-image |
|---|---|---|
| Latency | ~10 ms | 2–15 s |
| Cost/output | $0 (local) | $0.02–$0.12 API |
| Determinism | byte-identical | stochastic |
| Hallucination | 0.0% (pruner) | possible (wrong values/layout) |
| Offline | yes | no |

> These are *contextual* figures for LLM tools; no live LLM benchmark was run
> this session. Treat the right column as approximate industry ranges.

## 5. Regression notes (Phase 3)

- Chart parity added PNG/PPTX/PDF (D1–D3): export p50 rose from ~2.9 ms → **3.7 ms** (chart rasterization + PDF/PPTX vectors) — still 8× under gate.
- PNG text rasterization (P5) added earlier: included in the 3.7 ms figure.
- Corpus grew 6 → 10 layouts (D7): parse latency unchanged (0.050 ms).

## 6. Reproduction

```bash
cargo run --release --bin bench -- 50        # latency + GOAT gates
./target/release/katsvg "prompt" --out /tmp/v # artifacts
file /tmp/v/*                                  # format validation
```
