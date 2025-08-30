use std::ops::Range;
use gpui::{TextRun, Font, FontFeatures, FontWeight, FontStyle, rgb};
use crate::markdown_parser::{ParsedToken, MarkdownParser, MarkdownToken};
use super::text_content::TextContent;
use super::token_mode::TokenRenderMode;
use super::style_context::StyleContext;
use super::typography::Typography;

#[derive(Debug, Clone)]
pub struct StyledTextSegment {
    pub text: String,
    pub text_run: TextRun,
    pub font_size: f32,
}

#[derive(Clone)]
pub struct TextRunGenerator {
    parser: MarkdownParser,
}

impl TextRunGenerator {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }
    
    pub fn generate_styled_text_segments_with_context<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>, 
        style_context: &StyleContext, 
        buffer_font_size: f32
    ) -> Vec<StyledTextSegment> {
        if content.text_is_empty() {
            return vec![];
        }
        
        let content_str = content.text_to_string();
        let tokens = self.parser.parse_with_positions(&content_str);
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect();
        
        self.build_text_segments(&content_str, &token_modes, style_context, buffer_font_size)
    }

    pub fn generate_styled_text_segments<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> Vec<StyledTextSegment> {
        if content.text_is_empty() {
            return vec![];
        }
        
        let content_str = content.text_to_string();
        let tokens = self.parser.parse_with_positions(&content_str);
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect();
        
        // Use default style context for backward compatibility
        let default_style_context = StyleContext::default();
        self.build_text_segments(&content_str, &token_modes, &default_style_context, 16.0)
    }
    
    fn build_text_segments(
        &self,
        content_str: &str,
        token_modes: &[(ParsedToken, TokenRenderMode)],
        style_context: &StyleContext,
        buffer_font_size: f32
    ) -> Vec<StyledTextSegment> {
        let mut segments = Vec::new();
        let mut current_pos = 0;
        
        // Sort tokens by start position to process them in order
        let mut sorted_tokens: Vec<_> = token_modes.iter().collect();
        sorted_tokens.sort_by_key(|(token, _)| token.start);
        
        for (token, mode) in sorted_tokens {
            // Add any text before this token using theme colors
            if token.start > current_pos {
                let before_text = &content_str[current_pos..token.start];
                segments.push(StyledTextSegment {
                    text: before_text.to_string(),
                    text_run: TextRun {
                        len: before_text.len(),
                        font: Font {
                            family: "system-ui".into(),
                            features: FontFeatures::default(),
                            weight: FontWeight::NORMAL,
                            style: FontStyle::Normal,
                            fallbacks: None,
                        },
                        color: style_context.text_color.into(),
                        background_color: None,
                        underline: Default::default(),
                        strikethrough: Default::default(),
                    },
                    font_size: Typography::get_scalable_font_size_for_regular_text(buffer_font_size),
                });
            }
            
            // Handle the token based on its mode
            let (display_text, font_weight, font_style, color, font_family, font_size) = match mode {
                TokenRenderMode::Raw => {
                    // Raw mode: show original markdown syntax with muted colors
                    let original_text = &content_str[token.start..token.end];
                    (
                        original_text.to_string(), 
                        FontWeight::NORMAL, 
                        FontStyle::Normal, 
                        style_context.text_color, 
                        "system-ui",
                        Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                    )
                }
                TokenRenderMode::Preview => {
                    // Preview mode: show transformed content with appropriate styling
                    match &token.token_type {
                        MarkdownToken::Bold(inner_content) => {
                            (
                                inner_content.clone(), 
                                FontWeight::BOLD, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                        MarkdownToken::Italic(inner_content) => {
                            (
                                inner_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Italic, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                        MarkdownToken::Heading(level, content) => {
                            let font_size = Typography::get_scalable_font_size_for_heading_level(*level, buffer_font_size);
                            (
                                content.clone(), 
                                FontWeight::BOLD, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                font_size
                            )
                        }
                        MarkdownToken::Code(inner_content) => {
                            (
                                inner_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.code_color, 
                                "monospace",
                                Typography::get_scalable_font_size_for_code(buffer_font_size)
                            )
                        }
                        MarkdownToken::Tag(tag_content) => {
                            (
                                format!("#{}", tag_content), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                        MarkdownToken::Highlight(highlight_content) => {
                            (
                                highlight_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                        MarkdownToken::Emoji(emoji_content) => {
                            (
                                emoji_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                        MarkdownToken::Html(html_content) => {
                            (
                                html_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "monospace",
                                Typography::get_scalable_font_size_for_code(buffer_font_size)
                            )
                        }
                        MarkdownToken::Subscript(sub_content) => {
                            let font_size = Typography::scaled_rems(0.8, buffer_font_size);
                            (
                                sub_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                font_size
                            )
                        }
                        MarkdownToken::Superscript(sup_content) => {
                            let font_size = Typography::scaled_rems(0.8, buffer_font_size);
                            (
                                sup_content.clone(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                font_size
                            )
                        }
                        _ => {
                            // For other tokens, show original text
                            let original_text = &content_str[token.start..token.end];
                            (
                                original_text.to_string(), 
                                FontWeight::NORMAL, 
                                FontStyle::Normal, 
                                style_context.text_color, 
                                "system-ui",
                                Typography::get_scalable_font_size_for_regular_text(buffer_font_size)
                            )
                        }
                    }
                }
            };
            
            segments.push(StyledTextSegment {
                text: display_text.clone(),
                text_run: TextRun {
                    len: display_text.len(),
                    font: Font {
                        family: font_family.into(),
                        features: FontFeatures::default(),
                        weight: font_weight,
                        style: font_style,
                        fallbacks: None,
                    },
                    color: color.into(),
                    background_color: None,
                    underline: Default::default(),
                    strikethrough: Default::default(),
                },
                font_size,
            });
            
            current_pos = token.end;
        }
        
        // Add any remaining text after the last token
        if current_pos < content_str.len() {
            let remaining_text = &content_str[current_pos..];
            segments.push(StyledTextSegment {
                text: remaining_text.to_string(),
                text_run: TextRun {
                    len: remaining_text.len(),
                    font: Font {
                        family: "system-ui".into(),
                        features: FontFeatures::default(),
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: style_context.text_color.into(),
                    background_color: None,
                    underline: Default::default(),
                    strikethrough: Default::default(),
                },
                font_size: Typography::get_scalable_font_size_for_regular_text(buffer_font_size),
            });
        }
        
        segments
    }
    
    pub fn generate_mixed_text_runs<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> Vec<TextRun> {
        if content.text_is_empty() {
            return vec![];
        }
        
        let content_str = content.text_to_string();
        let tokens = self.parser.parse_with_positions(&content_str);
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect();
        
        // Build the transformed content and corresponding TextRuns
        let (_transformed_content, mut text_runs) = self.build_transformed_content_with_proper_runs(&content_str, &token_modes);
        
        // Apply selection highlighting if there's a selection  
        if let Some(sel) = selection {
            text_runs = self.apply_selection_highlighting_to_transformed(text_runs, &content_str, sel, &token_modes);
        }
        
        text_runs
    }
    
    /// Returns the transformed content string that should be displayed
    pub fn get_display_content<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> String {
        if content.text_is_empty() {
            return String::new();
        }
        
        let content_str = content.text_to_string();
        let tokens = self.parser.parse_with_positions(&content_str);
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect();
        
        let (transformed_content, _) = self.build_transformed_content_with_proper_runs(&content_str, &token_modes);
        transformed_content
    }
    
    fn build_transformed_content_with_proper_runs(
        &self, 
        original_content: &str, 
        token_modes: &[(ParsedToken, TokenRenderMode)]
    ) -> (String, Vec<TextRun>) {
        let mut transformed_text = String::new();
        let mut text_runs = Vec::new();
        let mut current_pos = 0;
        
        // Sort tokens by start position to process them in order
        let mut sorted_tokens: Vec<_> = token_modes.iter().collect();
        sorted_tokens.sort_by_key(|(token, _)| token.start);
        
        for (token, mode) in sorted_tokens {
            // Add any text before this token
            if token.start > current_pos {
                let before_text = &original_content[current_pos..token.start];
                transformed_text.push_str(before_text);
                
                // Add TextRun for the text before token
                text_runs.push(TextRun {
                    len: before_text.len(),
                    font: Font {
                        family: "system-ui".into(),
                        features: FontFeatures::default(),
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: rgb(0xcdd6f4).into(),
                    background_color: None,
                    underline: Default::default(),
                    strikethrough: Default::default(),
                });
            }
            
            // Handle the token based on its mode
            let (display_text, font_weight, font_style, color, font_family) = match mode {
                TokenRenderMode::Raw => {
                    // Raw mode: show original markdown syntax
                    let original_text = &original_content[token.start..token.end];
                    (original_text.to_string(), FontWeight::NORMAL, FontStyle::Normal, rgb(0x94a3b8), "system-ui")
                }
                TokenRenderMode::Preview => {
                    // Preview mode: show transformed content without markdown syntax
                    match &token.token_type {
                        MarkdownToken::Bold(inner_content) => {
                            (inner_content.clone(), FontWeight::BOLD, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                        MarkdownToken::Italic(inner_content) => {
                            (inner_content.clone(), FontWeight::NORMAL, FontStyle::Italic, rgb(0xcdd6f4), "system-ui")
                        }
                        MarkdownToken::Heading(_level, content) => {
                            (content.clone(), FontWeight::BOLD, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                        MarkdownToken::Code(inner_content) => {
                            (inner_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xa6da95), "monospace")
                        }
                        MarkdownToken::Tag(tag_content) => {
                            (format!("#{}", tag_content), FontWeight::NORMAL, FontStyle::Normal, rgb(0xf9e2af), "system-ui")
                        }
                        MarkdownToken::Highlight(highlight_content) => {
                            (highlight_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0x000000), "system-ui")
                        }
                        MarkdownToken::Emoji(emoji_content) => {
                            (emoji_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                        MarkdownToken::Html(html_content) => {
                            (html_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xf38ba8), "monospace")
                        }
                        MarkdownToken::Subscript(sub_content) => {
                            (sub_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                        MarkdownToken::Superscript(sup_content) => {
                            (sup_content.clone(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                        _ => {
                            // For other tokens, show original text
                            let original_text = &original_content[token.start..token.end];
                            (original_text.to_string(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xcdd6f4), "system-ui")
                        }
                    }
                }
            };
            
            transformed_text.push_str(&display_text);
            
            // Add TextRun for this token (using the display text length, not original)
            text_runs.push(TextRun {
                len: display_text.len(),
                font: Font {
                    family: font_family.into(),
                    features: FontFeatures::default(),
                    weight: font_weight,
                    style: font_style,
                    fallbacks: None,
                },
                color: color.into(),
                background_color: None,
                underline: Default::default(),
                strikethrough: Default::default(),
            });
            
            current_pos = token.end;
        }
        
        // Add any remaining text after the last token
        if current_pos < original_content.len() {
            let remaining_text = &original_content[current_pos..];
            transformed_text.push_str(remaining_text);
            
            text_runs.push(TextRun {
                len: remaining_text.len(),
                font: Font {
                    family: "system-ui".into(),
                    features: FontFeatures::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                    fallbacks: None,
                },
                color: rgb(0xcdd6f4).into(),
                background_color: None,
                underline: Default::default(),
                strikethrough: Default::default(),
            });
        }
        
        (transformed_text, text_runs)
    }
    
    fn apply_selection_highlighting_to_transformed(
        &self, 
        text_runs: Vec<TextRun>, 
        _original_content: &str, 
        _selection: Range<usize>, 
        _token_modes: &[(ParsedToken, TokenRenderMode)]
    ) -> Vec<TextRun> {
        // For now, we'll simplify selection highlighting since we transformed the content
        // This is complex to implement correctly because we need to map original positions to transformed positions
        // Let's return the original text_runs for now - this is a good foundation
        // TODO: Implement proper position mapping from original to transformed content
        text_runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_styled_text_segments() {
        let generator = TextRunGenerator::new();
        let content = "Regular text **bold text** `code`";
        
        // With cursor outside all tokens, should generate 4 segments with different styles
        let segments = generator.generate_styled_text_segments(content, 100, None);
        
        // Should have 4 segments: "Regular text " (16px), "bold text" (16px bold), " " (16px), "code" (14px monospace)
        assert_eq!(segments.len(), 4);
        
        // First segment: regular text before bold
        assert_eq!(segments[0].text, "Regular text ");
        assert_eq!(segments[0].font_size, 16.0);
        assert_eq!(segments[0].text_run.font.weight, gpui::FontWeight::NORMAL);
        
        // Second segment: bold text (transformed, no asterisks)
        assert_eq!(segments[1].text, "bold text");
        assert_eq!(segments[1].font_size, 16.0);
        assert_eq!(segments[1].text_run.font.weight, gpui::FontWeight::BOLD);
        
        // Third segment: space between bold and code
        assert_eq!(segments[2].text, " ");
        assert_eq!(segments[2].font_size, 16.0);
        
        // Fourth segment: code text (transformed, no backticks)
        assert_eq!(segments[3].text, "code");
        assert_eq!(segments[3].font_size, 14.0);
        assert_eq!(segments[3].text_run.font.family.as_ref(), "monospace");
    }
    
    #[test]
    fn test_heading_styling_with_different_font_sizes() {
        let generator = TextRunGenerator::new();
        
        // Test H1 with large font size
        let content = "# Large Heading";
        let segments = generator.generate_styled_text_segments(content, 100, None);
        
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Large Heading");
        assert_eq!(segments[0].font_size, 32.0); // H1 should be 2x buffer font size (32px)
        assert_eq!(segments[0].text_run.font.weight, gpui::FontWeight::BOLD);
        
        // Test H3 with medium font size
        let content = "### Medium Heading";
        let segments = generator.generate_styled_text_segments(content, 100, None);
        
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Medium Heading");
        assert_eq!(segments[0].font_size, 20.0); // H3 should be 1.25x buffer font size (20px)
        assert_eq!(segments[0].text_run.font.weight, gpui::FontWeight::BOLD);
    }
    
    #[test]
    fn test_generator_uses_style_context_colors() {
        let generator = TextRunGenerator::new();
        let style_context = StyleContext::new_for_test();
        
        // This should use StyleContext colors instead of hardcoded rgb() calls
        let segments = generator.generate_styled_text_segments_with_context(
            "Regular text **bold text**", 
            100, 
            None, 
            &style_context,
            16.0
        );
        
        assert_eq!(segments.len(), 2);
        
        // First segment should use text_color from StyleContext
        assert_eq!(segments[0].text_run.color, style_context.text_color.into());
        
        // Second segment (bold) should also use text_color from StyleContext  
        assert_eq!(segments[1].text_run.color, style_context.text_color.into());
    }
    
    #[test]
    fn test_get_display_content() {
        let generator = TextRunGenerator::new();
        let content = "Hello **world** and `code`";
        
        let display_content = generator.get_display_content(content, 0, None);
        
        // Should transform markdown tokens when cursor is outside them
        assert_eq!(display_content, "Hello world and code");
    }
}