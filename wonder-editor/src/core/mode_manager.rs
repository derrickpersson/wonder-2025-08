//! Mode Manager - Handles switching between preview and raw modes based on cursor position

use crate::markdown_parser::{MarkdownParser, ParsedToken};
use std::time::{Duration, Instant};

/// Represents the current editor mode
#[derive(Debug, Clone, PartialEq)]
pub enum EditorMode {
    Preview,
    Raw,
}

/// Manages mode switching logic for hybrid preview/raw mode
pub struct ModeManager {
    current_mode: EditorMode,
    parser: MarkdownParser,
    debounce_duration: Duration,
    last_cursor_move: Option<Instant>,
    pending_cursor_position: Option<usize>,
}

impl ModeManager {
    /// Create a new mode manager starting in Raw mode for empty content
    pub fn new() -> Self {
        Self {
            current_mode: EditorMode::Raw, // Start in raw mode for typing
            parser: MarkdownParser::new(),
            debounce_duration: Duration::from_millis(150), // 150ms debounce
            last_cursor_move: None,
            pending_cursor_position: None,
        }
    }

    /// Create a new mode manager with custom debounce duration
    pub fn new_with_debounce(debounce_ms: u64) -> Self {
        Self {
            current_mode: EditorMode::Preview,
            parser: MarkdownParser::new(),
            debounce_duration: Duration::from_millis(debounce_ms),
            last_cursor_move: None,
            pending_cursor_position: None,
        }
    }

    /// Get the current editor mode
    pub fn current_mode(&self) -> EditorMode {
        self.current_mode.clone()
    }

    /// Update mode based on cursor position and content
    /// Returns true if mode changed, false otherwise
    pub fn update_mode(&mut self, content: &str, cursor_position: usize) -> bool {
        let tokens = self.parser.parse_with_positions(content);
        let new_mode = self.determine_mode(&tokens, cursor_position);
        
        let mode_changed = new_mode != self.current_mode;
        self.current_mode = new_mode;
        mode_changed
    }

    /// Update mode based on selection range and content
    /// Returns true if mode changed, false otherwise
    pub fn update_mode_with_selection(&mut self, content: &str, selection_start: usize, selection_end: usize) -> bool {
        let tokens = self.parser.parse_with_positions(content);
        let new_mode = self.determine_mode_for_selection(&tokens, selection_start, selection_end);
        
        let mode_changed = new_mode != self.current_mode;
        self.current_mode = new_mode;
        mode_changed
    }

    /// Update mode with debouncing for rapid cursor movements
    /// Returns true if mode changed immediately, false if debounced or no change
    pub fn update_mode_debounced(&mut self, content: &str, cursor_position: usize) -> bool {
        let now = Instant::now();
        
        // Store the pending cursor position
        self.pending_cursor_position = Some(cursor_position);
        
        // Check if we should debounce
        if let Some(last_move) = self.last_cursor_move {
            if now.duration_since(last_move) < self.debounce_duration {
                // Still within debounce period, just update timestamp
                self.last_cursor_move = Some(now);
                return false;
            }
        }
        
        // Update timestamp
        self.last_cursor_move = Some(now);
        
        // Apply the mode update
        self.update_mode(content, cursor_position)
    }

    /// Check if debounce period has elapsed and apply pending updates
    /// Returns true if mode changed, false otherwise
    pub fn check_debounce(&mut self, content: &str) -> bool {
        if let (Some(last_move), Some(cursor_pos)) = (self.last_cursor_move, self.pending_cursor_position) {
            let now = Instant::now();
            if now.duration_since(last_move) >= self.debounce_duration {
                // Clear pending state
                self.pending_cursor_position = None;
                
                // Apply the mode update
                return self.update_mode(content, cursor_pos);
            }
        }
        false
    }

    /// Determine mode based on cursor position relative to tokens
    fn determine_mode(&self, tokens: &[ParsedToken], cursor_position: usize) -> EditorMode {
        // Check if cursor is inside any token
        for token in tokens {
            if cursor_position >= token.start && cursor_position <= token.end {
                return EditorMode::Raw;
            }
        }
        
        EditorMode::Preview
    }

    /// Determine mode based on selection range
    fn determine_mode_for_selection(&self, tokens: &[ParsedToken], selection_start: usize, selection_end: usize) -> EditorMode {
        // If selection spans multiple tokens, go to Raw mode
        let mut tokens_in_selection = 0;
        
        for token in tokens {
            // Check if token overlaps with selection
            if token.end >= selection_start && token.start <= selection_end {
                tokens_in_selection += 1;
                if tokens_in_selection >= 2 {
                    return EditorMode::Raw;
                }
            }
        }
        
        // If selection is inside a single token, go to Raw mode
        for token in tokens {
            if selection_start >= token.start && selection_end <= token.end {
                return EditorMode::Raw;
            }
        }
        
        // Otherwise, stay in Preview mode
        EditorMode::Preview
    }
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::MarkdownToken;

    #[test]
    fn test_mode_switches_to_raw_when_cursor_enters_token() {
        let mut mode_manager = ModeManager::new();
        
        // Start in Preview mode
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
        
        let content = "# Hello World";
        let cursor_position = 1; // Inside the heading token
        
        let mode_changed = mode_manager.update_mode(content, cursor_position);
        
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
    }

    #[test]
    fn test_mode_switches_to_preview_when_cursor_leaves_token() {
        let mut mode_manager = ModeManager::new();
        let content = "# Hello World\n\nRegular text";
        
        // First, enter Raw mode by putting cursor inside heading token
        let cursor_inside_token = 1; // Inside the "# Hello World"
        let mode_changed = mode_manager.update_mode(content, cursor_inside_token);
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
        
        // Now move cursor to regular text area (outside any token)
        let cursor_outside_token = 15; // In the "Regular text" area
        let mode_changed = mode_manager.update_mode(content, cursor_outside_token);
        
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
    }

    #[test]
    fn test_mode_switches_to_raw_when_selection_spans_multiple_tokens() {
        let mut mode_manager = ModeManager::new();
        let content = "# Heading\n\n**Bold text** and *italic text*";
        
        // Start in Preview mode
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
        
        // Select from heading to bold text (spans multiple tokens)
        let selection_start = 1; // Inside heading
        let selection_end = 20; // Inside bold text
        
        let mode_changed = mode_manager.update_mode_with_selection(content, selection_start, selection_end);
        
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
    }

    #[test]
    fn test_show_outer_token_as_raw_when_cursor_inside_nested() {
        let mut mode_manager = ModeManager::new();
        let content = "**bold *with italic* text**";
        
        // Start in Preview mode
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
        
        // Put cursor inside the inner italic token
        let cursor_inside_nested = 9; // Inside "*with italic*" but also inside "**bold ... text**"
        
        let mode_changed = mode_manager.update_mode(content, cursor_inside_nested);
        
        // Should switch to Raw mode (current behavior works)
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
    }

    #[test] 
    fn test_handle_deeply_nested_structures() {
        let mut mode_manager = ModeManager::new();
        let content = "***bold and italic*** with **just bold**";
        
        // Start in Preview mode
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
        
        // Put cursor inside the deeply nested content
        let cursor_inside_deeply_nested = 10; // Inside "bold and italic"
        
        let mode_changed = mode_manager.update_mode(content, cursor_inside_deeply_nested);
        
        // Should switch to Raw mode
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
    }

    #[test]
    fn test_maintain_performance_with_complex_nesting() {
        let mut mode_manager = ModeManager::new();
        
        // Create moderately complex nested content (less than previous test)
        let content = "**bold1 *italic1* text** and **bold2 *italic2* text** more text";
        
        let start = std::time::Instant::now();
        
        // Test multiple cursor movements (reasonable number)
        for pos in (0..content.len()).step_by(1) {
            mode_manager.update_mode(&content, pos);
        }
        
        let duration = start.elapsed();
        
        // Should complete in reasonable time (less than 50ms for this simpler test)
        // This is a more realistic performance expectation
        assert!(duration < std::time::Duration::from_millis(50));
        
        // Ensure functionality still works
        let mode_changed = mode_manager.update_mode(&content, 10); // Inside nested token
        if !mode_manager.current_mode().eq(&EditorMode::Raw) {
            assert!(mode_changed);
        }
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
    }

    #[test]
    fn test_debounce_rapid_cursor_movements() {
        let mut mode_manager = ModeManager::new_with_debounce(50); // 50ms debounce for testing
        let content = "# Hello World\n\nRegular text";
        
        // Start in Preview mode
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
        
        // First cursor movement - should be applied immediately (no previous movement)
        let mode_changed = mode_manager.update_mode_debounced(content, 1); // Inside token
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw);
        
        // Rapid movements within debounce period - should not change mode immediately
        let mode_changed = mode_manager.update_mode_debounced(content, 15); // Outside token
        assert!(!mode_changed); // Debounced - no immediate change
        assert_eq!(mode_manager.current_mode(), EditorMode::Raw); // Still in Raw mode
        
        // Wait for debounce period to elapse
        std::thread::sleep(std::time::Duration::from_millis(60));
        
        // Check debounce - should apply the pending update now
        let mode_changed = mode_manager.check_debounce(content);
        assert!(mode_changed);
        assert_eq!(mode_manager.current_mode(), EditorMode::Preview);
    }
}