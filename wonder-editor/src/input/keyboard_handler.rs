use super::{input_event::InputEvent, commands::{EditorCommand, CommandExecutor}};

#[derive(Debug)]
pub struct KeyboardHandler;

impl KeyboardHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_input_event<T: CommandExecutor>(
        &self,
        event: InputEvent,
        target: &mut T,
    ) -> bool {
        let command = self.input_event_to_command(event);
        target.execute(command)
    }

    pub fn handle_char_input<T: CommandExecutor>(
        &self,
        ch: char,
        target: &mut T,
    ) -> bool {
        self.handle_input_event(InputEvent::Character(ch), target)
    }

    fn input_event_to_command(&self, event: InputEvent) -> EditorCommand {
        match event {
            InputEvent::Character(ch) => EditorCommand::InsertChar(ch),
            InputEvent::Backspace => EditorCommand::Backspace,
            InputEvent::Delete => EditorCommand::Delete,
            InputEvent::ArrowLeft => EditorCommand::MoveCursorLeft,
            InputEvent::ArrowRight => EditorCommand::MoveCursorRight,
            InputEvent::ArrowUp => EditorCommand::MoveCursorUp,
            InputEvent::ArrowDown => EditorCommand::MoveCursorDown,
            InputEvent::ShiftArrowLeft => EditorCommand::ExtendSelectionLeft,
            InputEvent::ShiftArrowRight => EditorCommand::ExtendSelectionRight,
            InputEvent::CmdArrowLeft => EditorCommand::MoveToWordStart,
            InputEvent::CmdArrowRight => EditorCommand::MoveToWordEnd,
            InputEvent::CmdShiftArrowLeft => EditorCommand::ExtendSelectionToWordStart,
            InputEvent::CmdShiftArrowRight => EditorCommand::ExtendSelectionToWordEnd,
            InputEvent::Home => EditorCommand::MoveToLineStart,
            InputEvent::End => EditorCommand::MoveToLineEnd,
            InputEvent::PageUp => EditorCommand::MoveToDocumentStart,
            InputEvent::PageDown => EditorCommand::MoveToDocumentEnd,
            InputEvent::Enter => EditorCommand::InsertChar('\n'),
            InputEvent::Tab => EditorCommand::InsertChar('\t'),
        }
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TextDocument;

    #[test]
    fn test_keyboard_handler_char_input() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::new();
        
        let result = handler.handle_char_input('a', &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "a");
        assert_eq!(doc.cursor_position(), 1);
    }

    #[test]
    fn test_keyboard_handler_backspace() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello".to_string());
        
        let result = handler.handle_input_event(InputEvent::Backspace, &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "Hell");
        assert_eq!(doc.cursor_position(), 4);
    }

    #[test]
    fn test_keyboard_handler_arrow_keys() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello".to_string());
        doc.set_cursor_position(3);
        
        let result = handler.handle_input_event(InputEvent::ArrowLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 2);
        
        let result = handler.handle_input_event(InputEvent::ArrowRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 3);
    }

    #[test]
    fn test_keyboard_handler_home_end() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello World".to_string());
        doc.set_cursor_position(6);
        
        let result = handler.handle_input_event(InputEvent::Home, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0);
        
        let result = handler.handle_input_event(InputEvent::End, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 11);
    }

    #[test]
    fn test_keyboard_handler_enter_tab() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::new();
        
        let result = handler.handle_input_event(InputEvent::Enter, &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "\n");
        
        let result = handler.handle_input_event(InputEvent::Tab, &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "\n\t");
    }

    #[test]
    fn test_keyboard_handler_word_navigation() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // Middle of "world"
        
        // Test CmdArrowLeft - should go to start of word
        let result = handler.handle_input_event(InputEvent::CmdArrowLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 6); // Start of "world"
        
        // Test CmdArrowRight - should go to end of word
        let result = handler.handle_input_event(InputEvent::CmdArrowRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 11); // End of "world"
    }

    #[test]
    fn test_keyboard_handler_word_selection() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // Middle of "world"
        
        // Test CmdShiftArrowLeft - should select to start of word
        let result = handler.handle_input_event(InputEvent::CmdShiftArrowLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 6);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("wo".to_string()));
        
        // Clear selection
        doc.clear_selection();
        doc.set_cursor_position(8);
        
        // Test CmdShiftArrowRight - should select to end of word
        let result = handler.handle_input_event(InputEvent::CmdShiftArrowRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 11);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("rld".to_string()));
    }
}