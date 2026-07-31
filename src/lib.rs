//! katSVG Engine — High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Crate

pub mod chart;
pub mod router;
pub mod exporter;

pub use chart::{ChartColors, ChartGlyphRenderer};
pub use exporter::{ExportManager, PDFVectorExporter, PPTXPresentationExporter, PNGRasterExporter};
pub use router::{
    ChartSpec, ChartType, InfographicConstraintPruner, InfographicIntentRouter,
    InfographicLayoutSpec, LayoutConstraints, LayoutDef, RegionDef, SVGVectorRenderer, load_corpus,
};
