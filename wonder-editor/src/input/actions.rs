//! Editor actions and command system
//! 
//! This module defines all possible editor actions that can be triggered
//! by keyboard shortcuts, menu items, or other user interactions.

#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    // Text insertion and deletion
    InsertChar(char),
    InsertText(String),
    Backspace,
    Delete,
    
    // Advanced deletion operations
    DeletePreviousWord,
    DeleteNextWord,
    DeleteToLineStart,
    DeleteToLineEnd,
    DeleteCurrentLine,
    
    // Cursor movement
    MoveCursor(Movement),
    
    // Selection operations
    ExtendSelection(Movement),
    SelectAll,
    ClearSelection,
    
    // Text formatting
    ToggleFormat(FormatType),
    
    // Navigation
    MoveToPosition(usize),
    PageUp,
    PageDown,
    
    // Clipboard operations
    Copy,
    Cut,
    Paste,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Movement {
    // Character-level movement
    Left,
    Right,
    Up,
    Down,
    
    // Word-level movement
    WordStart,
    WordEnd,
    
    // Line-level movement
    LineStart,
    LineEnd,
    
    // Document-level movement
    DocumentStart,
    DocumentEnd,
    
    // Page-level movement
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatType {
    Bold,
    Italic,
    Code,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteDirection {
    Backward, // Backspace
    Forward,  // Delete
}

// Trait for handling editor actions
pub trait ActionHandler {
    /// Execute an editor action and return whether it was handled successfully
    fn handle_action(&mut self, action: EditorAction) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let action = EditorAction::MoveCursor(Movement::Left);
        assert_eq!(action, EditorAction::MoveCursor(Movement::Left));
    }

    #[test]
    fn test_movement_variants() {
        assert_eq!(Movement::WordStart, Movement::WordStart);
        assert_ne!(Movement::WordStart, Movement::WordEnd);
        
        // Test page movement variants for ENG-134
        assert_eq!(Movement::PageUp, Movement::PageUp);
        assert_eq!(Movement::PageDown, Movement::PageDown);
        assert_ne!(Movement::PageUp, Movement::PageDown);
    }

    #[test]
    fn test_format_type_variants() {
        assert_eq!(FormatType::Bold, FormatType::Bold);
        assert_ne!(FormatType::Bold, FormatType::Italic);
    }

    #[test]
    fn test_advanced_deletion_actions() {
        // Test word deletion actions
        let delete_prev_word = EditorAction::DeletePreviousWord;
        let delete_next_word = EditorAction::DeleteNextWord;
        assert_ne!(delete_prev_word, delete_next_word);
        
        // Test line deletion actions
        let delete_to_start = EditorAction::DeleteToLineStart;
        let delete_to_end = EditorAction::DeleteToLineEnd;
        assert_ne!(delete_to_start, delete_to_end);
        
        // Test current line deletion
        let delete_line = EditorAction::DeleteCurrentLine;
        assert_ne!(delete_line, delete_prev_word);
    }
}