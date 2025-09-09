use crate::core::text_document::TextDocument;
use crate::input::actions::{EditorAction, Movement};

#[test]
fn debug_cursor_movement() {
    let content = "Short\nThis is a very long line that will wrap multiple times when line wrapping is enabled with a narrow width setting\nAnother".to_string();
    let mut doc = TextDocument::with_content(content.clone());
    
    println!("Content: {}", content);
    println!("Content characters:");
    for (i, ch) in content.chars().enumerate() {
        if i <= 15 {
            println!("  Position {}: {:?} ({})", i, ch, ch.escape_debug());
        }
    }
    
    // Start at the end of the first short line
    doc.set_cursor_position(5); // End of "Short"
    let (line, col) = doc.get_cursor_line_and_column();
    println!("\nBefore move:");
    println!("  Position: {}", doc.cursor_position());
    println!("  Line: {}, Column: {}", line, col);
    
    // Check what character we're at
    let content_chars: Vec<char> = content.chars().collect();
    if doc.cursor_position() < content_chars.len() {
        println!("  Character at position: {:?}", content_chars[doc.cursor_position()]);
    }
    
    // Move down should go to the beginning of the wrapped long line
    doc.handle_action(EditorAction::MoveCursor(Movement::Down));
    let (new_line, new_col) = doc.get_cursor_line_and_column();
    
    println!("\nAfter move down:");
    println!("  Position: {}", doc.cursor_position());
    println!("  Line: {}, Column: {}", new_line, new_col);
    
    // Check what character we're at now
    if doc.cursor_position() < content_chars.len() {
        println!("  Character at position: {:?}", content_chars[doc.cursor_position()]);
    }
    
    // What should the position be?
    println!("\nExpected:");
    println!("  Should be at position 6 (start of 'This is a very long line...')");
    println!("  Character at position 6: {:?}", content_chars[6]);
}