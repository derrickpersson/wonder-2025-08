use std::ops::Range;
use crate::markdown_parser::{ParsedToken, MarkdownParser};
use gpui::{TextRun, px, rgb, Font, FontFeatures, FontWeight, FontStyle};

fn ranges_intersect(sel_start: usize, sel_end: usize, token_start: usize, token_end: usize) -> bool {
    sel_start <= token_end && sel_end >= token_start
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenRenderMode {
    Raw,
    Preview,
}

pub struct HybridTextRenderer {
    parser: MarkdownParser,
}

impl HybridTextRenderer {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }
    
    pub fn get_token_render_mode(&self, token: &ParsedToken, cursor_position: usize, selection: Option<Range<usize>>) -> TokenRenderMode {
        // Check if there's a selection that intersects with the token
        if let Some(sel) = selection {
            if ranges_intersect(sel.start, sel.end, token.start, token.end) {
                return TokenRenderMode::Raw;
            }
        }
        
        // Check if cursor is inside the token
        if cursor_position >= token.start && cursor_position <= token.end {
            TokenRenderMode::Raw
        } else {
            TokenRenderMode::Preview
        }
    }
    
    pub fn render_document(&self, content: &str, cursor_position: usize, selection: Option<Range<usize>>) -> Vec<(ParsedToken, TokenRenderMode)> {
        let tokens = self.parser.parse_with_positions(content);
        tokens.into_iter()
            .map(|token| {
                let mode = self.get_token_render_mode(&token, cursor_position, selection.clone());
                (token, mode)
            })
            .collect()
    }
    
    pub fn generate_mixed_text_runs(&self, content: &str, cursor_position: usize, selection: Option<Range<usize>>) -> Vec<TextRun> {
        if content.is_empty() {
            return vec![];
        }
        
        let token_modes = self.render_document(content, cursor_position, selection);
        
        // Create a list of text segments with their render modes
        let mut segments = Vec::new();
        let mut last_end = 0;
        
        for (token, mode) in token_modes {
            // Add text before this token as normal text (if any)
            if token.start > last_end {
                segments.push((last_end, token.start, TokenRenderMode::Preview));
            }
            
            // Add this token with its determined mode
            segments.push((token.start, token.end, mode));
            last_end = token.end;
        }
        
        // Add any remaining text after the last token
        if last_end < content.len() {
            segments.push((last_end, content.len(), TokenRenderMode::Preview));
        }
        
        // If no tokens, render entire content as preview
        if segments.is_empty() {
            segments.push((0, content.len(), TokenRenderMode::Preview));
        }
        
        // Convert segments to TextRuns with appropriate styling
        let mut text_runs = Vec::new();
        for (start, end, mode) in segments {
            if start < end {
                let len = end - start;
                let (font_weight, color) = match mode {
                    TokenRenderMode::Raw => {
                        // Raw mode: normal weight, lighter color to show it's being edited
                        (FontWeight::NORMAL, rgb(0x94a3b8)) // Slightly dimmed
                    }
                    TokenRenderMode::Preview => {
                        // Preview mode: bold weight, bright color to show it's formatted
                        (FontWeight::BOLD, rgb(0xcdd6f4)) // Normal bright color
                    }
                };
                
                text_runs.push(TextRun {
                    len,
                    font: Font {
                        family: "system-ui".into(),
                        features: FontFeatures::default(),
                        weight: font_weight,
                        style: FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: color.into(),
                    background_color: None,
                    underline: Default::default(),
                    strikethrough: Default::default(),
                });
            }
        }
        
        text_runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::{ParsedToken, MarkdownToken};

    #[test]
    fn test_create_hybrid_text_renderer() {
        let _renderer = HybridTextRenderer::new();
        // Should not panic - basic creation test
    }

    #[test]
    fn test_cursor_inside_token_should_be_raw() {
        let renderer = HybridTextRenderer::new();
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("test".to_string()),
            start: 5,
            end: 10,
        };
        
        let mode = renderer.get_token_render_mode(&token, 7, None);
        assert_eq!(mode, TokenRenderMode::Raw);
    }

    #[test]
    fn test_cursor_outside_token_should_be_preview() {
        let renderer = HybridTextRenderer::new();
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("test".to_string()),
            start: 5,
            end: 10,
        };
        
        // Cursor before token
        let mode = renderer.get_token_render_mode(&token, 3, None);
        assert_eq!(mode, TokenRenderMode::Preview);
        
        // Cursor after token
        let mode = renderer.get_token_render_mode(&token, 12, None);
        assert_eq!(mode, TokenRenderMode::Preview);
    }

    #[test]
    fn test_selection_intersecting_token_should_be_raw() {
        let renderer = HybridTextRenderer::new();
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("test".to_string()),
            start: 5,
            end: 10,
        };
        
        // Selection partially overlapping token (starts before, ends inside)
        let mode = renderer.get_token_render_mode(&token, 4, Some(3..7));
        assert_eq!(mode, TokenRenderMode::Raw);
        
        // Selection completely containing token
        let mode = renderer.get_token_render_mode(&token, 4, Some(2..12));
        assert_eq!(mode, TokenRenderMode::Raw);
        
        // Selection starting inside token
        let mode = renderer.get_token_render_mode(&token, 7, Some(7..15));
        assert_eq!(mode, TokenRenderMode::Raw);
    }

    #[test]
    fn test_render_document_with_mixed_modes() {
        let renderer = HybridTextRenderer::new();
        let content = "# Title **bold** normal";
        
        // Test with cursor position outside any token - use position beyond content length
        let modes = renderer.render_document(content, 100, None);
        
        // We should have at least one token and they should all be Preview mode 
        // since cursor is well outside all tokens
        assert!(modes.len() >= 1);
        for (_, mode) in &modes {
            assert_eq!(*mode, TokenRenderMode::Preview);
        }
        
        // Test with cursor inside a token - put it at position 1 (inside heading)
        let modes_with_cursor_in_heading = renderer.render_document(content, 1, None);
        
        // Find the heading token and verify it's Raw mode
        let heading_token = modes_with_cursor_in_heading.iter()
            .find(|(token, _)| matches!(token.token_type, crate::markdown_parser::MarkdownToken::Heading(_, _)));
        
        if let Some((_, mode)) = heading_token {
            assert_eq!(*mode, TokenRenderMode::Raw);
        }
    }
}