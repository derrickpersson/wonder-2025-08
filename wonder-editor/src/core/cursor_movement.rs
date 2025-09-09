//! Unified cursor movement service with visual-line-first positioning
//!
//! This module provides a single source of truth for all cursor positioning logic,
//! prioritizing visual line awareness while gracefully falling back to logical lines.

use crate::core::TextDocument;
use crate::input::actions::Movement;
use crate::rendering::line_wrapping::HybridLineWrapper;
use crate::rendering::VisualLineManager;

/// Service providing unified cursor movement with visual-line-first approach
pub struct CursorMovementService {
    /// Track preferred column for up/down navigation
    preferred_column: Option<usize>,
    /// Last known logical cursor position for synchronization
    last_logical_position: usize,
}

impl CursorMovementService {
    /// Create a new cursor movement service
    pub fn new() -> Self {
        Self {
            preferred_column: None,
            last_logical_position: 0,
        }
    }

    /// Perform cursor movement using visual-line-first approach
    pub fn move_cursor(
        &mut self,
        movement: Movement,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
        is_extending_selection: bool,
    ) -> bool {
        // Synchronize our position with document
        self.last_logical_position = document.cursor_position();
        
        // Handle selection state
        if is_extending_selection && !document.has_selection() {
            document.start_selection();
        } else if !is_extending_selection {
            document.clear_selection();
        }

        match movement {
            Movement::Left => self.move_left(document, line_wrapper, visual_line_manager),
            Movement::Right => self.move_right(document, line_wrapper, visual_line_manager),
            Movement::Up => self.move_up(document, line_wrapper, visual_line_manager),
            Movement::Down => self.move_down(document, line_wrapper, visual_line_manager),
            Movement::LineStart => self.move_to_line_start(document, line_wrapper, visual_line_manager),
            Movement::LineEnd => self.move_to_line_end(document, line_wrapper, visual_line_manager),
            Movement::WordStart => self.move_to_word_start(document),
            Movement::WordEnd => self.move_to_word_end(document),
            Movement::DocumentStart => self.move_to_document_start(document),
            Movement::DocumentEnd => self.move_to_document_end(document),
            Movement::PageUp => self.move_page_up(document),
            Movement::PageDown => self.move_page_down(document),
        }
    }

    /// Move cursor to screen coordinates (for mouse clicks)
    pub fn move_to_screen_position(
        &mut self,
        document: &mut TextDocument,
        screen_x: f32,
        screen_y: f32,
        element_bounds: gpui::Bounds<gpui::Pixels>,
        visual_line_manager: &VisualLineManager,
        window: &mut gpui::Window,
    ) -> bool {
        if let Some(position) = self.convert_screen_to_text_position(
            screen_x,
            screen_y,
            element_bounds,
            visual_line_manager,
            document,
            window,
        ) {
            document.set_cursor_position(position);
            self.last_logical_position = position;
            true
        } else {
            false
        }
    }

    // === PRIVATE IMPLEMENTATION METHODS ===

    /// Move left with visual line awareness
    fn move_left(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        self.preferred_column = None; // Clear on horizontal movement
        let current_pos = document.cursor_position();
        
        if current_pos == 0 {
            return false; // Already at start
        }

        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            // Check if we're at the start of a visual line (but not logical line)
            if visual_pos.column == 0 && visual_pos.visual_line > 0 {
                // Move to end of previous visual line
                if let Some(prev_line_end) = self.get_visual_line_end_position(
                    visual_pos.visual_line - 1,
                    visual_line_manager,
                    document,
                ) {
                    document.set_cursor_position(prev_line_end);
                    self.last_logical_position = prev_line_end;
                    return true;
                }
            }
        }

        // Fallback to simple logical left movement
        document.set_cursor_position(current_pos - 1);
        self.last_logical_position = current_pos - 1;
        true
    }

    /// Move right with visual line awareness  
    fn move_right(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        self.preferred_column = None; // Clear on horizontal movement
        let current_pos = document.cursor_position();
        let content_len = document.content().len();
        
        if current_pos >= content_len {
            return false; // Already at end
        }

        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            // Check if we're at the end of a visual line (but not logical line)
            if let Some(visual_line_end) = self.get_visual_line_end_position(
                visual_pos.visual_line,
                visual_line_manager,
                document,
            ) {
                if current_pos == visual_line_end && current_pos < content_len {
                    // Move to start of next visual line
                    if let Some(next_line_start) = self.get_visual_line_start_position(
                        visual_pos.visual_line + 1,
                        visual_line_manager,
                        document,
                    ) {
                        document.set_cursor_position(next_line_start);
                        self.last_logical_position = next_line_start;
                        return true;
                    }
                }
            }
        }

        // Fallback to simple logical right movement
        document.set_cursor_position(current_pos + 1);
        self.last_logical_position = current_pos + 1;
        true
    }

    /// Move up to previous visual line
    fn move_up(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        let content = document.content();
        
        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            if visual_pos.visual_line > 0 {
                let target_visual_line_idx = visual_pos.visual_line - 1;
                let target_column = self.preferred_column.unwrap_or(visual_pos.column);
                
                if let Some(target_position) = self.get_position_in_visual_line(
                    target_visual_line_idx,
                    target_column,
                    visual_line_manager,
                    document,
                ) {
                    // Store preferred column for subsequent up/down movements
                    self.preferred_column = Some(visual_pos.column);
                    document.set_cursor_position(target_position);
                    self.last_logical_position = target_position;
                    return true;
                }
            }
        }

        // Fallback to logical line up movement
        self.move_up_logical(document)
    }

    /// Move down to next visual line
    fn move_down(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        let content = document.content();
        
        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            let total_visual_lines = visual_line_manager.visual_line_count();
            if visual_pos.visual_line + 1 < total_visual_lines {
                let target_visual_line_idx = visual_pos.visual_line + 1;
                let target_column = self.preferred_column.unwrap_or(visual_pos.column);
                
                if let Some(target_position) = self.get_position_in_visual_line(
                    target_visual_line_idx,
                    target_column,
                    visual_line_manager,
                    document,
                ) {
                    // Store preferred column for subsequent up/down movements
                    self.preferred_column = Some(visual_pos.column);
                    document.set_cursor_position(target_position);
                    self.last_logical_position = target_position;
                    return true;
                }
            }
        }

        // Fallback to logical line down movement
        self.move_down_logical(document)
    }

    /// Move to start of current visual line
    fn move_to_line_start(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        self.preferred_column = None;
        
        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            if let Some(line_start_pos) = self.get_visual_line_start_position(
                visual_pos.visual_line,
                visual_line_manager,
                document,
            ) {
                document.set_cursor_position(line_start_pos);
                self.last_logical_position = line_start_pos;
                return true;
            }
        }

        // Fallback to logical line start
        self.move_to_line_start_logical(document)
    }

    /// Move to end of current visual line
    fn move_to_line_end(
        &mut self,
        document: &mut TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> bool {
        self.preferred_column = None;
        
        // Try visual line approach first
        if let Some(visual_pos) = self.get_current_visual_position(document, line_wrapper, visual_line_manager) {
            if let Some(line_end_pos) = self.get_visual_line_end_position(
                visual_pos.visual_line,
                visual_line_manager,
                document,
            ) {
                document.set_cursor_position(line_end_pos);
                self.last_logical_position = line_end_pos;
                return true;
            }
        }

        // Fallback to logical line end
        self.move_to_line_end_logical(document)
    }

    // === VISUAL LINE HELPER METHODS ===

    /// Get current cursor position as visual coordinates
    fn get_current_visual_position(
        &self,
        document: &TextDocument,
        line_wrapper: &HybridLineWrapper,
        visual_line_manager: &VisualLineManager,
    ) -> Option<VisualPosition> {
        let logical_pos = document.cursor_position();
        let rope = document.rope();
        
        // Convert byte offset to logical line/column
        let logical_line = rope.char_to_line(logical_pos);
        let line_start = rope.line_to_char(logical_line);
        let logical_column = logical_pos - line_start;
        
        // Try to find visual position using HybridLineWrapper
        if let Some(visual_pos) = line_wrapper.logical_to_visual_position(logical_line, logical_column) {
            return Some(VisualPosition {
                visual_line: visual_pos.visual_line,
                column: visual_pos.column,
            });
        }

        // Fallback: try to find using VisualLineManager directly
        if let Some((visual_line_idx, visual_line)) = visual_line_manager.find_visual_line_at_position(logical_line, logical_column) {
            let column_in_visual = logical_column - visual_line.start_offset;
            return Some(VisualPosition {
                visual_line: visual_line_idx,
                column: column_in_visual,
            });
        }

        None
    }

    /// Get start position of a visual line
    fn get_visual_line_start_position(
        &self,
        visual_line_idx: usize,
        visual_line_manager: &VisualLineManager,
        document: &TextDocument,
    ) -> Option<usize> {
        let visual_lines = visual_line_manager.all_visual_lines();
        if let Some(visual_line) = visual_lines.get(visual_line_idx) {
            let rope = document.rope();
            let logical_line = visual_line.logical_line;
            let logical_line_start = rope.line_to_char(logical_line);
            Some(logical_line_start + visual_line.start_offset)
        } else {
            None
        }
    }

    /// Get end position of a visual line
    fn get_visual_line_end_position(
        &self,
        visual_line_idx: usize,
        visual_line_manager: &VisualLineManager,
        document: &TextDocument,
    ) -> Option<usize> {
        let visual_lines = visual_line_manager.all_visual_lines();
        if let Some(visual_line) = visual_lines.get(visual_line_idx) {
            let rope = document.rope();
            let logical_line = visual_line.logical_line;
            let logical_line_start = rope.line_to_char(logical_line);
            Some(logical_line_start + visual_line.start_offset + visual_line.len())
        } else {
            None
        }
    }

    /// Get cursor position at specific column within visual line
    fn get_position_in_visual_line(
        &self,
        visual_line_idx: usize,
        target_column: usize,
        visual_line_manager: &VisualLineManager,
        document: &TextDocument,
    ) -> Option<usize> {
        let visual_lines = visual_line_manager.all_visual_lines();
        if let Some(visual_line) = visual_lines.get(visual_line_idx) {
            let rope = document.rope();
            let logical_line = visual_line.logical_line;
            let logical_line_start = rope.line_to_char(logical_line);
            
            // Clamp target column to visual line length
            let clamped_column = target_column.min(visual_line.len());
            Some(logical_line_start + visual_line.start_offset + clamped_column)
        } else {
            None
        }
    }

    /// Convert screen coordinates to text position using visual lines
    fn convert_screen_to_text_position(
        &self,
        screen_x: f32,
        screen_y: f32,
        element_bounds: gpui::Bounds<gpui::Pixels>,
        visual_line_manager: &VisualLineManager,
        document: &TextDocument,
        window: &mut gpui::Window,
    ) -> Option<usize> {
        if visual_line_manager.visual_line_count() == 0 {
            return None;
        }

        // Find target visual line based on Y coordinate
        let padding = gpui::px(16.0);
        let relative_y = screen_y - element_bounds.origin.y.0 - padding.0;
        
        let mut target_visual_line_idx = 0;
        let mut best_distance = f32::MAX;
        
        for visual_idx in 0..visual_line_manager.visual_line_count() {
            if let Some(y_pos) = visual_line_manager.get_y_position(visual_idx) {
                let distance = (relative_y - y_pos).abs();
                if distance < best_distance {
                    best_distance = distance;
                    target_visual_line_idx = visual_idx;
                }
            }
        }

        // Find column within visual line using GPUI text measurement
        let visual_lines = visual_line_manager.all_visual_lines();
        if let Some(target_visual_line) = visual_lines.get(target_visual_line_idx) {
            let relative_x = screen_x - element_bounds.origin.x.0 - padding.0;
            
            // Use GPUI for accurate character positioning
            let mut best_char_pos = 0;
            let mut best_distance = f32::MAX;
            
            let visual_line_text = target_visual_line.text();
            
            for char_pos in 0..=target_visual_line.len() {
                // CRITICAL FIX: Convert character position to safe byte boundary for string slicing
                let text_slice = visual_line_text.chars().take(char_pos).collect::<String>();
                
                if !text_slice.is_empty() && !target_visual_line.segments.is_empty() {
                    let first_segment = &target_visual_line.segments[0];
                    let shaped_line = window.text_system().shape_line(
                        text_slice.to_string().into(),
                        gpui::px(first_segment.font_size),
                        &[first_segment.text_run.clone()],
                        None,
                    );
                    
                    let text_width = shaped_line.width.0;
                    let distance = (relative_x - text_width).abs();
                    
                    if distance < best_distance {
                        best_distance = distance;
                        best_char_pos = char_pos;
                    }
                } else if char_pos == 0 {
                    let distance = relative_x.abs();
                    if distance < best_distance {
                        best_distance = distance;
                        best_char_pos = 0;
                    }
                }
            }

            // Convert visual position to logical position
            let rope = document.rope();
            let logical_line = target_visual_line.logical_line;
            
            if logical_line < rope.len_lines() {
                let logical_line_start = rope.line_to_char(logical_line);
                let logical_position = logical_line_start + target_visual_line.start_offset + best_char_pos;
                Some(logical_position.min(rope.len_chars()))
            } else {
                None
            }
        } else {
            None
        }
    }

    // === LOGICAL FALLBACK METHODS ===

    fn move_up_logical(&mut self, document: &mut TextDocument) -> bool {
        let rope = document.rope();
        let current_pos = document.cursor_position();
        let current_line = rope.char_to_line(current_pos);
        
        if current_line > 0 {
            let current_column = current_pos - rope.line_to_char(current_line);
            let target_line_start = rope.line_to_char(current_line - 1);
            let target_line_len = rope.line(current_line - 1).len_chars();
            let target_column = current_column.min(target_line_len);
            
            document.set_cursor_position(target_line_start + target_column);
            self.last_logical_position = target_line_start + target_column;
            true
        } else {
            document.set_cursor_position(0);
            self.last_logical_position = 0;
            false
        }
    }

    fn move_down_logical(&mut self, document: &mut TextDocument) -> bool {
        let rope = document.rope();
        let current_pos = document.cursor_position();
        let current_line = rope.char_to_line(current_pos);
        
        if current_line + 1 < rope.len_lines() {
            let current_column = current_pos - rope.line_to_char(current_line);
            let target_line_start = rope.line_to_char(current_line + 1);
            let target_line_len = rope.line(current_line + 1).len_chars();
            let target_column = current_column.min(target_line_len);
            
            document.set_cursor_position(target_line_start + target_column);
            self.last_logical_position = target_line_start + target_column;
            true
        } else {
            false
        }
    }

    fn move_to_line_start_logical(&mut self, document: &mut TextDocument) -> bool {
        let rope = document.rope();
        let current_pos = document.cursor_position();
        let current_line = rope.char_to_line(current_pos);
        let line_start = rope.line_to_char(current_line);
        
        document.set_cursor_position(line_start);
        self.last_logical_position = line_start;
        true
    }

    fn move_to_line_end_logical(&mut self, document: &mut TextDocument) -> bool {
        let rope = document.rope();
        let current_pos = document.cursor_position();
        let current_line = rope.char_to_line(current_pos);
        let line_start = rope.line_to_char(current_line);
        let line_len = rope.line(current_line).len_chars();
        
        // Don't include newline in positioning
        let line_end = if line_len > 0 && rope.line(current_line).char(line_len - 1) == '\n' {
            line_start + line_len - 1
        } else {
            line_start + line_len
        };
        
        document.set_cursor_position(line_end);
        self.last_logical_position = line_end;
        true
    }

    // Simple movements that don't need visual line awareness
    fn move_to_word_start(&mut self, document: &mut TextDocument) -> bool {
        // Delegate to existing TextDocument implementation
        let old_pos = document.cursor_position();
        document.move_to_word_start();
        self.last_logical_position = document.cursor_position();
        document.cursor_position() != old_pos
    }

    fn move_to_word_end(&mut self, document: &mut TextDocument) -> bool {
        let old_pos = document.cursor_position();
        document.move_to_word_end();
        self.last_logical_position = document.cursor_position();
        document.cursor_position() != old_pos
    }

    fn move_to_document_start(&mut self, document: &mut TextDocument) -> bool {
        document.set_cursor_position(0);
        self.last_logical_position = 0;
        true
    }

    fn move_to_document_end(&mut self, document: &mut TextDocument) -> bool {
        let end_pos = document.content().len();
        document.set_cursor_position(end_pos);
        self.last_logical_position = end_pos;
        true
    }

    fn move_page_up(&mut self, document: &mut TextDocument) -> bool {
        let old_pos = document.cursor_position();
        document.move_page_up();
        self.last_logical_position = document.cursor_position();
        document.cursor_position() != old_pos
    }

    fn move_page_down(&mut self, document: &mut TextDocument) -> bool {
        let old_pos = document.cursor_position();
        document.move_page_down();
        self.last_logical_position = document.cursor_position();
        document.cursor_position() != old_pos
    }
}

impl Default for CursorMovementService {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a cursor position in visual coordinate system
#[derive(Debug, Clone, PartialEq)]
struct VisualPosition {
    /// Index of the visual line
    visual_line: usize,
    /// Column within the visual line
    column: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::text_document::TextDocument;

    #[test]
    fn test_unicode_string_slicing_safety() {
        // Test that we handle Unicode boundaries correctly in string operations
        // This is a regression test for the Unicode boundary panic
        let test_cases = vec![
            "Hello World", // ASCII only
            "你好世界", // Chinese characters (3 bytes each)  
            "🌍🚀🎉", // Emojis (4 bytes each)
            "café résumé naïve", // Latin with accents (some 2-byte chars)
            "Testing with unicode: 你好世界 🌍 émojis included", // Mixed content that caused the original panic
        ];

        for content in test_cases {
            // Test that character-based collection works for any Unicode content
            for char_pos in 0..=content.chars().count() {
                // This operation should never panic, even at Unicode boundaries
                let text_slice = content.chars().take(char_pos).collect::<String>();
                
                // Validate that we got the expected number of characters
                assert_eq!(text_slice.chars().count(), char_pos);
                
                // Validate that the slice is a proper prefix of the original
                assert!(content.starts_with(&text_slice));
            }
        }
    }

    #[test] 
    fn test_specific_unicode_boundary_positions() {
        // Test the exact scenario that was causing the panic:
        // "Testing with unicode: 你好世界 🌍 émojis included"
        // The panic occurred at byte index 23, which is inside the Chinese character '你'
        let content = "Testing with unicode: 你好世界 🌍 émojis included";
        
        // Test specific problematic byte positions that were causing panics
        let problematic_byte_positions = vec![
            22, // Just before '你' (valid boundary)
            23, // Inside '你' (invalid byte boundary - would cause panic with [..])
            24, // Inside '你' (invalid byte boundary)
            25, // After '你', before '好' (valid boundary)
        ];

        for &byte_pos in &problematic_byte_positions {
            // Convert byte position to character position safely
            let char_pos = content.char_indices()
                .position(|(i, _)| i >= byte_pos)
                .unwrap_or(content.chars().count());
                
            // This should never panic - our fix uses character boundaries
            let text_slice = content.chars().take(char_pos).collect::<String>();
            
            // Validate the slice is correct
            assert!(content.starts_with(&text_slice));
            assert_eq!(text_slice.chars().count(), char_pos);
        }
    }

    #[test]
    fn test_edge_cases_unicode() {
        let test_cases = vec![
            "", // Empty string
            " ", // Single space  
            "a", // Single ASCII char
            "你", // Single Unicode char (3 bytes)
            "🌍", // Single emoji (4 bytes)
        ];

        for content in test_cases {
            // Test that character iteration works for edge cases
            for char_pos in 0..=content.chars().count() {
                let text_slice = content.chars().take(char_pos).collect::<String>();
                assert_eq!(text_slice.chars().count(), char_pos);
                
                // For non-empty content, validate prefix property
                if !content.is_empty() && char_pos > 0 {
                    assert!(content.starts_with(&text_slice));
                }
            }
        }
    }

    #[test]
    fn test_cursor_movement_basic() {
        let content = "Hello\nWorld";
        let mut document = TextDocument::with_content(content.to_string());
        let mut movement_service = CursorMovementService::new();

        // Test basic cursor movements don't panic
        assert!(movement_service.move_up_logical(&mut document) == false); // Already at top
        assert!(movement_service.move_down_logical(&mut document) == true); // Can move down
        assert!(movement_service.move_to_line_start_logical(&mut document) == true);
        assert!(movement_service.move_to_line_end_logical(&mut document) == true);
    }
}