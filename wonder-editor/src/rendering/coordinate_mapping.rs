use std::ops::Range;
use crate::markdown_parser::{ParsedToken, MarkdownParser, MarkdownToken};
use super::text_content::TextContent;
use super::token_mode::TokenRenderMode;

// ENG-173: Unified Coordinate System Data Structures
#[derive(Debug, Clone)]
pub struct CoordinateMap {
    /// Bidirectional position mapping: original_pos -> display_pos
    pub original_to_display: Vec<usize>,
    /// Bidirectional position mapping: display_pos -> original_pos  
    pub display_to_original: Vec<usize>,
    /// Token boundary tracking for accurate coordinate mapping
    pub token_boundaries: Vec<TokenBoundary>,
    /// Line offset tracking for multiline content
    pub line_offsets: Vec<LineOffset>,
}

#[derive(Debug, Clone)]
pub struct TokenBoundary {
    /// Start position in original content
    pub original_start: usize,
    /// End position in original content
    pub original_end: usize,
    /// Start position in display content
    pub display_start: usize,
    /// End position in display content
    pub display_end: usize,
    /// Type of markdown token
    pub token_type: MarkdownToken,
    /// How this token should be rendered
    pub render_mode: TokenRenderMode,
}

#[derive(Debug, Clone)]
pub struct LineOffset {
    /// Line number (0-based)
    pub line_number: usize,
    /// Character offset in original content where line starts
    pub original_offset: usize,
    /// Character offset in display content where line starts
    pub display_offset: usize,
}

#[derive(Clone)]
pub struct CoordinateMapper {
    parser: MarkdownParser,
}

impl CoordinateMapper {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }
    
    // ENG-173: Phase 2 - Create unified coordinate mapping system with token boundary tracking
    pub fn create_coordinate_map<T: TextContent>(
        &self, 
        content: T, 
        cursor_position: usize, 
        selection: Option<Range<usize>>
    ) -> CoordinateMap {
        let content_str = content.text_to_string();
        let selection_range = selection;
        
        // Get tokens and their render modes
        let tokens = self.parser.parse_with_positions(&content_str);
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens.into_iter()
            .map(|token| {
                let mode = TokenRenderMode::get_for_token(&token, cursor_position, selection_range.clone());
                (token, mode)
            })
            .collect();
        
        // Initialize coordinate mapping structures
        let mut original_to_display = Vec::new();
        let mut display_to_original = Vec::new();
        let mut token_boundaries = Vec::new();
        let line_offsets = Vec::new(); // TODO: Implement line offset tracking
        
        let mut display_pos = 0;
        let mut original_pos = 0;
        let mut last_token_end = 0;
        
        // Process content character by character, tracking token boundaries
        for (token, mode) in &token_modes {
            // Handle text before this token
            if token.start > last_token_end {
                let text_between = &content_str[last_token_end..token.start];
                for _ in text_between.chars() {
                    original_to_display.push(display_pos);
                    display_to_original.push(original_pos);
                    display_pos += 1;
                    original_pos += 1;
                }
            }
            
            // Process the token based on its render mode
            let display_start = display_pos;
            let display_content = match (&token.token_type, &mode) {
                (MarkdownToken::Bold(text), TokenRenderMode::Preview) |
                (MarkdownToken::Italic(text), TokenRenderMode::Preview) |
                (MarkdownToken::Strikethrough(text), TokenRenderMode::Preview) => {
                    // In preview mode, hide markdown syntax
                    text.clone()
                },
                (MarkdownToken::Code(text), TokenRenderMode::Preview) => {
                    // Code in preview mode (no backticks)
                    text.clone()
                },
                (MarkdownToken::Heading(_, text), TokenRenderMode::Preview) => {
                    // Heading in preview mode (no # symbols)
                    text.clone()
                },
                _ => {
                    // Raw mode or unsupported token: show original
                    content_str[token.start..token.end].to_string()
                }
            };
            
            // Map positions for this token
            let token_original_chars = token.end - token.start;
            let display_chars = display_content.chars().count();
            
            // Track token boundary
            token_boundaries.push(TokenBoundary {
                original_start: token.start,
                original_end: token.end,
                display_start,
                display_end: display_start + display_chars,
                token_type: token.token_type.clone(),
                render_mode: mode.clone(),
            });
            
            if mode == &TokenRenderMode::Preview {
                // In preview mode, map original positions to transformed display
                for i in 0..token_original_chars {
                    let progress = if token_original_chars > 0 {
                        i as f32 / token_original_chars as f32
                    } else {
                        0.0
                    };
                    let display_offset = (progress * display_chars as f32) as usize;
                    original_to_display.push(display_start + display_offset.min(display_chars.saturating_sub(1)));
                }
                
                // Map display positions back to original proportionally
                for i in 0..display_chars {
                    if display_chars == 0 {
                        display_to_original.push(token.start);
                    } else {
                        let progress = i as f32 / display_chars as f32;
                        let original_offset = (progress * (token.end - token.start) as f32) as usize;
                        display_to_original.push(token.start + original_offset.min(token.end - token.start));
                    }
                }
                
                display_pos += display_chars;
                original_pos = token.end;
            } else {
                // Raw mode: 1:1 mapping
                for i in 0..token_original_chars {
                    original_to_display.push(display_pos + i);
                    display_to_original.push(token.start + i);
                }
                display_pos += token_original_chars;
                original_pos = token.end;
            }
            
            last_token_end = token.end;
        }
        
        // Handle any remaining text after the last token
        if last_token_end < content_str.len() {
            let remaining_text = &content_str[last_token_end..];
            for _ in remaining_text.chars() {
                original_to_display.push(display_pos);
                display_to_original.push(original_pos);
                display_pos += 1;
                original_pos += 1;
            }
        }
        
        CoordinateMap {
            original_to_display,
            display_to_original,
            token_boundaries,
            line_offsets,
        }
    }
    
    pub fn map_cursor_position<T: TextContent>(
        &self, 
        content: T, 
        original_cursor_pos: usize, 
        selection: Option<Range<usize>>
    ) -> usize {
        let coordinate_map = self.create_coordinate_map(content, original_cursor_pos, selection);
        if original_cursor_pos < coordinate_map.original_to_display.len() {
            coordinate_map.original_to_display[original_cursor_pos]
        } else {
            coordinate_map.original_to_display.last().copied().unwrap_or(0)
        }
    }
    
    pub fn map_display_position_to_original<T: TextContent>(
        &self, 
        content: T, 
        display_position: usize, 
        cursor_position: usize, 
        selection: Option<(usize, usize)>
    ) -> usize {
        let selection_range = selection.map(|(start, end)| start..end);
        let coordinate_map = self.create_coordinate_map(content, cursor_position, selection_range);
        
        if display_position < coordinate_map.display_to_original.len() {
            coordinate_map.display_to_original[display_position]
        } else {
            coordinate_map.display_to_original.last().copied().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinate_map_creation() {
        let mapper = CoordinateMapper::new();
        let content = "Hello **world**!";
        
        // Test with cursor at position 0 (all tokens in preview mode)
        let map = mapper.create_coordinate_map(content, 0, None);
        
        // Should have mappings for all original positions
        assert!(map.original_to_display.len() > 0);
        assert!(map.display_to_original.len() > 0);
        
        // Should have token boundaries
        assert!(map.token_boundaries.len() > 0);
        
        // Test round-trip consistency
        for (original_pos, &display_pos) in map.original_to_display.iter().enumerate() {
            if display_pos < map.display_to_original.len() {
                let back_to_original = map.display_to_original[display_pos];
                assert!(
                    (back_to_original as i32 - original_pos as i32).abs() <= 2, 
                    "Round trip consistency failed: {} -> {} -> {}", 
                    original_pos, display_pos, back_to_original
                );
            }
        }
    }
    
    #[test]
    fn test_token_boundary_tracking() {
        let mapper = CoordinateMapper::new();
        let content = "Hello **world** and *italic* text";
        
        let map = mapper.create_coordinate_map(content, 8, None);
        
        // Should have token boundaries for bold and italic tokens
        assert!(map.token_boundaries.len() >= 2);
        
        // Find the bold token boundary
        let bold_boundary = map.token_boundaries.iter()
            .find(|b| matches!(b.token_type, MarkdownToken::Bold(_)))
            .expect("Should have bold token boundary");
        
        // Bold token "**world**" starts at position 6
        assert_eq!(bold_boundary.original_start, 6);
        assert_eq!(bold_boundary.original_end, 15);
        
        // When cursor is at position 8 (inside bold), it should be in Raw mode
        assert_eq!(bold_boundary.render_mode, TokenRenderMode::Raw);
    }
    
    #[test]
    fn test_coordinate_mapping_accuracy_mixed_content() {
        let mapper = CoordinateMapper::new();
        let content = "Hello **world** and *italic* text";
        
        // Test various cursor positions
        
        // 1. Before any markdown
        let display_pos_0 = mapper.map_cursor_position(content, 0, None);
        assert_eq!(display_pos_0, 0, "Position at start should map to 0");
        
        // 2. At start of bold markdown
        let display_pos_6 = mapper.map_cursor_position(content, 6, None); // At "**world**"
        assert_eq!(display_pos_6, 6, "Position at bold start should map correctly");
        
        // 3. Inside bold markdown  
        let display_pos_8 = mapper.map_cursor_position(content, 8, None); // Inside "**world**"
        let expected_8 = 8; // Position after "**world**" when bold token is in Raw mode
        assert_eq!(display_pos_8, expected_8, "Position inside bold should map correctly");
        
        // 4. At end of bold markdown
        let display_pos_13 = mapper.map_cursor_position(content, 13, None); // After "**world**"
        let expected_13 = 13; // Position after "**world**" when bold token is in Raw mode
        assert_eq!(display_pos_13, expected_13, "Position after bold should map to end of transformed content");
        
        // 5. Before italic markdown  
        let display_pos_18 = mapper.map_cursor_position(content, 18, None); // At "*italic*" 
        let expected_18 = 14; // Based on token boundary mapping: "Hello **world** " length when Bold is Raw
        assert_eq!(display_pos_18, expected_18, "Position at italic start should map correctly");
    }
    
    #[test]  
    fn test_coordinate_mapping_with_selection() {
        let mapper = CoordinateMapper::new();
        let content = "Hello **world** test";
        
        // Selection covering the bold token
        let selection = Some(5..14);
        let display_pos = mapper.map_cursor_position(content, 0, selection);
        
        // With selection, the bold token should be in Raw mode
        // So position mapping should reflect raw content
        assert_eq!(display_pos, 0);
    }
    
    #[test]
    fn test_coordinate_consistency_round_trip() {
        let mapper = CoordinateMapper::new();
        let content = "Test **bold** text";
        
        // Test round-trip consistency for various positions
        for original in [0, 5, 6, 10, 13, 18] {
            let display = mapper.map_cursor_position(content, original, None);
            let back_to_original = mapper.map_display_position_to_original(
                content, 
                display, 
                original, 
                None
            );
            
            // Should return to approximately the same position (within tolerance)
            let diff = (back_to_original as i32 - original as i32).abs();
            assert!(diff <= 2, "Round trip failed for position {}: {} -> {} -> {}, diff = {}", 
                    original, original, display, back_to_original, diff);
        }
    }
    
    #[test]
    fn test_coordinate_accuracy_with_multiline_content() {
        let mapper = CoordinateMapper::new();
        let content = "Line 1 **bold**\nLine 2 *italic*\nLine 3 normal";
        
        // Find line positions
        let line2_start = content.find("Line 2").unwrap();
        
        // Position in italic on line 2
        let italic_start = content.find("*italic*").unwrap();
        let pos_in_italic = mapper.map_cursor_position(content, italic_start + 1, None); // Inside "*italic*"
        
        // When cursor is inside italic token, that token stays in Raw mode
        // So we only get savings from the bold token transformation (-4 chars)
        // Expected: line2_start - 4 + 8 = 16 - 4 + 8 = 20 ("Line 2 *" length after bold transformation)  
        let expected_italic = line2_start - 4 + 8; 
        assert_eq!(pos_in_italic, expected_italic, "Position in italic should map correctly");
    }
}