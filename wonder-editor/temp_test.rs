#[cfg(test)]
mod temp_debug_tests {
    use crate::core::text_document::TextDocument;
    use crate::input::actions::{EditorAction, Movement};

    #[test]
    fn debug_positions() {
        let content = "Short\nThis is a very long line that will wrap multiple times when line wrapping is enabled with a narrow width setting\nAnother".to_string();
        let mut doc = TextDocument::with_content(content.clone());
        
        println!("\n=== CONTENT ANALYSIS ===");
        println!("Full content: {:?}", content);
        
        // Print each character with its position
        for (i, ch) in content.chars().enumerate() {
            if i <= 20 {  // Show first 20 characters
                println!("Position {}: '{:?}'", i, ch);
            }
        }
        
        // Test specific positions
        println!("\n=== CURSOR TESTS ===");
        
        // Position 5 - what the test claims is "end of Short"
        doc.set_cursor_position(5);
        let (line, col) = doc.get_cursor_line_and_column();
        println!("Position 5 - Line: {}, Column: {}", line, col);
        
        // Move down
        doc.handle_action(EditorAction::MoveCursor(Movement::Down));
        println!("After down: Position {}", doc.cursor_position());
        
        // Reset and try position 4 (actual end of "Short")
        doc.set_cursor_position(4);
        let (line, col) = doc.get_cursor_line_and_column();
        println!("Position 4 - Line: {}, Column: {}", line, col);
        
        // Move down
        doc.handle_action(EditorAction::MoveCursor(Movement::Down));
        println!("After down from 4: Position {}", doc.cursor_position());
        
        assert!(false, "Debug output shown above");
    }
}