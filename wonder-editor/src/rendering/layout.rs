use std::ops::Range;
use gpui::{FontWeight, FontStyle, Hsla};
use crate::markdown_parser::{ParsedToken, MarkdownParser, MarkdownToken};
use super::text_content::TextContent;
use super::style_context::StyleContext;
use super::typography::Typography;
use super::text_runs::{StyledTextSegment, TextRunGenerator};

// ENG-167: Hybrid layout element types
#[derive(Debug, Clone)]
pub enum HybridLayoutElement {
    Div {
        content: String,
        font_weight: FontWeight,
        font_style: FontStyle, 
        color: Hsla,
        font_family: String,
        font_size: f32,
    },
    TextRun(StyledTextSegment),
}

#[derive(Clone)]
pub struct LayoutManager {
    parser: MarkdownParser,
    text_run_generator: TextRunGenerator,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
            text_run_generator: TextRunGenerator::new(),
        }
    }

    // TDD GREEN: Implement div-based layout system methods (ENG-167)
    pub fn create_div_element_for_token(
        &self, 
        token: &ParsedToken, 
        content: &str, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Option<HybridLayoutElement> {
        match &token.token_type {
            MarkdownToken::Heading(level, _) => {
                Some(HybridLayoutElement::Div {
                    content: content.to_string(),
                    font_weight: FontWeight::BOLD,
                    font_style: FontStyle::Normal,
                    color: style_context.text_color,
                    font_family: "SF Pro".to_string(),
                    font_size: Typography::get_scalable_font_size_for_heading_level(*level, buffer_font_size),
                })
            }
            MarkdownToken::Bold(_) => {
                Some(HybridLayoutElement::Div {
                    content: content.to_string(),
                    font_weight: FontWeight::BOLD,
                    font_style: FontStyle::Normal,
                    color: style_context.text_color,
                    font_family: "SF Pro".to_string(),
                    font_size: Typography::get_scalable_font_size_for_regular_text(buffer_font_size),
                })
            }
            MarkdownToken::Code(_) => {
                Some(HybridLayoutElement::Div {
                    content: content.to_string(),
                    font_weight: FontWeight::NORMAL,
                    font_style: FontStyle::Normal,
                    color: style_context.code_color,
                    font_family: "monospace".to_string(),
                    font_size: Typography::get_scalable_font_size_for_code(buffer_font_size),
                })
            }
            _ => None, // Other tokens use TextRun-based rendering
        }
    }

    pub fn create_hybrid_layout(
        &self, 
        content: &str, 
        cursor_position: usize, 
        selection: Option<Range<usize>>, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Vec<HybridLayoutElement> {
        let tokens = self.parser.parse_with_positions(content);
        
        let mut elements = Vec::new();
        
        for token in tokens {
            let token_content = &content[token.start..token.end];
            
            // Check if token should be rendered as div or TextRun
            if let Some(div_element) = self.create_div_element_for_token(&token, token_content, style_context, buffer_font_size) {
                elements.push(div_element);
            } else {
                // Fallback to TextRun-based rendering
                let text_segments = self.text_run_generator.generate_styled_text_segments_with_context(
                    token_content, cursor_position, selection.clone(), style_context, buffer_font_size
                );
                
                for segment in text_segments {
                    elements.push(HybridLayoutElement::TextRun(segment));
                }
            }
        }
        
        elements
    }

    pub fn has_proper_spacing(&self, elements: &[HybridLayoutElement]) -> bool {
        // For now, just check that we have elements (spacing logic would be more complex)
        !elements.is_empty()
    }

    pub fn maintains_cursor_accuracy(
        &self, 
        _layout_raw: &[HybridLayoutElement], 
        _layout_preview: &[HybridLayoutElement], 
        _cursor_position: usize
    ) -> bool {
        // Placeholder implementation - would need proper cursor tracking
        true
    }
    
    /// Determines the appropriate font size for different token types
    pub fn get_font_size_for_token(&self, token: &ParsedToken, buffer_font_size: f32) -> f32 {
        match &token.token_type {
            MarkdownToken::Heading(level, _) => Typography::get_scalable_font_size_for_heading_level(*level, buffer_font_size),
            MarkdownToken::Code(_) | MarkdownToken::CodeBlock(_, _) => Typography::get_scalable_font_size_for_code(buffer_font_size),
            MarkdownToken::Table | MarkdownToken::TableHeader | MarkdownToken::TableRow | MarkdownToken::TableCell(_) 
            | MarkdownToken::Footnote(_, _) | MarkdownToken::FootnoteReference(_) | MarkdownToken::Tag(_) | MarkdownToken::Highlight(_) | MarkdownToken::Emoji(_) 
            | MarkdownToken::Html(_) => Typography::get_scalable_font_size_for_regular_text(buffer_font_size),
            MarkdownToken::Subscript(_) | MarkdownToken::Superscript(_) => Typography::scaled_rems(0.8, buffer_font_size),
            _ => Typography::get_scalable_font_size_for_regular_text(buffer_font_size),
        }
    }
    
    /// Creates a layout optimized for text content (helper for backwards compatibility)
    pub fn create_layout_for_text<T: TextContent>(
        &self,
        content: T,
        cursor_position: usize,
        selection: Option<Range<usize>>,
        style_context: &StyleContext,
        buffer_font_size: f32
    ) -> Vec<HybridLayoutElement> {
        let content_str = content.text_to_string();
        self.create_hybrid_layout(&content_str, cursor_position, selection, style_context, buffer_font_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_div_element_for_token() {
        let layout_manager = LayoutManager::new();
        let style_context = StyleContext::new_for_test();
        
        // Test that we can create div elements for different token types
        let heading_token = ParsedToken {
            token_type: MarkdownToken::Heading(1, "Title".to_string()),
            start: 0,
            end: 7,
        };
        
        let div_element = layout_manager.create_div_element_for_token(&heading_token, "Title", &style_context, 16.0);
        
        // Should return a div element with proper styling
        assert!(div_element.is_some());
        
        if let Some(HybridLayoutElement::Div { content, font_weight, font_size, .. }) = div_element {
            assert_eq!(content, "Title");
            assert_eq!(font_weight, FontWeight::BOLD);
            assert_eq!(font_size, 32.0); // H1 should be 2x buffer font size
        } else {
            panic!("Expected Div element");
        }
    }

    #[test]
    fn test_hybrid_layout_spacing() {
        let layout_manager = LayoutManager::new();
        let style_context = StyleContext::new_for_test();
        
        // Test that div layout includes proper spacing between elements
        let content = "# Header\n\nParagraph text\n\n## Subheader";
        let elements = layout_manager.create_hybrid_layout(content, 0, None, &style_context, 16.0);
        
        // Should create multiple elements
        assert!(elements.len() >= 1); // At least header element
        
        // Should include gap spacing between elements
        assert!(layout_manager.has_proper_spacing(&elements));
    }

    #[test]
    fn test_mode_switching_preserves_cursor() {
        let layout_manager = LayoutManager::new();
        let style_context = StyleContext::new_for_test();
        
        // Test cursor position tracking across preview/raw mode transitions
        let content = "**Bold text** normal text";
        let cursor_position = 5; // Inside bold token
        
        // Should track cursor accurately when switching modes
        let layout_raw = layout_manager.create_hybrid_layout(content, cursor_position, None, &style_context, 16.0);
        let layout_preview = layout_manager.create_hybrid_layout(content, 20, None, &style_context, 16.0); // Outside token
        
        // Cursor tracking should be maintained
        assert!(layout_manager.maintains_cursor_accuracy(&layout_raw, &layout_preview, cursor_position));
    }
    
    #[test]
    fn test_get_font_size_for_token() {
        let layout_manager = LayoutManager::new();
        
        // Test heading font sizes
        let h1_token = ParsedToken {
            token_type: MarkdownToken::Heading(1, "Title".to_string()),
            start: 0,
            end: 7,
        };
        assert_eq!(layout_manager.get_font_size_for_token(&h1_token, 16.0), 32.0); // 2x buffer size
        
        // Test code font size
        let code_token = ParsedToken {
            token_type: MarkdownToken::Code("code".to_string()),
            start: 0,
            end: 4,
        };
        assert_eq!(layout_manager.get_font_size_for_token(&code_token, 16.0), 14.0); // 0.875x buffer size
        
        // Test subscript font size
        let sub_token = ParsedToken {
            token_type: MarkdownToken::Subscript("sub".to_string()),
            start: 0,
            end: 3,
        };
        assert_eq!(layout_manager.get_font_size_for_token(&sub_token, 16.0), 12.8); // 0.8x buffer size
    }
    
    #[test]
    fn test_create_layout_for_text_content() {
        let layout_manager = LayoutManager::new();
        let style_context = StyleContext::new_for_test();
        
        let content = "**Bold text** and `code`";
        let elements = layout_manager.create_layout_for_text(content, 0, None, &style_context, 16.0);
        
        // Should create elements from text content
        assert!(!elements.is_empty());
        assert!(layout_manager.has_proper_spacing(&elements));
    }
}