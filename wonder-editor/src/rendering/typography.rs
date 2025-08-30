use gpui::{FontWeight, FontStyle, Hsla};
use crate::markdown_parser::{ParsedToken, MarkdownToken};
use super::style_context::StyleContext;

// ENG-168: Typography style structure for mode-aware styling  
#[derive(Debug, Clone, PartialEq)]
pub struct HeadingTypographyStyle {
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Hsla,
    pub font_family: String,
}

pub struct Typography;

impl Typography {
    // Legacy fixed font size methods (kept for compatibility)
    pub fn get_font_size_for_heading_level(level: u32) -> f32 {
        match level {
            1 => 24.0,
            2 => 20.0,
            3 => 18.0,
            4 => 17.0,
            5 => 16.0,
            6 => 15.0,
            _ => 16.0,
        }
    }

    pub fn get_font_size_for_code() -> f32 {
        14.0
    }

    pub fn get_font_size_for_regular_text() -> f32 {
        16.0
    }

    // ENG-166: Scalable font sizing methods
    pub fn scaled_rems(rem_factor: f32, buffer_font_size: f32) -> f32 {
        rem_factor * buffer_font_size
    }

    pub fn get_scalable_font_size_for_heading_level(level: u32, buffer_font_size: f32) -> f32 {
        // ENG-168: Implement typography hierarchy
        let rem_factor = match level {
            1 => 2.0,    // H1 - 2x buffer font size
            2 => 1.5,    // H2 - 1.5x buffer font size
            3 => 1.25,   // H3 - 1.25x buffer font size
            4 => 1.0,    // H4 - 1x buffer font size
            5 => 0.875,  // H5 - 0.875x buffer font size
            6 => 0.85,   // H6 - 0.85x buffer font size
            _ => 1.0,    // Default for invalid levels
        };
        Self::scaled_rems(rem_factor, buffer_font_size)
    }

    pub fn get_scalable_font_size_for_code(buffer_font_size: f32) -> f32 {
        Self::scaled_rems(0.875, buffer_font_size) // Code is 0.875x buffer font size
    }

    pub fn get_scalable_font_size_for_regular_text(buffer_font_size: f32) -> f32 {
        Self::scaled_rems(1.0, buffer_font_size) // 1x buffer font size
    }
    
    // ENG-168: Add line height calculation (1.25x font size)
    pub fn get_line_height_for_font_size(font_size: f32) -> f32 {
        font_size * 1.25
    }
    
    // ENG-168: Typography style data structure for mode-aware styling
    pub fn get_heading_typography_style(
        token: &ParsedToken, 
        is_preview_mode: bool, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> HeadingTypographyStyle {
        if let MarkdownToken::Heading(level, _) = &token.token_type {
            if is_preview_mode {
                // Preview mode: styled headings with proper hierarchy
                let font_size = Self::get_scalable_font_size_for_heading_level(*level, buffer_font_size);
                HeadingTypographyStyle {
                    font_weight: FontWeight::BOLD,
                    font_style: FontStyle::Normal,
                    font_size,
                    line_height: Self::get_line_height_for_font_size(font_size),
                    color: style_context.text_color,
                    font_family: "SF Pro".to_string(),
                }
            } else {
                // Raw mode: show markdown syntax with normal styling
                HeadingTypographyStyle {
                    font_weight: FontWeight::NORMAL,
                    font_style: FontStyle::Normal, 
                    font_size: buffer_font_size,
                    line_height: Self::get_line_height_for_font_size(buffer_font_size),
                    color: style_context.text_color,
                    font_family: "SF Pro".to_string(),
                }
            }
        } else {
            // Default styling for non-heading tokens
            HeadingTypographyStyle {
                font_weight: FontWeight::NORMAL,
                font_style: FontStyle::Normal,
                font_size: buffer_font_size,
                line_height: Self::get_line_height_for_font_size(buffer_font_size),
                color: style_context.text_color,
                font_family: "SF Pro".to_string(),
            }
        }
    }

    pub fn get_font_size_for_token(token: &ParsedToken, buffer_font_size: f32) -> f32 {
        match &token.token_type {
            MarkdownToken::Heading(level, _) => Self::get_scalable_font_size_for_heading_level(*level, buffer_font_size),
            MarkdownToken::Code(_) | MarkdownToken::CodeBlock(_, _) => Self::get_scalable_font_size_for_code(buffer_font_size),
            _ => Self::get_scalable_font_size_for_regular_text(buffer_font_size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scaled_rems_calculation() {
        // Test rem scaling calculation
        assert_eq!(Typography::scaled_rems(1.0, 16.0), 16.0);
        assert_eq!(Typography::scaled_rems(1.5, 16.0), 24.0);
        assert_eq!(Typography::scaled_rems(0.875, 16.0), 14.0);
        
        // Test with different buffer font sizes
        assert_eq!(Typography::scaled_rems(2.0, 20.0), 40.0);
        assert_eq!(Typography::scaled_rems(1.25, 12.0), 15.0);
        
        // Test edge cases
        assert_eq!(Typography::scaled_rems(0.0, 16.0), 0.0);
        assert_eq!(Typography::scaled_rems(1.0, 0.0), 0.0);
    }
    
    #[test] 
    fn test_heading_sizes_scale_with_buffer_font() {
        // Test H1 scaling (should be 2x buffer font size with new typography hierarchy)
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(1, 16.0), 32.0);
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(1, 20.0), 40.0);
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(1, 12.0), 24.0);
        
        // Test H2 scaling (should be 1.5x buffer font size with new typography hierarchy) 
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(2, 16.0), 24.0);
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(2, 20.0), 30.0);
        
        // Test H5 (should be 0.875x buffer font size with new typography hierarchy)
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(5, 16.0), 14.0);
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(5, 20.0), 17.5);
    }
    
    #[test]
    fn test_code_font_scales_with_buffer_font() {
        // Code should be 0.875x buffer font size
        assert_eq!(Typography::get_scalable_font_size_for_code(16.0), 14.0);
        assert_eq!(Typography::get_scalable_font_size_for_code(20.0), 17.5);
        assert_eq!(Typography::get_scalable_font_size_for_code(12.0), 10.5);
    }
    
    #[test]
    fn test_typography_hierarchy_ratios() {
        let buffer_font_size = 16.0;
        
        // Test heading size ratios: H1: 2x, H2: 1.5x, H3: 1.25x, H4: 1x, H5: 0.875x, H6: 0.85x
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(1, buffer_font_size), 32.0, "H1 should be 2x buffer font size");
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(2, buffer_font_size), 24.0, "H2 should be 1.5x buffer font size");
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(3, buffer_font_size), 20.0, "H3 should be 1.25x buffer font size");
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(4, buffer_font_size), 16.0, "H4 should be 1x buffer font size");
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(5, buffer_font_size), 14.0, "H5 should be 0.875x buffer font size");
        assert_eq!(Typography::get_scalable_font_size_for_heading_level(6, buffer_font_size), 13.6, "H6 should be 0.85x buffer font size");
    }
    
    #[test]
    fn test_line_height_calculation() {
        // Test line height is 1.25x font size
        assert_eq!(Typography::get_line_height_for_font_size(16.0), 20.0, "Line height should be 1.25x font size");
        assert_eq!(Typography::get_line_height_for_font_size(24.0), 30.0, "Line height should be 1.25x font size for larger fonts");
        assert_eq!(Typography::get_line_height_for_font_size(32.0), 40.0, "Line height should be 1.25x font size for H1");
    }
    
    #[test]
    fn test_heading_font_weights_and_styles() {
        let style_context = StyleContext::new_for_test();
        
        // Test that headings get proper font weights (bold) in both modes
        let h1_token = ParsedToken {
            token_type: MarkdownToken::Heading(1, "Title".to_string()),
            start: 0,
            end: 7,
        };
        
        // Test preview mode styling
        let preview_styling = Typography::get_heading_typography_style(&h1_token, true, &style_context, 16.0);
        assert_eq!(preview_styling.font_weight, FontWeight::BOLD);
        assert_eq!(preview_styling.font_size, 32.0); // 2x buffer font size
        assert_eq!(preview_styling.line_height, 40.0); // 1.25x font size
        
        // Test raw mode styling  
        let raw_styling = Typography::get_heading_typography_style(&h1_token, false, &style_context, 16.0);
        assert_eq!(raw_styling.font_weight, FontWeight::NORMAL); // Raw mode shows markdown syntax, normal weight
        assert_eq!(raw_styling.font_size, 16.0); // Buffer font size for raw text
    }
}