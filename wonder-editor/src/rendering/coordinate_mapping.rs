use super::text_content::TextContent;
use super::token_mode::TokenRenderMode;
use crate::markdown_parser::{MarkdownParser, MarkdownToken, ParsedToken};
use std::ops::Range;

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

    /// Convert a byte position to a character position within the given content
    fn byte_to_char_position(content: &str, byte_pos: usize) -> usize {
        if byte_pos == 0 {
            return 0;
        }
        if byte_pos >= content.len() {
            return content.chars().count();
        }

        // Count characters up to the byte position
        let mut char_count = 0;
        let mut byte_count = 0;

        for ch in content.chars() {
            if byte_count >= byte_pos {
                break;
            }
            byte_count += ch.len_utf8();
            char_count += 1;
        }

        char_count
    }

    /// Convert a byte range to character count
    fn byte_range_to_char_count(content: &str, start_byte: usize, end_byte: usize) -> usize {
        if start_byte >= end_byte || start_byte >= content.len() {
            return 0;
        }

        let safe_start = start_byte.min(content.len());
        let safe_end = end_byte.min(content.len());

        // Extract the slice and count characters
        if let Some(slice) = content.get(safe_start..safe_end) {
            slice.chars().count()
        } else {
            0
        }
    }

    // ENG-173: Phase 2 - Create unified coordinate mapping system with token boundary tracking
    pub fn create_coordinate_map<T: TextContent>(
        &self,
        content: T,
        cursor_position: usize,
        selection: Option<Range<usize>>,
    ) -> CoordinateMap {
        let content_str = content.text_to_string();
        let selection_range = selection;

        // Get tokens and their render modes
        let tokens = self.parser.parse_with_positions(&content_str);
        eprintln!("DEBUG COORD: Parsing content: {:?}", content_str);
        eprintln!("DEBUG COORD: Found {} tokens", tokens.len());
        let token_modes: Vec<(ParsedToken, TokenRenderMode)> = tokens
            .into_iter()
            .map(|token| {
                eprintln!(
                    "DEBUG COORD: Token {:?} at [{}, {}]",
                    token.token_type, token.start, token.end
                );
                let mode = TokenRenderMode::get_for_token(
                    &token,
                    cursor_position,
                    selection_range.clone(),
                );
                eprintln!(
                    "DEBUG COORD: Cursor at {}, token mode: {:?}",
                    cursor_position, mode
                );
                (token, mode)
            })
            .collect();

        // Initialize coordinate mapping structures
        let mut original_to_display = Vec::new();
        let mut display_to_original = Vec::new();
        let mut token_boundaries = Vec::new();
        let line_offsets: Vec<LineOffset> = { Vec::new() };

        let mut display_pos = 0;
        let mut original_pos = 0; // This tracks character positions
        let mut last_token_end = 0; // This tracks byte positions

        // Process content character by character, tracking token boundaries
        for (token, mode) in &token_modes {
            // Handle text before this token (convert byte positions to character positions)
            if token.start > last_token_end {
                let text_between = &content_str[last_token_end..token.start];
                let chars_between = text_between.chars().count();
                for _ in 0..chars_between {
                    original_to_display.push(display_pos);
                    display_to_original.push(original_pos);
                    display_pos += 1;
                    original_pos += 1;
                }
            }

            // Process the token based on its render mode
            let display_start = display_pos;
            let display_content = match (&token.token_type, &mode) {
                (MarkdownToken::Bold(text), TokenRenderMode::Preview)
                | (MarkdownToken::Italic(text), TokenRenderMode::Preview)
                | (MarkdownToken::Strikethrough(text), TokenRenderMode::Preview) => {
                    // In preview mode, hide markdown syntax
                    text.clone()
                }
                (MarkdownToken::Code(text), TokenRenderMode::Preview) => {
                    // Code in preview mode (no backticks)
                    text.clone()
                }
                (MarkdownToken::Heading(_, text), TokenRenderMode::Preview) => {
                    // Heading in preview mode (no # symbols)
                    eprintln!("DEBUG COORD: Heading in preview mode, text: {:?}", text);
                    text.clone()
                }
                _ => {
                    // Raw mode or unsupported token: show original
                    let original = content_str[token.start..token.end].to_string();
                    eprintln!(
                        "DEBUG COORD: Raw mode or unsupported, using original: {:?}",
                        original
                    );
                    original
                }
            };
            eprintln!(
                "DEBUG COORD: Display content for token: {:?}",
                display_content
            );

            // Map positions for this token - CRITICAL FIX: Convert byte range to character count
            let token_original_chars =
                Self::byte_range_to_char_count(&content_str, token.start, token.end);
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
                    original_to_display
                        .push(display_start + display_offset.min(display_chars.saturating_sub(1)));
                }

                // Map display positions back to original proportionally
                for i in 0..display_chars {
                    if display_chars == 0 {
                        display_to_original.push(original_pos);
                    } else {
                        let progress = i as f32 / display_chars as f32;
                        let original_offset = (progress * token_original_chars as f32) as usize;
                        display_to_original.push(
                            original_pos
                                + original_offset.min(token_original_chars.saturating_sub(1)),
                        );
                    }
                }

                display_pos += display_chars;
                original_pos += token_original_chars; // Advance by character count, not byte count
            } else {
                // Raw mode: 1:1 mapping
                for i in 0..token_original_chars {
                    original_to_display.push(display_pos + i);
                    display_to_original.push(original_pos + i); // Use character position + offset
                }
                display_pos += token_original_chars;
                original_pos += token_original_chars; // Advance by character count, not byte count
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

        // CRITICAL FIX: Always add a mapping for the position after the last character
        // This handles cursor positions at the end of tokens/content
        original_to_display.push(display_pos);
        if !display_to_original.is_empty() {
            display_to_original.push(original_pos);
        }

        eprintln!(
            "DEBUG COORD: Final coordinate map - {} original->display mappings",
            original_to_display.len()
        );

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
        selection: Option<Range<usize>>,
    ) -> usize {
        eprintln!(
            "DEBUG MAP_CURSOR: Mapping cursor position {} to display",
            original_cursor_pos
        );
        let coordinate_map = self.create_coordinate_map(content, original_cursor_pos, selection);
        eprintln!(
            "DEBUG MAP_CURSOR: Coordinate map has {} original->display mappings",
            coordinate_map.original_to_display.len()
        );
        if original_cursor_pos < coordinate_map.original_to_display.len() {
            let display_pos = coordinate_map.original_to_display[original_cursor_pos];
            eprintln!(
                "DEBUG MAP_CURSOR: Original pos {} maps to display pos {}",
                original_cursor_pos, display_pos
            );
            display_pos
        } else {
            let fallback = coordinate_map
                .original_to_display
                .last()
                .copied()
                .unwrap_or(0);
            eprintln!(
                "DEBUG MAP_CURSOR: Position {} out of bounds, using fallback {}",
                original_cursor_pos, fallback
            );
            fallback
        }
    }

    pub fn map_display_position_to_original<T: TextContent>(
        &self,
        content: T,
        display_position: usize,
        cursor_position: usize,
        selection: Option<(usize, usize)>,
    ) -> usize {
        let selection_range = selection.map(|(start, end)| start..end);
        let coordinate_map = self.create_coordinate_map(content, cursor_position, selection_range);

        if display_position < coordinate_map.display_to_original.len() {
            coordinate_map.display_to_original[display_position]
        } else {
            coordinate_map
                .display_to_original
                .last()
                .copied()
                .unwrap_or(0)
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
                    original_pos,
                    display_pos,
                    back_to_original
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
        let bold_boundary = map
            .token_boundaries
            .iter()
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
        assert_eq!(
            display_pos_6, 6,
            "Position at bold start should map correctly"
        );

        // 3. Inside bold markdown
        let display_pos_8 = mapper.map_cursor_position(content, 8, None); // Inside "**world**"
        let expected_8 = 8; // Position after "**world**" when bold token is in Raw mode
        assert_eq!(
            display_pos_8, expected_8,
            "Position inside bold should map correctly"
        );

        // 4. At end of bold markdown
        let display_pos_13 = mapper.map_cursor_position(content, 13, None); // After "**world**"
        let expected_13 = 13; // Position after "**world**" when bold token is in Raw mode
        assert_eq!(
            display_pos_13, expected_13,
            "Position after bold should map to end of transformed content"
        );

        // 5. Before italic markdown
        let display_pos_18 = mapper.map_cursor_position(content, 18, None); // At "*italic*"
        let expected_18 = 14; // Based on token boundary mapping: "Hello **world** " length when Bold is Raw
        assert_eq!(
            display_pos_18, expected_18,
            "Position at italic start should map correctly"
        );
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
    fn test_unicode_character_counting_in_coordinate_mapping() {
        let mapper = CoordinateMapper::new();
        // This is the exact content from the debug output that shows the issue
        let content = "Testing with unicode: 你好世界 🌍 émojis included";

        // First, let's understand what's happening
        let char_count = content.chars().count();
        let byte_count = content.len();
        println!("DEBUG: Content: {:?}", content);
        println!(
            "DEBUG: Character count: {}, byte count: {}",
            char_count, byte_count
        );

        // The real issue: markdown parser uses BYTE positions, but we need CHARACTER positions
        // Let's find the actual byte position where the cursor was in the debug output
        let emoji_byte_pos = content.find("🌍").unwrap();
        let emoji_char_pos = content
            .chars()
            .take_while(|&c| {
                content.as_ptr() as usize + content.find(c).unwrap_or(0)
                    < content.as_ptr() as usize + emoji_byte_pos
            })
            .count();

        println!(
            "DEBUG: Emoji '🌍' at byte position: {}, char position: {}",
            emoji_byte_pos, emoji_char_pos
        );

        // The issue from the debug output: position 44 was a BYTE position that mapped to display position 33
        // Let's test what actually happens when we get token positions from the parser
        let tokens = mapper.parser.parse_with_positions(content);
        println!("DEBUG: Tokens from parser:");
        for token in &tokens {
            let token_text = &content[token.start..token.end];
            println!(
                "  Token: {:?} at bytes {}..{} = {:?}",
                token.token_type, token.start, token.end, token_text
            );
        }

        // Test the problematic case: when parser gives us byte position 44
        // In the original debug, "Original pos 44 maps to display pos 33"
        // Position 44 in bytes should be somewhere after the emoji
        if byte_count > 44 {
            let byte_pos = 44;

            // Convert byte position to character position (this is what we need to fix)
            let char_pos = content
                .chars()
                .take_while(|&c| {
                    let byte_offset = content.as_ptr() as usize + content.find(c).unwrap_or(0);
                    byte_offset < content.as_ptr() as usize + byte_pos
                })
                .count();

            println!(
                "DEBUG: Byte position {} corresponds to character position {}",
                byte_pos, char_pos
            );

            // Test both: current (wrong) implementation treats byte pos as char pos
            let wrong_result = mapper.map_cursor_position(content, byte_pos, None); // Treating byte pos as char pos
            let correct_result = mapper.map_cursor_position(content, char_pos, None); // Using correct char pos

            println!(
                "DEBUG: Byte pos {} treated as char pos -> display pos {}",
                byte_pos, wrong_result
            );
            println!(
                "DEBUG: Correct char pos {} -> display pos {}",
                char_pos, correct_result
            );

            // This should expose the issue - they should be different if there's a byte/char mismatch
            assert_ne!(wrong_result, correct_result, "Treating byte position as character position should give different results for Unicode content");
        }

        // Basic sanity checks
        assert_ne!(
            char_count, byte_count,
            "Unicode content should have different byte and character counts"
        );
        assert!(
            char_count <= byte_count,
            "Character count should be <= byte count"
        );
    }

    #[test]
    fn test_mixed_unicode_content_coordinate_mapping() {
        let mapper = CoordinateMapper::new();

        // Test various Unicode content scenarios
        let test_cases = vec![
            // Simple ASCII (baseline)
            ("Hello world", 5, 5),
            // Chinese characters (3 bytes each)
            ("你好", 1, 1), // After first character
            // Mixed ASCII + Chinese
            ("Hi 你好", 4, 4), // After "Hi " and before Chinese
            // Emoji (4 bytes)
            ("🌍 world", 1, 1), // After emoji
            // Mixed content like the original issue
            ("Test 你好 🌍 text", 7, 7), // After "Test 你好 " (space after Chinese)
            ("Test 你好 🌍 text", 9, 9), // After emoji and space
        ];

        for (content, char_pos, expected_display_pos) in test_cases {
            println!("Testing: {:?} at char pos {}", content, char_pos);
            let display_pos = mapper.map_cursor_position(content, char_pos, None);
            assert_eq!(
                display_pos, expected_display_pos,
                "Failed for content {:?}: char pos {} should map to display pos {} (got {})",
                content, char_pos, expected_display_pos, display_pos
            );
        }
    }

    #[test]
    fn test_byte_to_char_conversion_utilities() {
        let content = "Test 你好 🌍 émojis";

        // Test the utility functions directly
        assert_eq!(CoordinateMapper::byte_to_char_position(content, 0), 0);
        assert_eq!(CoordinateMapper::byte_to_char_position(content, 5), 5); // After "Test "

        // Chinese characters start at byte 5, character 5
        let chinese_byte_pos = content.find("你").unwrap();
        let chinese_char_pos = CoordinateMapper::byte_to_char_position(content, chinese_byte_pos);
        assert_eq!(chinese_char_pos, 5);

        // Test byte range to character count
        let emoji_start = content.find("🌍").unwrap();
        let emoji_end = emoji_start + "🌍".len();
        let emoji_char_count =
            CoordinateMapper::byte_range_to_char_count(content, emoji_start, emoji_end);
        assert_eq!(
            emoji_char_count, 1,
            "Emoji should count as 1 character despite being 4 bytes"
        );

        // Test Chinese character range
        let chinese_range_start = content.find("你").unwrap();
        let chinese_range_end = chinese_range_start + "你好".len();
        let chinese_char_count = CoordinateMapper::byte_range_to_char_count(
            content,
            chinese_range_start,
            chinese_range_end,
        );
        assert_eq!(
            chinese_char_count, 2,
            "Two Chinese characters should count as 2 characters despite being 6 bytes"
        );
    }

    #[test]
    fn test_coordinate_consistency_round_trip() {
        let mapper = CoordinateMapper::new();
        let content = "Test **bold** text";

        // Test round-trip consistency for various positions
        for original in [0, 5, 6, 10, 13, 18] {
            let display = mapper.map_cursor_position(content, original, None);
            let back_to_original =
                mapper.map_display_position_to_original(content, display, original, None);

            // Should return to approximately the same position (within tolerance)
            let diff = (back_to_original as i32 - original as i32).abs();
            assert!(
                diff <= 2,
                "Round trip failed for position {}: {} -> {} -> {}, diff = {}",
                original,
                original,
                display,
                back_to_original,
                diff
            );
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
        assert_eq!(
            pos_in_italic, expected_italic,
            "Position in italic should map correctly"
        );
    }
}
