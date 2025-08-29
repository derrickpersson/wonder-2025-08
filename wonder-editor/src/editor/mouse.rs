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
    // Comprehensive fix for mouse positioning accuracy with hybrid rendering
    fn convert_point_to_character_index(&self, point: Point<Pixels>) -> usize {
        let padding = px(16.0);
        
        // Account for editor bounds (status bar height + editor padding)
        let editor_content_y_offset = px(30.0) + padding; // Status bar height + padding
        let relative_y = point.y - editor_content_y_offset;
        let relative_x = point.x - padding;
        
        let content = self.document.content();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return 0;
        }
        
        // Find the correct line by accounting for different line heights based on content
        let mut current_y_offset = px(0.0);
        let mut line_index = 0;
        let mut chars_before_line = 0;
        
        for (idx, line_content) in lines.iter().enumerate() {
            // Calculate line height based on content (headings have larger line heights)
            let line_height = self.calculate_line_height_for_content(line_content, chars_before_line);
            
            // Check if click is within this line's bounds
            if relative_y >= current_y_offset && relative_y < current_y_offset + line_height {
                line_index = idx;
                break;
            }
            
            current_y_offset += line_height;
            chars_before_line += line_content.chars().count() + 1; // +1 for newline
            
            // If we're past the last line, use the last line
            if idx == lines.len() - 1 {
                line_index = idx;
            }
        }
        
        // If clicked beyond all lines, return end of document
        if line_index >= lines.len() {
            return content.chars().count();
        }
        
        let line_content = lines[line_index];
        
        // Calculate character position within the clicked line using more accurate text measurement
        let char_offset_in_line = self.calculate_character_offset_from_x_position(
            line_content, 
            relative_x, 
            chars_before_line
        );
        
        let original_position = chars_before_line + char_offset_in_line;
        original_position.min(content.chars().count())
    }
    
    // Calculate line height based on content (headings are taller)
    fn calculate_line_height_for_content(&self, line_content: &str, line_start_pos: usize) -> Pixels {
        // Default line height
        let base_line_height = px(24.0);
        
        // Check if this line is a heading and calculate appropriate height
        if line_content.starts_with('#') {
            let heading_level = line_content.chars().take_while(|&c| c == '#').count() as u32;
            if heading_level <= 6 {
                // Get the font size for this heading level 
                let base_font_size = 16.0;
                let heading_font_size = self.hybrid_renderer.get_scalable_font_size_for_heading_level(heading_level, base_font_size);
                let heading_line_height = self.hybrid_renderer.get_line_height_for_font_size(heading_font_size);
                return px(heading_line_height);
            }
        }
        
        base_line_height
    }
    
    // Calculate character offset from X position with better accuracy
    fn calculate_character_offset_from_x_position(&self, line_content: &str, x_position: Pixels, line_start_pos: usize) -> usize {
        if line_content.is_empty() {
            return 0;
        }
        
        // Use binary search to find the character position that best matches the X coordinate
        let mut best_offset = 0;
        let mut min_distance = f32::MAX;
        
        for char_offset in 0..=line_content.chars().count() {
            let estimated_x = self.estimate_x_position_for_character_offset(line_content, char_offset, line_start_pos);
            let distance = (estimated_x.0 - x_position.0).abs();
            
            if distance < min_distance {
                min_distance = distance;
                best_offset = char_offset;
            }
        }
        
        best_offset
    }
    
    // Estimate X position for a character offset in a line
    fn estimate_x_position_for_character_offset(&self, line_content: &str, char_offset: usize, line_start_pos: usize) -> Pixels {
        if char_offset == 0 {
            return px(0.0);
        }
        
        let text_up_to_offset: String = line_content.chars().take(char_offset).collect();
        
        // Determine font size based on content type (heading vs regular text)
        let base_font_size = 16.0;
        let font_size = if line_content.starts_with('#') {
            let heading_level = line_content.chars().take_while(|&c| c == '#').count() as u32;
            if heading_level <= 6 {
                self.hybrid_renderer.get_scalable_font_size_for_heading_level(heading_level, base_font_size)
            } else {
                base_font_size
            }
        } else {
            base_font_size
        };
        
        // Use a more accurate character width estimation based on font size
        // This is still an approximation but accounts for different font sizes
        let char_width = font_size * 0.6; // Slightly better approximation than before
        
        px(text_up_to_offset.chars().count() as f32 * char_width)
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