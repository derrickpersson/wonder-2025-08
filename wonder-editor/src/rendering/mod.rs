// Rendering module - Refactored from monolithic hybrid_renderer.rs
// This module organizes rendering functionality into focused, single-responsibility sub-modules

pub mod text_content;
pub mod style_context;
pub mod typography;
pub mod token_mode;
pub mod coordinate_mapping;
pub mod text_runs;
pub mod layout;
// Re-export main types for convenience
pub use text_content::TextContent;
pub use style_context::StyleContext;
pub use typography::HeadingTypographyStyle;
pub use token_mode::TokenRenderMode;
pub use coordinate_mapping::{CoordinateMap, CoordinateMapper};
pub use text_runs::{StyledTextSegment, TextRunGenerator};
pub use layout::{HybridLayoutElement, LayoutManager};
pub use typography::Typography;

// TODO: Add integration tests for the rendering module