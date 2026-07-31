//! katSVG Engine — High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Crate

pub mod router;
pub mod exporter;

pub use exporter::{ExportManager, PDFVectorExporter, PPTXPresentationExporter, PNGRasterExporter};
pub use router::{
    InfographicConstraintPruner, InfographicIntentRouter, InfographicLayoutSpec, LayoutConstraints,
    LayoutDef, RegionDef, SVGVectorRenderer, load_corpus,
};
