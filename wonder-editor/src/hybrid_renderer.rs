use std::ops::Range;
use crate::markdown_parser::{ParsedToken, MarkdownParser};
use crate::rendering::{
    TextContent, StyleContext, TokenRenderMode, Typography,
    StyledTextSegment, HybridLayoutElement, HeadingTypographyStyle,
    CoordinateMapper, TextRunGenerator, LayoutManager,
};
use gpui::{TextRun, rgb, Font, FontFeatures, FontWeight, FontStyle, Hsla};

#[derive(Clone)]
pub struct HybridTextRenderer {
    parser: MarkdownParser,
    coordinate_mapper: CoordinateMapper,
    text_run_generator: TextRunGenerator,
    layout_manager: LayoutManager,
}

impl HybridTextRenderer {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
            coordinate_mapper: CoordinateMapper::new(),
            text_run_generator: TextRunGenerator::new(),
            layout_manager: LayoutManager::new(),
        }
    }
    
    /// Returns the tokens in the given markdown content with their render modes
    pub fn render_document(&self, content: &str, cursor_position: usize, selection: Option<Range<usize>>) -> Vec<(ParsedToken, TokenRenderMode)> {
        let tokens = self.parser.parse_with_positions(content);
        tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect()
    }
    
    // Font size methods - delegated to Typography module
    pub fn get_font_size_for_heading_level(&self, level: u32) -> f32 {
        Typography::get_font_size_for_heading_level(level)
    }

    pub fn get_font_size_for_code(&self) -> f32 {
        Typography::get_font_size_for_code()
    }

    pub fn get_font_size_for_regular_text(&self) -> f32 {
        Typography::get_font_size_for_regular_text()
    }
    
    // Scalable font size methods - delegated to Typography module
    pub fn get_scalable_font_size_for_heading_level(&self, level: u32, buffer_font_size: f32) -> f32 {
        Typography::get_scalable_font_size_for_heading_level(level, buffer_font_size)
    }

    pub fn get_scalable_font_size_for_code(&self, buffer_font_size: f32) -> f32 {
        Typography::get_scalable_font_size_for_code(buffer_font_size)
    }

    pub fn get_scalable_font_size_for_regular_text(&self, buffer_font_size: f32) -> f32 {
        Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
    }
    
    pub fn scaled_rems(&self, rem_factor: f32, buffer_font_size: f32) -> f32 {
        Typography::scaled_rems(rem_factor, buffer_font_size)
    }
    
    pub fn get_line_height_for_font_size(&self, font_size: f32) -> f32 {
        Typography::get_line_height_for_font_size(font_size)
    }
    
    // Typography style methods - delegated to Typography module
    pub fn get_heading_typography_style(
        &self, 
        token: &ParsedToken, 
        is_preview_mode: bool, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> HeadingTypographyStyle {
        Typography::get_heading_typography_style(token, is_preview_mode, style_context, buffer_font_size)
    }
    
    // Layout methods - delegated to LayoutManager
    pub fn create_div_element_for_token(
        &self, 
        token: &ParsedToken, 
        content: &str, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Option<HybridLayoutElement> {
        self.layout_manager.create_div_element_for_token(token, content, style_context, buffer_font_size)
    }

    pub fn create_hybrid_layout(
        &self, 
        content: &str, 
        cursor_position: usize, 
        selection: Option<Range<usize>>, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Vec<HybridLayoutElement> {
        self.layout_manager.create_hybrid_layout(content, cursor_position, selection, style_context, buffer_font_size)
    }

    pub fn has_proper_spacing(&self, elements: &[HybridLayoutElement]) -> bool {
        self.layout_manager.has_proper_spacing(elements)
    }

    pub fn maintains_cursor_accuracy(
        &self, 
        layout_raw: &[HybridLayoutElement], 
        layout_preview: &[HybridLayoutElement], 
        cursor_position: usize
    ) -> bool {
        self.layout_manager.maintains_cursor_accuracy(layout_raw, layout_preview, cursor_position)
    }
    
    pub fn get_font_size_for_token(&self, token: &ParsedToken, buffer_font_size: f32) -> f32 {
        self.layout_manager.get_font_size_for_token(token, buffer_font_size)
    }
    
    // Text run generation methods - delegated to TextRunGenerator
    pub fn generate_styled_text_segments_with_context<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Vec<StyledTextSegment> {
        self.text_run_generator.generate_styled_text_segments_with_context(
            content, cursor_position, selection, style_context, buffer_font_size
        )
    }

    pub fn generate_styled_text_segments<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> Vec<StyledTextSegment> {
        self.text_run_generator.generate_styled_text_segments(content, cursor_position, selection)
    }
    
    pub fn generate_mixed_text_runs<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> Vec<TextRun> {
        self.text_run_generator.generate_mixed_text_runs(content, cursor_position, selection)
    }
    
    /// Returns the transformed content string that should be displayed
    pub fn get_display_content<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> String {
        self.text_run_generator.get_display_content(content, cursor_position, selection)
    }
    
    // Coordinate mapping methods - delegated to CoordinateMapper
    pub fn map_cursor_position<T: TextContent>(
        &self, 
        content: T, 
        original_cursor_pos: usize, 
        selection: Option<Range<usize>>
    ) -> usize {
        self.coordinate_mapper.map_cursor_position(content, original_cursor_pos, selection)
    }
    
    pub fn map_display_position_to_original<T: TextContent>(
        &self, 
        content: T, 
        display_position: usize, 
        cursor_position: usize, 
        selection: Option<(usize, usize)>
    ) -> usize {
        self.coordinate_mapper.map_display_position_to_original(
            content, display_position, cursor_position, selection
        )
    }
    
    // Legacy methods for backwards compatibility (kept for fallback/testing)
    pub fn get_token_render_mode(&self, token: &ParsedToken, cursor_position: usize, selection: Option<Range<usize>>) -> TokenRenderMode {
        TokenRenderMode::get_for_token(token, cursor_position, selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::MarkdownToken;
    
    #[test]
    fn test_hybrid_renderer_creation() {
        let renderer = HybridTextRenderer::new();
        
        // Should create renderer with all component managers
        let content = "Hello **world**!";
        let segments = renderer.generate_styled_text_segments(content, 0, None);
        assert!(!segments.is_empty());
    }
    
    #[test]
    fn test_render_document() {
        let renderer = HybridTextRenderer::new();
        let content = "Hello **world** and `code`!";
        
        let token_modes = renderer.render_document(content, 0, None);
        
        // Should return tokens with their render modes
        assert!(!token_modes.is_empty());
        
        // At cursor position 0, tokens not containing cursor should be in Preview mode
        // Tokens that start at position 0 (if any) would be in Raw mode
        let has_preview_tokens = token_modes.iter().any(|(_, mode)| *mode == TokenRenderMode::Preview);
        assert!(has_preview_tokens, "Should have at least one Preview mode token");
        
        // Test specific cursor inside token behavior  
        let token_modes_inside = renderer.render_document(content, 8, None); // Inside **world**
        let bold_token = token_modes_inside.iter()
            .find(|(token, _)| matches!(token.token_type, MarkdownToken::Bold(_)));
        if let Some((_, mode)) = bold_token {
            assert_eq!(*mode, TokenRenderMode::Raw, "Token containing cursor should be Raw");
        }
    }
    
    #[test]
    fn test_font_size_delegation() {
        let renderer = HybridTextRenderer::new();
        
        // Test that font size methods delegate correctly to Typography
        assert_eq!(renderer.get_font_size_for_heading_level(1), 24.0);
        assert_eq!(renderer.get_font_size_for_code(), 14.0);
        assert_eq!(renderer.get_font_size_for_regular_text(), 16.0);
        
        // Test scalable methods
        assert_eq!(renderer.get_scalable_font_size_for_heading_level(1, 16.0), 32.0);
        assert_eq!(renderer.get_scalable_font_size_for_code(16.0), 14.0);
        assert_eq!(renderer.get_scalable_font_size_for_regular_text(16.0), 16.0);
    }
    
    #[test]
    fn test_coordinate_mapping_delegation() {
        let renderer = HybridTextRenderer::new();
        let content = "Hello **world**!";
        
        // Test cursor mapping
        let display_pos = renderer.map_cursor_position(content, 0, None);
        assert_eq!(display_pos, 0);
        
        // Test reverse mapping
        let original_pos = renderer.map_display_position_to_original(content, 0, 0, None);
        assert_eq!(original_pos, 0);
    }
    
    #[test]
    fn test_layout_delegation() {
        let renderer = HybridTextRenderer::new();
        let style_context = StyleContext::new_for_test();
        
        // Test layout creation
        let content = "# Header\n\nParagraph text";
        let elements = renderer.create_hybrid_layout(content, 0, None, &style_context, 16.0);
        
        assert!(!elements.is_empty());
        assert!(renderer.has_proper_spacing(&elements));
    }
    
    #[test]
    fn test_backwards_compatibility() {
        let renderer = HybridTextRenderer::new();
        
        // Test that legacy methods still work
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("text".to_string()),
            start: 0,
            end: 6,
        };
        
        let mode = renderer.get_token_render_mode(&token, 3, None); // Inside token
        assert_eq!(mode, TokenRenderMode::Raw);
        
        let mode_outside = renderer.get_token_render_mode(&token, 10, None); // Outside token
        assert_eq!(mode_outside, TokenRenderMode::Preview);
    }
}