use std::ops::Range;
use crate::markdown_parser::{ParsedToken, MarkdownParser, MarkdownToken};
use gpui::{TextRun, rgb, Font, FontFeatures, FontWeight, FontStyle};

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
        
        let token_modes = self.render_document(content, cursor_position, selection.clone());
        
        // Build the transformed text and corresponding TextRuns
        let (_transformed_text, mut text_runs) = self.build_transformed_content(content, &token_modes);
        
        // Apply selection highlighting if there's a selection
        if let Some(sel) = selection {
            text_runs = self.apply_selection_highlighting(text_runs, content, sel);
        }
        
        text_runs
    }
    
    fn build_transformed_content(&self, original_content: &str, token_modes: &[(ParsedToken, TokenRenderMode)]) -> (String, Vec<TextRun>) {
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
                
                // Add TextRun for the text before token (always preview mode for regular text)
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
            let (display_text, font_weight, font_style, color) = match mode {
                TokenRenderMode::Raw => {
                    // Raw mode: show original markdown syntax
                    let original_text = &original_content[token.start..token.end];
                    (original_text.to_string(), FontWeight::NORMAL, FontStyle::Normal, rgb(0x94a3b8))
                }
                TokenRenderMode::Preview => {
                    // Preview mode: show formatted content
                    match &token.token_type {
                        MarkdownToken::Bold(inner_content) => {
                            (inner_content.clone(), FontWeight::BOLD, FontStyle::Normal, rgb(0xcdd6f4))
                        }
                        MarkdownToken::Italic(inner_content) => {
                            (inner_content.clone(), FontWeight::NORMAL, FontStyle::Italic, rgb(0xcdd6f4))
                        }
                        _ => {
                            // For other tokens, show original text
                            let original_text = &original_content[token.start..token.end];
                            (original_text.to_string(), FontWeight::NORMAL, FontStyle::Normal, rgb(0xcdd6f4))
                        }
                    }
                }
            };
            
            transformed_text.push_str(&display_text);
            
            // Add TextRun for this token
            text_runs.push(TextRun {
                len: display_text.len(),
                font: Font {
                    family: "system-ui".into(),
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
    
    fn apply_selection_highlighting(&self, text_runs: Vec<TextRun>, _content: &str, selection: Range<usize>) -> Vec<TextRun> {
        let mut highlighted_runs = Vec::new();
        let mut current_pos = 0;
        let selection_color = rgb(0x4c7cf9).into(); // Blue highlight color
        
        for run in text_runs {
            let run_start = current_pos;
            let run_end = current_pos + run.len;
            
            // Check if this run intersects with the selection
            if selection.start < run_end && selection.end > run_start {
                // Calculate intersection boundaries
                let highlight_start = selection.start.max(run_start);
                let highlight_end = selection.end.min(run_end);
                
                // Split the run into up to 3 parts: before, highlighted, after
                
                // Part 1: Before highlight (if any)
                if highlight_start > run_start {
                    let before_len = highlight_start - run_start;
                    highlighted_runs.push(TextRun {
                        len: before_len,
                        font: run.font.clone(),
                        color: run.color,
                        background_color: run.background_color,
                        underline: run.underline.clone(),
                        strikethrough: run.strikethrough.clone(),
                    });
                }
                
                // Part 2: Highlighted section
                if highlight_end > highlight_start {
                    let highlight_len = highlight_end - highlight_start;
                    highlighted_runs.push(TextRun {
                        len: highlight_len,
                        font: run.font.clone(),
                        color: run.color,
                        background_color: Some(selection_color),
                        underline: run.underline.clone(),
                        strikethrough: run.strikethrough.clone(),
                    });
                }
                
                // Part 3: After highlight (if any)
                if run_end > highlight_end {
                    let after_len = run_end - highlight_end;
                    highlighted_runs.push(TextRun {
                        len: after_len,
                        font: run.font,
                        color: run.color,
                        background_color: run.background_color,
                        underline: run.underline,
                        strikethrough: run.strikethrough,
                    });
                }
            } else {
                // No intersection with selection, keep original run
                highlighted_runs.push(run);
            }
            
            current_pos = run_end;
        }
        
        highlighted_runs
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

    #[test]
    fn test_content_transformation_bold_preview() {
        let renderer = HybridTextRenderer::new();
        let content = "**bold text**";
        
        // With cursor outside token, bold should be in preview mode
        let text_runs = renderer.generate_mixed_text_runs(content, 100, None);
        
        // Should have exactly one text run for the transformed content
        assert_eq!(text_runs.len(), 1);
        
        // The text run should be for "bold text" (without asterisks) and bold weight
        let run = &text_runs[0];
        assert_eq!(run.len, "bold text".len());
        assert_eq!(run.font.weight, gpui::FontWeight::BOLD);
    }

    #[test]
    fn test_content_transformation_italic_preview() {
        let renderer = HybridTextRenderer::new();
        let content = "*italic text*";
        
        // With cursor outside token, italic should be in preview mode
        let text_runs = renderer.generate_mixed_text_runs(content, 100, None);
        
        // Should have exactly one text run for the transformed content
        assert_eq!(text_runs.len(), 1);
        
        // The text run should be for "italic text" (without asterisks) and italic style
        let run = &text_runs[0];
        assert_eq!(run.len, "italic text".len());
        assert_eq!(run.font.style, gpui::FontStyle::Italic);
    }

    #[test]
    fn test_content_transformation_bold_raw() {
        let renderer = HybridTextRenderer::new();
        let content = "**bold text**";
        
        // With cursor inside token (position 5), bold should be in raw mode
        let text_runs = renderer.generate_mixed_text_runs(content, 5, None);
        
        // Should have exactly one text run for the original markdown
        assert_eq!(text_runs.len(), 1);
        
        // The text run should be for the full "**bold text**" and normal weight
        let run = &text_runs[0];
        assert_eq!(run.len, "**bold text**".len());
        assert_eq!(run.font.weight, gpui::FontWeight::NORMAL);
    }

    #[test]
    fn test_mixed_content_with_bold_and_text() {
        let renderer = HybridTextRenderer::new();
        let content = "Regular text **bold text** more text";
        
        // With cursor outside all tokens, bold should be in preview mode
        let text_runs = renderer.generate_mixed_text_runs(content, 100, None);
        
        // Should have 3 text runs: regular text, bold text (transformed), more text
        assert_eq!(text_runs.len(), 3);
        
        // First run: "Regular text "
        assert_eq!(text_runs[0].len, "Regular text ".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL);
        
        // Second run: "bold text" (transformed from **bold text**)
        assert_eq!(text_runs[1].len, "bold text".len());
        assert_eq!(text_runs[1].font.weight, gpui::FontWeight::BOLD);
        
        // Third run: " more text"
        assert_eq!(text_runs[2].len, " more text".len());
        assert_eq!(text_runs[2].font.weight, gpui::FontWeight::NORMAL);
    }

    #[test]
    fn test_selection_with_markdown_content() {
        let renderer = HybridTextRenderer::new();
        let content = "Regular **bold** text";
        
        // Test selection that intersects with bold token - this should make the intersecting token raw
        let text_runs = renderer.generate_mixed_text_runs(content, 12, Some(8..16));
        
        // Should have 3 text runs, with the bold token in raw mode due to selection
        assert_eq!(text_runs.len(), 3);
        
        // First run: "Regular "
        assert_eq!(text_runs[0].len, "Regular ".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL);
        
        // Second run: "**bold**" (raw mode due to selection)
        assert_eq!(text_runs[1].len, "**bold**".len());
        assert_eq!(text_runs[1].font.weight, gpui::FontWeight::NORMAL);
        
        // Third run: " text"
        assert_eq!(text_runs[2].len, " text".len());
        assert_eq!(text_runs[2].font.weight, gpui::FontWeight::NORMAL);
    }

    #[test]
    fn test_selection_extends_beyond_token() {
        let renderer = HybridTextRenderer::new();
        let content = "Hello **world** test";
        
        // Selection from 6 to 15 spans the entire bold token plus some surrounding text
        let text_runs = renderer.generate_mixed_text_runs(content, 10, Some(6..15));
        
        // Should have 3 text runs, with the bold token in raw mode due to selection intersection
        assert_eq!(text_runs.len(), 3);
        
        // First run: "Hello "
        assert_eq!(text_runs[0].len, "Hello ".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL);
        
        // Second run: "**world**" (raw mode due to selection intersection)
        assert_eq!(text_runs[1].len, "**world**".len());
        assert_eq!(text_runs[1].font.weight, gpui::FontWeight::NORMAL);
        
        // Third run: " test"
        assert_eq!(text_runs[2].len, " test".len());
        assert_eq!(text_runs[2].font.weight, gpui::FontWeight::NORMAL);
    }

    #[test]
    fn test_selection_background_highlighting() {
        let renderer = HybridTextRenderer::new();
        let content = "Hello world";
        
        // Selection from position 2 to 7 ("llo w")
        let text_runs = renderer.generate_mixed_text_runs(content, 5, Some(2..7));
        
        // Find text runs that should have background highlighting
        let mut found_highlighted = false;
        for run in &text_runs {
            if run.background_color.is_some() {
                found_highlighted = true;
                // Check that it has the selection highlight color
                let selection_color = rgb(0x4c7cf9).into();
                assert_eq!(run.background_color.unwrap(), selection_color);
            }
        }
        
        // Should have at least one text run with background highlighting
        assert!(found_highlighted, "No text runs found with selection background highlighting");
    }
}