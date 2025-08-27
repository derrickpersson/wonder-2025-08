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
            InputEvent::CmdArrowLeft => EditorCommand::MoveToLineStart,
            InputEvent::CmdArrowRight => EditorCommand::MoveToLineEnd,
            InputEvent::CmdShiftArrowLeft => EditorCommand::ExtendSelectionToWordStart,
            InputEvent::CmdShiftArrowRight => EditorCommand::ExtendSelectionToWordEnd,
            InputEvent::CmdArrowUp => EditorCommand::MoveToDocumentStart,
            InputEvent::CmdArrowDown => EditorCommand::MoveToDocumentEnd,
            InputEvent::CmdShiftArrowUp => EditorCommand::ExtendSelectionToDocumentStart,
            InputEvent::CmdShiftArrowDown => EditorCommand::ExtendSelectionToDocumentEnd,
            InputEvent::Home => EditorCommand::MoveToLineStart,
            InputEvent::End => EditorCommand::MoveToLineEnd,
            InputEvent::PageUp => EditorCommand::MovePageUp,
            InputEvent::PageDown => EditorCommand::MovePageDown,
            InputEvent::Enter => EditorCommand::InsertChar('\n'),
            InputEvent::Tab => EditorCommand::InsertChar('\t'),
            InputEvent::CmdA => EditorCommand::SelectAll,
            InputEvent::CmdC => EditorCommand::Copy,
            InputEvent::CmdX => EditorCommand::Cut,
            InputEvent::CmdV => EditorCommand::Paste,
            InputEvent::CmdShiftV => EditorCommand::PasteWithoutFormatting,
            InputEvent::CtrlC => EditorCommand::Copy,
            InputEvent::CtrlX => EditorCommand::Cut,
            InputEvent::CtrlV => EditorCommand::Paste,
            InputEvent::CtrlShiftV => EditorCommand::PasteWithoutFormatting,
            InputEvent::OptionLeft => EditorCommand::MoveToWordStartPlatform,
            InputEvent::OptionRight => EditorCommand::MoveToWordEndPlatform,
            InputEvent::ShiftHome => EditorCommand::ExtendSelectionToLineStart,
            InputEvent::ShiftEnd => EditorCommand::ExtendSelectionToLineEnd,
            InputEvent::CtrlShiftHome => EditorCommand::ExtendSelectionToDocumentStartPlatform,
            InputEvent::CtrlShiftEnd => EditorCommand::ExtendSelectionToDocumentEndPlatform,
            InputEvent::CmdShiftHome => EditorCommand::ExtendSelectionToDocumentStartPlatform,
            InputEvent::CmdShiftEnd => EditorCommand::ExtendSelectionToDocumentEndPlatform,
            InputEvent::FnUp => EditorCommand::PageUpPlatform,
            InputEvent::FnDown => EditorCommand::PageDownPlatform,
            InputEvent::ShiftPageUp => EditorCommand::ExtendSelectionPageUp,
            InputEvent::ShiftPageDown => EditorCommand::ExtendSelectionPageDown,
            InputEvent::CtrlShiftUp => EditorCommand::ExtendSelectionPageUp,
            InputEvent::CtrlShiftDown => EditorCommand::ExtendSelectionPageDown,
            InputEvent::Escape => EditorCommand::ClearSelection,
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
        
        // Test CmdArrowLeft - now moves to line start on macOS (changed behavior)
        let result = handler.handle_input_event(InputEvent::CmdArrowLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0); // Line start
        
        // Test CmdArrowRight - now moves to line end on macOS (changed behavior)
        let result = handler.handle_input_event(InputEvent::CmdArrowRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 16); // Line end
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

    #[test]
    fn test_keyboard_handler_cmd_up_down() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        doc.set_cursor_position(8); // In "Line 2"
        
        // Test CmdArrowUp - should go to document start
        let result = handler.handle_input_event(InputEvent::CmdArrowUp, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0);
        
        // Test CmdArrowDown - should go to document end
        let result = handler.handle_input_event(InputEvent::CmdArrowDown, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 20); // End of document
    }

    #[test]
    fn test_keyboard_handler_page_navigation() {
        let handler = KeyboardHandler::new();
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15".to_string();
        let mut doc = TextDocument::with_content(content);
        doc.set_cursor_position(50); // Somewhere in the middle
        
        // Test PageUp
        let result = handler.handle_input_event(InputEvent::PageUp, &mut doc);
        assert!(result);
        assert!(doc.cursor_position() < 50); // Should move up
        
        let up_pos = doc.cursor_position();
        
        // Test PageDown
        let result = handler.handle_input_event(InputEvent::PageDown, &mut doc);
        assert!(result);
        assert!(doc.cursor_position() > up_pos); // Should move down
    }

    #[test]
    fn test_keyboard_handler_cmd_shift_up_down() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // In middle
        
        // Test CmdShiftArrowUp - should extend selection to document start
        let result = handler.handle_input_event(InputEvent::CmdShiftArrowUp, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("Hello wo".to_string()));
        
        // Clear selection and test extend to document end
        doc.clear_selection();
        doc.set_cursor_position(8);
        let result = handler.handle_input_event(InputEvent::CmdShiftArrowDown, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 16);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("rld test".to_string()));
    }

    #[test]
    fn test_keyboard_handler_cmd_a_select_all() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // In middle
        
        // Test CmdA - should select all text
        let result = handler.handle_input_event(InputEvent::CmdA, &mut doc);
        assert!(result);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("Hello world test".to_string()));
        assert_eq!(doc.cursor_position(), 16); // At end
    }

    #[test]
    fn test_keyboard_handler_clipboard_operations() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world".to_string());
        
        // Test copy with selection
        doc.set_cursor_position(0);
        doc.start_selection();
        doc.set_cursor_position(5); // Select "Hello"
        
        let result = handler.handle_input_event(InputEvent::CmdC, &mut doc);
        assert!(result);
        assert_eq!(doc.get_clipboard_content(), Some("Hello".to_string()));
        
        // Clear selection and move to end
        doc.clear_selection();
        doc.set_cursor_position(11); // At end
        let result = handler.handle_input_event(InputEvent::CmdV, &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "Hello worldHello");
        assert_eq!(doc.cursor_position(), 16); // After pasted text
        
        // Test cut
        doc.set_cursor_position(6);
        doc.start_selection();
        doc.set_cursor_position(11); // Select "world"
        
        let result = handler.handle_input_event(InputEvent::CmdX, &mut doc);
        assert!(result);
        assert_eq!(doc.content(), "Hello Hello");
        assert_eq!(doc.get_clipboard_content(), Some("world".to_string()));
    }

    #[test]
    fn test_platform_specific_line_navigation_macos() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // Middle of "world"
        
        // On macOS, CmdLeft/Right should move to line boundaries (not word boundaries)
        let result = handler.handle_input_event(InputEvent::CmdArrowLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0); // Should go to line start, not word start
        
        let result = handler.handle_input_event(InputEvent::CmdArrowRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 16); // Should go to line end, not word end
    }

    #[test]
    fn test_option_arrow_word_navigation_macos() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // Middle of "world"
        
        // On macOS, Option+Left/Right should move to word boundaries
        let result = handler.handle_input_event(InputEvent::OptionLeft, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 6); // Start of "world"
        
        let result = handler.handle_input_event(InputEvent::OptionRight, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 11); // End of "world"
    }

    #[test]
    fn test_shift_home_end_selection() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // Middle of "world"
        
        // Test Shift+Home - should extend selection to line start
        let result = handler.handle_input_event(InputEvent::ShiftHome, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 0);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("Hello wo".to_string()));
        
        // Clear selection and test Shift+End
        doc.clear_selection();
        doc.set_cursor_position(8);
        let result = handler.handle_input_event(InputEvent::ShiftEnd, &mut doc);
        assert!(result);
        assert_eq!(doc.cursor_position(), 16);
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("rld test".to_string()));
    }

    #[test]
    fn test_escape_clears_selection() {
        let handler = KeyboardHandler::new();
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(0);
        doc.start_selection();
        doc.set_cursor_position(5); // Select "Hello"
        
        assert!(doc.has_selection());
        
        let result = handler.handle_input_event(InputEvent::Escape, &mut doc);
        assert!(result);
        assert!(!doc.has_selection());
    }
}