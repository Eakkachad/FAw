//! katSVG Engine — High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Crate

pub mod router;
pub mod exporter;

pub use exporter::ExportManager;
pub use router::{InfographicConstraintPruner, InfographicIntentRouter, InfographicLayoutSpec, SVGVectorRenderer};
