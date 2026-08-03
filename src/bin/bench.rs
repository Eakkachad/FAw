//! katSVG Benchmark Harness + GOAT Gate (stable Rust, no nightly `test` crate).
//!
//! Measures intent-parse, render, and full-export latency (p50/p99) over N
//! iterations, verifies byte-determinism, and enforces GOAT-style thresholds.
//! Exit code 0 = gates pass; non-zero = a gate failed.
//!
//! ```bash
//! cargo run --release --bin bench -- 200
//! ```

use katsvg_engine::InfographicIntentRouter;
use std::time::{Duration, Instant};

const PROMPTS: [&str; 4] = [
    "Build a 4-step AI Agent Deployment Timeline in dark mode",
    "Q3 financial KPI dashboard with metrics in navy",
    "Show a bar chart: Q1: 10, Q2: 25, Q3: 15, Q4: 30 in navy banner",
    "Compare GPT-4 vs Gemini feature matrix",
];

// GOAT gate thresholds (tuned to current hardware; keep generous for CI noise)
const GATE_PARSE_P50_MS: f64 = 0.5;
const GATE_PARSE_P99_MS: f64 = 2.0;
const GATE_EXPORT_P50_MS: f64 = 30.0;
const GATE_EXPORT_P99_MS: f64 = 60.0;

fn pct(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn report(label: &str, samples: &[Duration]) {
    let mut sorted = samples.to_vec();
    sorted.sort();
    let sum: u64 = sorted.iter().map(|d| d.as_nanos() as u64).sum();
    let mean = sum as f64 / sorted.len().max(1) as f64;
    println!(
        "  {:<12} p50 {:>8.3} ms | p99 {:>8.3} ms | mean {:>8.3} ms | n={}",
        label,
        pct(&sorted, 0.50).as_secs_f64() * 1000.0,
        pct(&sorted, 0.99).as_secs_f64() * 1000.0,
        mean / 1e6,
        sorted.len()
    );
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(100);

    println!("katSVG GOAT Benchmark Harness — {} iterations\n", n);

    let router = InfographicIntentRouter::new();

    // G1: byte-determinism (same prompt -> byte-identical spec serialization)
    let a = serde_json::to_vec(&router.parse_and_route(PROMPTS[0])).unwrap();
    let b = serde_json::to_vec(&router.parse_and_route(PROMPTS[0])).unwrap();
    let deterministic = a == b;
    println!(
        "G1 determinism     : {} (same prompt -> identical spec)",
        if deterministic { "PASS" } else { "FAIL" }
    );

    // G2/G3: latency sampling across prompts
    let mut parse_samples = Vec::with_capacity(n);
    let mut export_samples = Vec::with_capacity(n);

    for _ in 0..n {
        for prompt in PROMPTS {
            let t0 = Instant::now();
            let spec = router.parse_and_route(prompt);
            parse_samples.push(t0.elapsed());

            let out_dir = std::env::temp_dir().join(format!("katsvg_bench_{}", std::process::id()));
            let t1 = Instant::now();
            let _ = katsvg_engine::ExportManager::export_all(&spec, &out_dir);
            export_samples.push(t1.elapsed());
        }
    }

    println!("G2 intent parsing  :");
    report("parse", &parse_samples);
    println!("G3 export (4 files):");
    report("export", &export_samples);

    let parse_p50 = pct(&parse_samples.clone(), 0.50).as_secs_f64() * 1000.0;
    let parse_p99 = pct(&parse_samples.clone(), 0.99).as_secs_f64() * 1000.0;
    let export_p50 = pct(&export_samples.clone(), 0.50).as_secs_f64() * 1000.0;
    let export_p99 = pct(&export_samples.clone(), 0.99).as_secs_f64() * 1000.0;

    let gates = vec![
        ("parse p50 < 0.5 ms", parse_p50 < GATE_PARSE_P50_MS),
        ("parse p99 < 2.0 ms", parse_p99 < GATE_PARSE_P99_MS),
        ("export p50 < 30 ms", export_p50 < GATE_EXPORT_P50_MS),
        ("export p99 < 60 ms", export_p99 < GATE_EXPORT_P99_MS),
        ("determinism", deterministic),
    ];

    println!("\nGOAT gate results:");
    let mut all_pass = true;
    for (name, ok) in &gates {
        println!("  [{:>4}] {}", if *ok { "PASS" } else { "FAIL" }, name);
        all_pass &= ok;
    }

    println!(
        "\n{}",
        if all_pass {
            "ALL GATES PASS"
        } else {
            "GATE FAILURE"
        }
    );
    std::process::exit(if all_pass { 0 } else { 1 });
}
