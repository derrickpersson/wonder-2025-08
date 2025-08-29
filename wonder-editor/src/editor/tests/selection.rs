use super::helpers::{create_test_editor_minimal, TestableEditor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_basic() {
        let mut editor = create_test_editor_minimal();
        editor.handle_char_input('H');
        editor.handle_char_input('e');
        editor.handle_char_input('l');
        editor.handle_char_input('l');
        editor.handle_char_input('o');
        
        // Test selection functionality
        editor.document_mut().set_cursor_position(0);
        editor.document_mut().start_selection();
        editor.document_mut().set_cursor_position(5);
        
        assert!(editor.has_selection());
        assert_eq!(editor.selected_text(), Some("Hello".to_string()));
    }

    // More selection tests will be moved here from the original test module
}