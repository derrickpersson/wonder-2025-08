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
}