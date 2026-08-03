//! katSVG Engine — High-Speed, Lightweight, Zero-Hallucination SVG Infographic Generator Crate

pub mod chart;
pub mod chart_pdf;
pub mod chart_pptx;
pub mod chart_raster;
pub mod compositor;
pub mod data_binding;
pub mod exporter;
pub mod font;
pub mod icon;
pub mod icon_paths;
pub mod icon_raster;
pub mod palette;
pub mod pdf_font;
pub mod retrieval;
pub mod router;
pub mod text;

pub use chart::{ChartColors, ChartGlyphRenderer};
pub use data_binding::{BoundData, parse_data};
pub use exporter::{ExportManager, PDFVectorExporter, PNGRasterExporter, PPTXPresentationExporter};
pub use font::{font_stack, font_style_block, has_non_ascii};
pub use icon::IconRenderer;
pub use palette::{ContrastPair, PaletteColors, PaletteEntry, PaletteRegistry, PaletteRoles};
pub use retrieval::{
    EmbeddingRetriever, RetrievalPipeline, RetrievedLayout, TagRetriever, default_retriever,
};
pub use router::{
    ChartSpec, ChartType, InfographicConstraintPruner, InfographicIntentRouter,
    InfographicLayoutSpec, LayoutConstraints, LayoutDef, RegionDef, SVGVectorRenderer, load_corpus,
};
pub use text::TextRenderer;
