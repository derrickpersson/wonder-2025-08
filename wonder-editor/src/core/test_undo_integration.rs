//! Test the undo/redo integration with real text operations

#[cfg(test)]
mod tests {
    use crate::core::TextDocument;

    #[test]
    fn test_typing_creates_undoable_commands() {
        let mut doc = TextDocument::new();
        
        // Initially no undo available
        assert!(!doc.can_undo());
        
        // Type a character using the direct method (which should now use commands)
        doc.insert_char('H');
        
        // Now undo should be available
        assert!(doc.can_undo());
        assert_eq!(doc.content(), "H");
        
        // Undo should work
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "");
        assert!(!doc.can_undo());
        
        // Redo should work
        assert!(doc.can_redo());
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "H");
    }

    #[test]
    fn test_backspace_creates_undoable_commands() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        
        // Move cursor to end
        doc.set_cursor_position(5);
        
        // Backspace should create undoable command
        assert!(doc.backspace());
        assert_eq!(doc.content(), "Hell");
        assert!(doc.can_undo());
        
        // Undo should restore the character
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "Hello");
    }

    #[test]
    fn test_multiple_typing_operations() {
        let mut doc = TextDocument::new();
        
        // Type multiple characters - these will be grouped into one transaction
        doc.insert_char('H');
        doc.insert_char('e');  
        doc.insert_char('l');
        doc.insert_char('l');
        doc.insert_char('o');
        
        assert_eq!(doc.content(), "Hello");
        assert!(doc.can_undo());
        
        // All characters should be undone together as one transaction
        assert!(doc.perform_undo()); // Remove entire "Hello" transaction
        assert_eq!(doc.content(), "");
        
        // No more undos
        assert!(!doc.can_undo());
        
        // Can redo the entire transaction
        assert!(doc.can_redo());
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "Hello");
        assert!(!doc.can_redo());
    }

}