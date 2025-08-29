use super::helpers::{create_test_editor_minimal, TestableEditor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_click_position() {
        let mut editor = create_test_editor_minimal();
        editor.handle_char_input('H');
        editor.handle_char_input('e');
        editor.handle_char_input('l');
        editor.handle_char_input('l');
        editor.handle_char_input('o');
        
        // Test clicking at position
        let result = editor.handle_click_at_position(2);
        assert!(result);
        assert_eq!(editor.cursor_position(), 2);
    }

    // More mouse interaction tests will be moved here from the original test module
}