use crate::core::TextDocument;
use crate::input::{ActionHandler, InputRouter};

// Test helper that creates a minimal editor without GPUI context
// This is for testing core functionality that doesn't require focus handling
pub fn create_test_editor_minimal() -> TestableEditor {
    TestableEditor {
        document: TextDocument::new(),
        input_router: InputRouter::new(),
        focused: false,
        // ENG-142: Initialize scroll state
        scroll_offset: 0.0,
        // ENG-143: Initialize context menu state
        context_menu_visible: false,
        context_menu_position: None,
        simulated_clipboard_content: None,
    }
}

// Wrapper struct for testing that mimics MarkdownEditor without FocusHandle
#[derive(Debug)]
pub struct TestableEditor {
    document: TextDocument,
    input_router: InputRouter,
    focused: bool,
    // ENG-142: Scroll state for testing
    scroll_offset: f32,
    // ENG-143: Context menu state for testing
    context_menu_visible: bool,
    context_menu_position: Option<usize>,
    simulated_clipboard_content: Option<String>,
}

impl TestableEditor {
    // Mirror the methods we need for testing
    pub fn content(&self) -> String {
        self.document.content()
    }

    pub fn cursor_position(&self) -> usize {
        self.document.cursor_position()
    }

    pub fn handle_char_input(&mut self, ch: char) {
        self.input_router.handle_char_input(ch, &mut self.document);
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn get_content(&self) -> String {
        self.content()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn handle_key_input(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn document_mut(&mut self) -> &mut TextDocument {
        &mut self.document
    }

    // Convenience methods for easier testing
    pub fn has_selection(&self) -> bool {
        self.document.has_selection()
    }

    pub fn selected_text(&self) -> Option<String> {
        self.document.selected_text()
    }

    // ENG-137: Click-to-position functionality
    pub fn handle_click_at_position(&mut self, position: usize) -> bool {
        // Hide context menu on left-click
        self.context_menu_visible = false;
        self.context_menu_position = None;
        
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Set cursor to clicked position
        self.document.set_cursor_position(clamped_position);
        
        true // Return true to indicate successful handling
    }

    pub fn handle_click_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
        // In a real UI, we'd map display coordinates to source coordinates
        // For testing, we'll use a simple approach: assume 1:1 mapping for basic text
        let source_position = display_position;
        self.handle_click_at_position(source_position)
    }

    // ENG-138: Drag selection functionality for real UI
    pub fn handle_mouse_down_at_position(&mut self, position: usize) -> bool {
        // Hide context menu on left-click
        self.context_menu_visible = false;
        self.context_menu_position = None;
        
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Clear any existing selection and set cursor to clicked position
        self.document.clear_selection();
        self.document.set_cursor_position(clamped_position);
        
        // Note: We don't start selection immediately on mouse down
        // Selection starts when mouse moves (drag begins)
        
        true
    }

    pub fn handle_mouse_drag_to_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // If no selection is active, start one from current cursor position
        if !self.document.has_selection() {
            self.document.start_selection();
        }
        
        // Extend selection to the new position
        self.document.set_cursor_position(clamped_position);
        
        true
    }

    pub fn handle_mouse_up_at_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Finalize the selection at the release position
        self.document.set_cursor_position(clamped_position);
        
        // Selection remains active after mouse up
        true
    }

    pub fn handle_mouse_down_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
        // In a real UI, we'd map display coordinates to source coordinates
        // For testing, we'll use a simple approach: assume 1:1 mapping for basic text
        let source_position = display_position;
        self.handle_mouse_down_at_position(source_position)
    }

    pub fn handle_mouse_drag_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
        // In a real UI, we'd map display coordinates to source coordinates
        // For testing, we'll use a simple approach: assume 1:1 mapping for basic text
        let source_position = display_position;
        self.handle_mouse_drag_to_position(source_position)
    }

    pub fn handle_mouse_up_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
        // In a real UI, we'd map display coordinates to source coordinates
        // For testing, we'll use a simple approach: assume 1:1 mapping for basic text
        let source_position = display_position;
        self.handle_mouse_up_at_position(source_position)
    }

    // ENG-139: Double/triple-click selection (simplified for testing)
    pub fn handle_click_at_position_with_click_count(&mut self, position: usize, click_count: u32) -> bool {
        match click_count {
            1 => self.handle_click_at_position(position),
            2 => self.select_word_at_position(position),
            3 => self.select_line_at_position(position),
            _ => self.handle_click_at_position(position), // Fallback to single click
        }
    }

    pub fn select_word_at_position(&mut self, position: usize) -> bool {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find word boundaries around the clicked position
        let (word_start, word_end) = self.find_word_boundaries(&content, clamped_position);
        
        // Set selection to cover the word
        self.document.set_cursor_position(word_start);
        self.document.start_selection();
        self.document.set_cursor_position(word_end);
        
        true
    }

    pub fn select_line_at_position(&mut self, position: usize) -> bool {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find line boundaries around the clicked position
        let (line_start, line_end) = self.find_line_boundaries(&content, clamped_position);
        
        // Set selection to cover the line
        self.document.set_cursor_position(line_start);
        self.document.start_selection();
        self.document.set_cursor_position(line_end);
        
        true
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
    pub fn handle_shift_click_at_position(&mut self, position: usize) -> bool {
        if self.document.has_selection() {
            // If there's an existing selection, extend it from the original start point
            if let Some((start, _end)) = self.document.selection_range() {
                // Clear current selection and create new one from original start to clicked position
                self.document.clear_selection();
                self.document.set_cursor_position(start);
                self.document.start_selection();
                self.document.set_cursor_position(position);
            }
        } else {
            // No existing selection - create from cursor to clicked position
            self.document.start_selection();
            self.document.set_cursor_position(position);
        }
        
        true
    }
}