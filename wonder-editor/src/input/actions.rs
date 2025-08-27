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
    }

    #[test]
    fn test_format_type_variants() {
        assert_eq!(FormatType::Bold, FormatType::Bold);
        assert_ne!(FormatType::Bold, FormatType::Italic);
    }
}