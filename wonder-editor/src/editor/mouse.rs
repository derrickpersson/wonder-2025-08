use crate::core::TextDocument;
use gpui::{
    px, Context, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Window,
};

use super::MarkdownEditor;

impl MarkdownEditor {
    // Mouse event handlers
    pub(super) fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Focus the editor when clicked
        window.focus(&self.focus_handle);
        self.focused = true;
        
        // ENG-137/138: Convert mouse coordinates to character position
        let character_position = self.convert_point_to_character_index(event.position);
        
        // ENG-140: Check for Shift modifier for selection extension
        if event.modifiers.shift {
            // Shift+click - extend selection (no drag support for Shift+click)
            self.handle_shift_click_at_position(character_position);
        } else {
            // ENG-139: Detect click count for double/triple-click selection
            let now = std::time::Instant::now();
            let click_count = self.calculate_click_count(character_position, now);
            
            // Handle different click types
            self.handle_click_with_count(character_position, click_count);
            
            // Set flag that we're potentially starting a drag operation (only for single clicks)
            if click_count == 1 {
                self.is_mouse_down = true;
                self.mouse_down_position = Some(character_position);
            }
        }
        
        cx.notify();
    }

    pub(super) fn handle_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // ENG-138: Handle mouse up for drag selection
        if self.is_mouse_down {
            let character_position = self.convert_point_to_character_index(event.position);
            self.handle_mouse_up_at_position(character_position);
            
            self.is_mouse_down = false;
            self.mouse_down_position = None;
            
            cx.notify();
        }
    }

    pub(super) fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // ENG-138: Handle mouse drag for selection
        if self.is_mouse_down {
            let character_position = self.convert_point_to_character_index(event.position);
            self.handle_mouse_drag_to_position(character_position);
            
            cx.notify();
        }
    }

    // Helper method to convert screen coordinates to character positions
    fn convert_point_to_character_index(&self, point: Point<Pixels>) -> usize {
        // Basic coordinate to character conversion
        // This mimics the logic in character_index_for_point but for direct use
        let padding = px(16.0);
        let line_height = px(24.0);
        
        // Account for editor bounds (status bar height + editor padding)
        let editor_content_y_offset = px(30.0) + padding; // Status bar height + padding
        let relative_y = point.y - editor_content_y_offset;
        let line_index = ((relative_y / line_height).floor() as usize).max(0);
        
        let content = self.document.content();
        let lines: Vec<&str> = content.lines().collect();
        
        // If clicked beyond last line, return end of document
        if line_index >= lines.len() {
            return content.chars().count();
        }
        
        // Calculate character position within the clicked line
        let line_content = lines[line_index];
        let relative_x = point.x - padding;
        
        // Simple character width approximation (will be improved with proper text measurement)
        let font_size = px(16.0);
        let approx_char_width = font_size * 0.6;
        let char_index_in_line = ((relative_x / approx_char_width).floor() as usize).min(line_content.chars().count());
        
        // Calculate absolute position in document
        let chars_before_line: usize = lines.iter().take(line_index).map(|l| l.chars().count() + 1).sum();
        let absolute_position = chars_before_line + char_index_in_line;
        
        absolute_position.min(content.chars().count())
    }

    // ENG-139: Click count and selection helpers
    fn calculate_click_count(&mut self, position: usize, now: std::time::Instant) -> u32 {
        const DOUBLE_CLICK_TIME: std::time::Duration = std::time::Duration::from_millis(500);
        const CLICK_POSITION_TOLERANCE: usize = 3; // Allow small position differences
        
        // Check if this is a rapid click at same position
        let is_rapid_click = now.duration_since(self.last_click_time) <= DOUBLE_CLICK_TIME;
        let is_same_position = self.last_click_position
            .map(|last_pos| position.abs_diff(last_pos) <= CLICK_POSITION_TOLERANCE)
            .unwrap_or(false);
        
        let click_count = if is_rapid_click && is_same_position {
            // Determine if this is the second, third, etc. click
            // For simplicity, we'll detect double-click vs triple-click based on timing
            if now.duration_since(self.last_click_time) <= std::time::Duration::from_millis(250) {
                3 // Very rapid = triple click
            } else {
                2 // Moderately rapid = double click
            }
        } else {
            1 // Single click
        };
        
        // Update tracking
        self.last_click_time = now;
        self.last_click_position = Some(position);
        
        click_count
    }

    fn handle_click_with_count(&mut self, position: usize, click_count: u32) {
        match click_count {
            1 => {
                // Single click - position cursor
                self.handle_mouse_down_at_position(position);
            },
            2 => {
                // Double click - select word at position
                self.select_word_at_position(position);
            },
            3 => {
                // Triple click - select line at position  
                self.select_line_at_position(position);
            },
            _ => {
                // Fall back to single click for higher counts
                self.handle_mouse_down_at_position(position);
            }
        }
    }

    fn select_word_at_position(&mut self, position: usize) {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find word boundaries around the clicked position
        let (word_start, word_end) = self.find_word_boundaries(&content, clamped_position);
        
        // Set selection to cover the word
        self.document.set_cursor_position(word_start);
        self.document.start_selection();
        self.document.set_cursor_position(word_end);
    }

    fn select_line_at_position(&mut self, position: usize) {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find line boundaries around the clicked position
        let (line_start, line_end) = self.find_line_boundaries(&content, clamped_position);
        
        // Set selection to cover the line
        self.document.set_cursor_position(line_start);
        self.document.start_selection();
        self.document.set_cursor_position(line_end);
    }

    fn find_word_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        if chars.is_empty() || position >= chars.len() {
            return (0, 0);
        }
        
        // Find start of word (move left while character is word-like)
        let mut start = position;
        while start > 0 && self.is_word_char(chars[start.saturating_sub(1)]) {
            start -= 1;
        }
        
        // Find end of word (move right while character is word-like)
        let mut end = position;
        while end < chars.len() && self.is_word_char(chars[end]) {
            end += 1;
        }
        
        (start, end)
    }

    fn find_line_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        if chars.is_empty() {
            return (0, 0);
        }
        
        let clamped_pos = position.min(chars.len());
        
        // Find start of line (move left until newline or beginning)
        let mut start = clamped_pos;
        while start > 0 && chars[start - 1] != '\n' {
            start -= 1;
        }
        
        // Find end of line (move right until newline or end)
        let mut end = clamped_pos;
        while end < chars.len() && chars[end] != '\n' {
            end += 1;
        }
        
        (start, end)
    }

    fn is_word_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    // ENG-140: Shift+click selection extension
    fn handle_shift_click_at_position(&mut self, position: usize) {
        if self.document.has_selection() {
            // If there's an existing selection, extend it from the original start point
            let (start, _end) = self.document.selection_range().unwrap();
            // Clear current selection and create new one from original start to clicked position
            self.document.clear_selection();
            self.document.set_cursor_position(start);
            self.document.start_selection();
            self.document.set_cursor_position(position);
        } else {
            // No existing selection - create from cursor to clicked position
            self.document.start_selection();
            self.document.set_cursor_position(position);
        }
    }

    // Public API for mouse interactions
    pub fn handle_click_at_position(&mut self, position: usize) -> bool {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        self.document.set_cursor_position(clamped_position);
        self.document.clear_selection();
        
        true // Successfully handled
    }

    pub fn handle_mouse_down_at_position(&mut self, position: usize) -> bool {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        self.document.set_cursor_position(clamped_position);
        self.document.clear_selection();
        
        true // Successfully handled
    }

    pub fn handle_mouse_drag_to_position(&mut self, position: usize) -> bool {
        if let Some(start_pos) = self.mouse_down_position {
            let content = self.document.content();
            let max_pos = content.chars().count();
            let clamped_position = position.min(max_pos);
            
            // Create selection from start_pos to current position
            let (selection_start, selection_end) = if start_pos <= clamped_position {
                (start_pos, clamped_position)
            } else {
                (clamped_position, start_pos)
            };
            
            self.document.set_cursor_position(selection_start);
            self.document.start_selection();
            self.document.set_cursor_position(selection_end);
            
            true
        } else {
            false
        }
    }

    pub fn handle_mouse_up_at_position(&mut self, position: usize) -> bool {
        if self.mouse_down_position.is_some() {
            // Finalize any drag selection
            self.handle_mouse_drag_to_position(position);
            true
        } else {
            false
        }
    }
}