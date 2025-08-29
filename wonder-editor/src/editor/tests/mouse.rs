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
    
    #[test] 
    fn test_mouse_positioning_with_hybrid_content() {
        let mut editor = create_test_editor_minimal();
        
        // Create content with mixed markdown that will have different display vs original positions
        // Content: "Hello **world** and text" 
        // Display: "Hello world and text" (when **world** is in preview mode)
        let content = "Hello **world** and text";
        for ch in content.chars() {
            editor.handle_char_input(ch);
        }
        
        // Test clicking at various display positions and verify cursor accuracy
        
        // Position 0: At "H" - should map correctly
        let result = editor.handle_click_at_position(0);
        assert!(result);
        assert_eq!(editor.cursor_position(), 0, "Click at start should position cursor at 0");
        
        // Position 6: At start of "**world**" in original, but after "Hello " in display
        // When cursor is at 6, the **world** token should be in Raw mode
        let result = editor.handle_click_at_position(6); 
        assert!(result);
        assert_eq!(editor.cursor_position(), 6, "Click at bold markdown should position correctly");
        
        // Position 10: Inside "**world**" - should map to correct position when in raw mode
        let result = editor.handle_click_at_position(10);
        assert!(result);
        assert_eq!(editor.cursor_position(), 10, "Click inside bold content should position correctly");
    }
    
    #[test]
    fn test_mouse_positioning_accuracy_with_headings() {
        let mut editor = create_test_editor_minimal();
        
        // Create content with headings that have different font sizes
        // This tests positioning accuracy with our typography hierarchy
        let content = "# Heading\nRegular text";
        for ch in content.chars() {
            editor.handle_char_input(ch);
        }
        
        // Test positioning at various points
        
        // Position 2: Inside heading (should account for larger font size in preview mode)
        let result = editor.handle_click_at_position(2);
        assert!(result);
        assert_eq!(editor.cursor_position(), 2, "Click inside heading should position correctly");
        
        // Position 10: After newline, in regular text (different font size)
        let result = editor.handle_click_at_position(10);
        assert!(result);
        assert_eq!(editor.cursor_position(), 10, "Click in regular text after heading should position correctly");
    }
    
    #[test] 
    fn test_mouse_positioning_accuracy_long_lines() {
        let mut editor = create_test_editor_minimal();
        
        // Create a document with very long lines - this is where inaccuracy gets worse
        let long_line = "This is a very long line of text with markdown **bold text** and more content that extends far beyond typical line lengths and should expose coordinate mapping issues when clicking at the end";
        for ch in long_line.chars() {
            editor.handle_char_input(ch);
        }
        
        // Test positioning at the end of the long line - this should be most inaccurate
        let end_position = long_line.chars().count();
        let result = editor.handle_click_at_position(end_position);
        assert!(result);
        assert_eq!(editor.cursor_position(), end_position, "Click at end of long line should position correctly");
        
        // Test positioning in the middle of the long line after markdown
        let middle_position = 100; // Position well into the text
        let result = editor.handle_click_at_position(middle_position);
        assert!(result);
        assert_eq!(editor.cursor_position(), middle_position, "Click in middle of long line should position correctly");
    }
    
    #[test]
    fn test_mouse_positioning_with_mixed_line_heights() {
        let mut editor = create_test_editor_minimal();
        
        // Create content with mixed line heights (headings and regular text)
        // This replicates the issue from the user's screenshot
        let content = "# Wonder is an AI powered note taking app that helps you explore your thinking.\n\nMake curiosity the default\n- instead of boredom, curiosity prevails\n- Should be a default reaction to boredom\n\nYour learning platform.";
        for ch in content.chars() {
            editor.handle_char_input(ch);
        }
        
        // Test clicking on "default" in "- Should be a default reaction to boredom"
        // First, find where this line starts
        let lines: Vec<&str> = content.lines().collect();
        let mut chars_before_target_line = 0;
        let mut target_line_index = 0;
        
        for (idx, line) in lines.iter().enumerate() {
            if line.contains("Should be a default reaction") {
                target_line_index = idx;
                break;
            }
            chars_before_target_line += line.chars().count() + 1; // +1 for newline
        }
        
        // Find position of "default" within that line
        let target_line = lines[target_line_index];
        let default_offset = target_line.find("default").expect("Should find 'default' in line");
        let default_position = chars_before_target_line + default_offset;
        
        // Test clicking at "default"
        let result = editor.handle_click_at_position(default_position);
        assert!(result);
        assert_eq!(editor.cursor_position(), default_position, "Click on 'default' should position cursor correctly");
        
        // Test clicking at the start of "default" (the 'd')
        let result = editor.handle_click_at_position(default_position);
        assert!(result);
        assert_eq!(editor.cursor_position(), default_position, "Click at start of 'default' should be accurate");
    }

    // More mouse interaction tests will be moved here from the original test module
}