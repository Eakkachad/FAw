//! katSVG Engine — High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Crate

pub mod chart;
pub mod font;
pub mod icon;
pub mod palette;
pub mod retrieval;
pub mod router;
pub mod text;
pub mod exporter;

pub use chart::{ChartColors, ChartGlyphRenderer};
pub use exporter::{ExportManager, PDFVectorExporter, PPTXPresentationExporter, PNGRasterExporter};
pub use font::{FONT_STACK, font_style_block};
pub use icon::IconRenderer;
pub use palette::{ContrastPair, PaletteColors, PaletteEntry, PaletteRegistry, PaletteRoles};
pub use retrieval::{EmbeddingRetriever, RetrievedLayout, RetrievalPipeline, TagRetriever, default_retriever};
pub use text::TextRenderer;
pub use router::{
    ChartSpec, ChartType, InfographicConstraintPruner, InfographicIntentRouter,
    InfographicLayoutSpec, LayoutConstraints, LayoutDef, RegionDef, SVGVectorRenderer, load_corpus,
};
