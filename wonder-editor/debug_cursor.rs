use crate::core::text_document::TextDocument;
use crate::input::actions::{EditorAction, Movement};

fn debug_cursor_test() {
    let mut doc = TextDocument::with_content(
        "Short\nThis is a very long line that will wrap multiple times when line wrapping is enabled with a narrow width setting\nAnother".to_string()
    );
    
    let content = doc.content();
    println!("Content: {:?}", content);
    println!("Content length: {}", content.len());
    
    // Find positions manually
    for (i, ch) in content.chars().enumerate() {
        if i <= 15 {
            println!("Position {}: '{}' ({})", i, ch.escape_debug(), ch as u8);
        }
    }
    
    // Start at the end of the first short line
    doc.set_cursor_position(5); // End of "Short"
    let (line, col) = doc.get_cursor_line_and_column();
    println!("Before move - Position: {}, Line: {}, Column: {}", doc.cursor_position(), line, col);
    
    // Move down should go to the beginning of the wrapped long line
    doc.handle_action(EditorAction::MoveCursor(Movement::Down));
    let (new_line, new_col) = doc.get_cursor_line_and_column();
    println!("After move - Position: {}, Line: {}, Column: {}", doc.cursor_position(), new_line, new_col);
}