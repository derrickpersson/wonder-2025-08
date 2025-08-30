use std::ops::Range;
use crate::markdown_parser::ParsedToken;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenRenderMode {
    Raw,
    Preview,
}

/// Utility function to check if two ranges intersect
pub fn ranges_intersect(sel_start: usize, sel_end: usize, token_start: usize, token_end: usize) -> bool {
    sel_start <= token_end && sel_end >= token_start
}

impl TokenRenderMode {
    /// Determines if a token should be rendered in Raw or Preview mode based on cursor and selection
    pub fn get_for_token(token: &ParsedToken, cursor_position: usize, selection: Option<Range<usize>>) -> TokenRenderMode {
        let token_start = token.start;
        let token_end = token.end;
        
        // 1. Token contains cursor -> Raw mode for editing
        if cursor_position >= token_start && cursor_position <= token_end {
            return TokenRenderMode::Raw;
        }
        
        // 2. Token intersects with selection -> Raw mode for clarity
        if let Some(sel) = selection {
            if ranges_intersect(sel.start, sel.end, token_start, token_end) {
                return TokenRenderMode::Raw;
            }
        }
        
        // 3. Default to Preview mode for better visual representation
        TokenRenderMode::Preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::MarkdownToken;
    
    #[test]
    fn test_cursor_inside_token_should_be_raw() {
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("text".to_string()),
            start: 5,
            end: 13,
        };
        
        // Cursor at position 8 (inside the token)
        let mode = TokenRenderMode::get_for_token(&token, 8, None);
        assert_eq!(mode, TokenRenderMode::Raw);
        
        // Cursor at token boundaries should still be Raw
        assert_eq!(TokenRenderMode::get_for_token(&token, 5, None), TokenRenderMode::Raw);
        assert_eq!(TokenRenderMode::get_for_token(&token, 13, None), TokenRenderMode::Raw);
    }
    
    #[test]
    fn test_cursor_outside_token_should_be_preview() {
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("text".to_string()),
            start: 5,
            end: 13,
        };
        
        // Cursor before token
        let mode = TokenRenderMode::get_for_token(&token, 2, None);
        assert_eq!(mode, TokenRenderMode::Preview);
        
        // Cursor after token
        let mode_after = TokenRenderMode::get_for_token(&token, 15, None);
        assert_eq!(mode_after, TokenRenderMode::Preview);
    }
    
    #[test]
    fn test_selection_intersecting_token_should_be_raw() {
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("text".to_string()),
            start: 5,
            end: 13,
        };
        
        // Selection partially overlapping token
        let selection = Some(3..7);
        let mode = TokenRenderMode::get_for_token(&token, 0, selection);
        assert_eq!(mode, TokenRenderMode::Raw);
        
        // Selection fully containing token
        let selection_full = Some(2..15);
        let mode_full = TokenRenderMode::get_for_token(&token, 0, selection_full);
        assert_eq!(mode_full, TokenRenderMode::Raw);
        
        // Selection inside token
        let selection_inside = Some(7..10);
        let mode_inside = TokenRenderMode::get_for_token(&token, 0, selection_inside);
        assert_eq!(mode_inside, TokenRenderMode::Raw);
    }
    
    #[test]
    fn test_selection_not_intersecting_token_should_be_preview() {
        let token = ParsedToken {
            token_type: MarkdownToken::Bold("text".to_string()),
            start: 5,
            end: 13,
        };
        
        // Selection before token
        let selection = Some(0..3);
        let mode = TokenRenderMode::get_for_token(&token, 20, selection);
        assert_eq!(mode, TokenRenderMode::Preview);
        
        // Selection after token
        let selection_after = Some(15..20);
        let mode_after = TokenRenderMode::get_for_token(&token, 0, selection_after);
        assert_eq!(mode_after, TokenRenderMode::Preview);
    }
    
    #[test]
    fn test_ranges_intersect() {
        // Test various intersection scenarios
        assert!(ranges_intersect(0, 5, 3, 7));  // Partial overlap
        assert!(ranges_intersect(3, 7, 0, 5));  // Partial overlap (reversed)
        assert!(ranges_intersect(2, 8, 3, 5));  // Complete containment
        assert!(ranges_intersect(3, 5, 2, 8));  // Complete containment (reversed)
        assert!(!ranges_intersect(0, 2, 3, 5)); // No overlap
        assert!(!ranges_intersect(3, 5, 0, 2)); // No overlap (reversed)
        assert!(ranges_intersect(0, 3, 3, 5));  // Adjacent (touching at boundary)
    }
}