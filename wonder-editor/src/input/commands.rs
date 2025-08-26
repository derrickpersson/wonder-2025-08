use crate::core::TextDocument;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    InsertChar(char),
    InsertText(String),
    Backspace,
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorDown,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToDocumentStart,
    MoveToDocumentEnd,
    StartSelection,
    ClearSelection,
    SelectAll,
    ExtendSelectionLeft,
    ExtendSelectionRight,
    MoveToWordStart,
    MoveToWordEnd,
    ExtendSelectionToWordStart,
    ExtendSelectionToWordEnd,
    MovePageUp,
    MovePageDown,
    ExtendSelectionToDocumentStart,
    ExtendSelectionToDocumentEnd,
    ToggleBold,
    ToggleItalic,
    Copy,
    Cut,
    Paste,
    PasteWithoutFormatting,
}

pub trait CommandExecutor {
    fn execute(&mut self, command: EditorCommand) -> bool;
}

impl CommandExecutor for TextDocument {
    fn execute(&mut self, command: EditorCommand) -> bool {
        match command {
            EditorCommand::InsertChar(ch) => {
                self.insert_char(ch);
                true
            }
            EditorCommand::InsertText(text) => {
                self.insert_text(&text);
                true
            }
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Delete => self.delete_char(),
            EditorCommand::MoveCursorLeft => {
                self.move_cursor_left();
                true
            }
            EditorCommand::MoveCursorRight => {
                self.move_cursor_right();
                true
            }
            EditorCommand::MoveCursorUp => {
                self.move_cursor_up();
                true
            }
            EditorCommand::MoveCursorDown => {
                self.move_cursor_down();
                true
            }
            EditorCommand::MoveToLineStart => {
                self.move_to_line_start();
                true
            }
            EditorCommand::MoveToLineEnd => {
                self.move_to_line_end();
                true
            }
            EditorCommand::MoveToDocumentStart => {
                self.move_to_document_start();
                true
            }
            EditorCommand::MoveToDocumentEnd => {
                self.move_to_document_end();
                true
            }
            EditorCommand::StartSelection => {
                self.start_selection();
                true
            }
            EditorCommand::ClearSelection => {
                self.clear_selection();
                true
            }
            EditorCommand::SelectAll => {
                self.select_all();
                true
            }
            EditorCommand::ExtendSelectionLeft => {
                self.extend_selection_left();
                true
            }
            EditorCommand::ExtendSelectionRight => {
                self.extend_selection_right();
                true
            }
            EditorCommand::MoveToWordStart => {
                self.move_to_word_start();
                true
            }
            EditorCommand::MoveToWordEnd => {
                self.move_to_word_end();
                true
            }
            EditorCommand::ExtendSelectionToWordStart => {
                self.extend_selection_to_word_start();
                true
            }
            EditorCommand::ExtendSelectionToWordEnd => {
                self.extend_selection_to_word_end();
                true
            }
            EditorCommand::MovePageUp => {
                self.move_page_up();
                true
            }
            EditorCommand::MovePageDown => {
                self.move_page_down();
                true
            }
            EditorCommand::ExtendSelectionToDocumentStart => {
                self.extend_selection_to_document_start();
                true
            }
            EditorCommand::ExtendSelectionToDocumentEnd => {
                self.extend_selection_to_document_end();
                true
            }
            EditorCommand::ToggleBold => {
                self.toggle_bold();
                true
            }
            EditorCommand::ToggleItalic => {
                self.toggle_italic();
                true
            }
            EditorCommand::Copy => {
                self.copy();
                true
            }
            EditorCommand::Cut => {
                self.cut();
                true
            }
            EditorCommand::Paste => {
                self.paste();
                true
            }
            EditorCommand::PasteWithoutFormatting => {
                self.paste(); // For now, same as regular paste
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char_command() {
        let mut doc = TextDocument::new();
        let command = EditorCommand::InsertChar('a');
        let result = doc.execute(command);
        
        assert!(result);
        assert_eq!(doc.content(), "a");
        assert_eq!(doc.cursor_position(), 1);
    }

    #[test]
    fn test_movement_commands() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        doc.set_cursor_position(3);
        
        let result = doc.execute(EditorCommand::MoveCursorLeft);
        assert!(result);
        assert_eq!(doc.cursor_position(), 2);
        
        let result = doc.execute(EditorCommand::MoveCursorRight);
        assert!(result);
        assert_eq!(doc.cursor_position(), 3);
    }

    #[test]
    fn test_backspace_command() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        let result = doc.execute(EditorCommand::Backspace);
        
        assert!(result);
        assert_eq!(doc.content(), "Hell");
        assert_eq!(doc.cursor_position(), 4);
    }

    #[test]
    fn test_selection_commands() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        doc.set_cursor_position(2);
        
        let result = doc.execute(EditorCommand::StartSelection);
        assert!(result);
        
        doc.set_cursor_position(5);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("llo".to_string()));
        
        let result = doc.execute(EditorCommand::ClearSelection);
        assert!(result);
        assert!(!doc.has_selection());
    }

    #[test]
    fn test_cmd_b_wraps_selection_with_asterisks() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(0);
        doc.start_selection();
        doc.set_cursor_position(5); // Select "Hello"
        
        let result = doc.execute(EditorCommand::ToggleBold);
        
        assert!(result);
        assert_eq!(doc.content(), "**Hello** world");
        assert!(!doc.has_selection()); // Selection should be cleared after formatting
    }

    #[test]
    fn test_cmd_i_wraps_selection_with_single_asterisk() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(6);
        doc.start_selection();
        doc.set_cursor_position(11); // Select "world"
        
        let result = doc.execute(EditorCommand::ToggleItalic);
        
        assert!(result);
        assert_eq!(doc.content(), "Hello *world*");
        assert!(!doc.has_selection()); // Selection should be cleared after formatting
    }

    #[test]
    fn test_toggle_existing_bold_formatting() {
        let mut doc = TextDocument::with_content("**bold text** normal".to_string());
        doc.set_cursor_position(2);  // Inside the bold text
        doc.start_selection();
        doc.set_cursor_position(11); // Select "bold text"
        
        let result = doc.execute(EditorCommand::ToggleBold);
        
        assert!(result);
        assert_eq!(doc.content(), "bold text normal");
        assert!(!doc.has_selection());
    }

    #[test]
    fn test_insert_bold_markers_when_no_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(6); // Between "Hello" and "world"
        
        let result = doc.execute(EditorCommand::ToggleBold);
        
        assert!(result);
        assert_eq!(doc.content(), "Hello ****world");
        // Cursor should be positioned between the markers
        assert_eq!(doc.cursor_position(), 8); 
    }

    #[test]
    fn test_insert_italic_markers_when_no_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(6); // Between "Hello" and "world"
        
        let result = doc.execute(EditorCommand::ToggleItalic);
        
        assert!(result);
        assert_eq!(doc.content(), "Hello **world");
        // Cursor should be positioned between the markers
        assert_eq!(doc.cursor_position(), 7);
    }

    #[test]
    fn test_shift_arrow_left_extends_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5); // After "Hello"
        
        // First shift+left should start selection and move cursor left
        let result = doc.execute(EditorCommand::ExtendSelectionLeft);
        assert!(result);
        assert_eq!(doc.cursor_position(), 4);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("o".to_string()));
        
        // Second shift+left should extend selection further
        let result = doc.execute(EditorCommand::ExtendSelectionLeft);
        assert!(result);
        assert_eq!(doc.cursor_position(), 3);
        assert_eq!(doc.selected_text(), Some("lo".to_string()));
    }

    #[test]
    fn test_shift_arrow_right_extends_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5); // After "Hello"
        
        // First shift+right should start selection and move cursor right
        let result = doc.execute(EditorCommand::ExtendSelectionRight);
        assert!(result);
        assert_eq!(doc.cursor_position(), 6);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some(" ".to_string()));
        
        // Second shift+right should extend selection further
        let result = doc.execute(EditorCommand::ExtendSelectionRight);
        assert!(result);
        assert_eq!(doc.cursor_position(), 7);
        assert_eq!(doc.selected_text(), Some(" w".to_string()));
    }

    #[test]
    fn test_shift_arrow_changes_direction() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5); // After "Hello"
        
        // Start with shift+right
        doc.execute(EditorCommand::ExtendSelectionRight);
        assert_eq!(doc.selected_text(), Some(" ".to_string()));
        
        // Now shift+left should reduce selection by going back
        let result = doc.execute(EditorCommand::ExtendSelectionLeft);
        assert!(result);
        assert_eq!(doc.cursor_position(), 5);
        assert!(!doc.has_selection()); // Selection should be cleared when cursor returns to anchor
    }

    #[test]
    fn test_copy_with_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(0);
        doc.start_selection();
        doc.set_cursor_position(5); // Select "Hello"
        
        let result = doc.execute(EditorCommand::Copy);
        assert!(result);
        
        // Content should remain unchanged
        assert_eq!(doc.content(), "Hello world");
        // Selection should remain
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("Hello".to_string()));
        // Should have copied to clipboard (verified by checking clipboard state)
        assert_eq!(doc.get_clipboard_content(), Some("Hello".to_string()));
    }

    #[test]
    fn test_copy_without_selection_copies_current_line() {
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        doc.set_cursor_position(9); // In "Line 2"
        
        let result = doc.execute(EditorCommand::Copy);
        assert!(result);
        
        // Content should remain unchanged
        assert_eq!(doc.content(), "Line 1\nLine 2\nLine 3");
        // No selection should exist
        assert!(!doc.has_selection());
        // Should have copied current line to clipboard
        assert_eq!(doc.get_clipboard_content(), Some("Line 2\n".to_string()));
    }

    #[test]
    fn test_cut_with_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(0);
        doc.start_selection();
        doc.set_cursor_position(5); // Select "Hello"
        
        let result = doc.execute(EditorCommand::Cut);
        assert!(result);
        
        // Selected text should be removed
        assert_eq!(doc.content(), " world");
        // No selection should remain
        assert!(!doc.has_selection());
        // Cursor should be at start of remaining text
        assert_eq!(doc.cursor_position(), 0);
        // Should have copied to clipboard
        assert_eq!(doc.get_clipboard_content(), Some("Hello".to_string()));
    }

    #[test]
    fn test_cut_without_selection_cuts_current_line() {
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        doc.set_cursor_position(9); // In "Line 2"
        
        let result = doc.execute(EditorCommand::Cut);
        assert!(result);
        
        // Current line should be removed
        assert_eq!(doc.content(), "Line 1\nLine 3");
        // No selection should exist
        assert!(!doc.has_selection());
        // Cursor should be at start of next line
        assert_eq!(doc.cursor_position(), 7); // Start of "Line 3"
        // Should have copied current line to clipboard
        assert_eq!(doc.get_clipboard_content(), Some("Line 2\n".to_string()));
    }

    #[test]
    fn test_paste_with_clipboard_content() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5); // After "Hello"
        
        // Set up clipboard content
        doc.copy_text_to_clipboard(" amazing".to_string());
        
        let result = doc.execute(EditorCommand::Paste);
        assert!(result);
        
        // Text should be inserted at cursor position
        assert_eq!(doc.content(), "Hello amazing world");
        // Cursor should be after pasted text
        assert_eq!(doc.cursor_position(), 13); // After " amazing"
    }

    #[test]
    fn test_paste_replaces_selection() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(6); // Start of "world"
        doc.start_selection();
        doc.set_cursor_position(11); // Select "world"
        
        // Set up clipboard content
        doc.copy_text_to_clipboard("universe".to_string());
        
        let result = doc.execute(EditorCommand::Paste);
        assert!(result);
        
        // Selected text should be replaced with pasted content
        assert_eq!(doc.content(), "Hello universe");
        // No selection should remain
        assert!(!doc.has_selection());
        // Cursor should be after pasted text
        assert_eq!(doc.cursor_position(), 14); // After "universe"
    }

    #[test]
    fn test_paste_without_clipboard_content() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5);
        
        // No clipboard content
        assert_eq!(doc.get_clipboard_content(), None);
        
        let result = doc.execute(EditorCommand::Paste);
        assert!(result);
        
        // Content should remain unchanged when clipboard is empty
        assert_eq!(doc.content(), "Hello world");
        assert_eq!(doc.cursor_position(), 5);
    }
}