use gpui::{
    div, prelude::*, rgb, Context, Render,
};
use crate::core::TextDocument;
use crate::input::{KeyboardHandler, InputEvent};

pub struct MarkdownEditor {
    document: TextDocument,
    keyboard_handler: KeyboardHandler,
    focused: bool,
}

impl MarkdownEditor {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            document: TextDocument::new(),
            keyboard_handler: KeyboardHandler::new(),
            focused: false,
        }
    }

    pub fn new_with_content(content: String) -> Self {
        Self {
            document: TextDocument::with_content(content),
            keyboard_handler: KeyboardHandler::new(),
            focused: false,
        }
    }

    // Content access
    pub fn content(&self) -> &str {
        self.document.content()
    }

    pub fn cursor_position(&self) -> usize {
        self.document.cursor_position()
    }

    pub fn has_selection(&self) -> bool {
        self.document.has_selection()
    }

    // Focus management
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    // Input handling - delegates to keyboard handler
    pub fn handle_char_input(&mut self, ch: char) {
        self.keyboard_handler.handle_char_input(ch, &mut self.document);
    }

    pub fn handle_input_event(&mut self, event: InputEvent) {
        self.keyboard_handler.handle_input_event(event, &mut self.document);
    }

    // Legacy compatibility methods for tests
    pub fn get_content(&self) -> &str {
        self.content()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn handle_key_input(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn handle_special_key(&mut self, key: crate::input::SpecialKey) {
        let event: InputEvent = key.into();
        self.handle_input_event(event);
    }

    pub fn delete_char(&mut self) {
        self.handle_input_event(InputEvent::Backspace);
    }

    // Provide access to document for more complex operations
    pub fn document(&self) -> &TextDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut TextDocument {
        &mut self.document
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = if self.document.is_empty() {
            "Start typing your markdown content...".to_string()
        } else {
            self.document.content().to_string()
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .p_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .bg(rgb(0x11111b))
                    .rounded_md()
                    .border_1()
                    .border_color(if self.focused { rgb(0x89b4fa) } else { rgb(0x313244) })
                    .p_4()
                    .text_color(rgb(0xcdd6f4))
                    .child(content)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper method for backward compatibility
    fn new_with_buffer() -> MarkdownEditor {
        MarkdownEditor {
            document: TextDocument::new(),
            keyboard_handler: KeyboardHandler::new(),
            focused: false,
        }
    }

    #[test]
    fn test_handle_keyboard_input_basic_char() {
        let mut editor = new_with_buffer();
        
        // Test character input handling
        editor.handle_key_input('a');
        assert_eq!(editor.get_content(), "a");
        assert_eq!(editor.cursor_position(), 1);
        
        editor.handle_key_input('b');
        assert_eq!(editor.get_content(), "ab");
        assert_eq!(editor.cursor_position(), 2);
    }

    #[test]
    fn test_handle_special_keys() {
        let mut editor = new_with_buffer();
        
        // Type some text first
        editor.handle_key_input('h');
        editor.handle_key_input('e');
        editor.handle_key_input('l');
        editor.handle_key_input('l');
        editor.handle_key_input('o');
        assert_eq!(editor.get_content(), "hello");
        assert_eq!(editor.cursor_position(), 5);
        
        // Test backspace key
        editor.handle_special_key(crate::input::SpecialKey::Backspace);
        assert_eq!(editor.get_content(), "hell");
        assert_eq!(editor.cursor_position(), 4);
        
        // Test arrow keys
        editor.handle_special_key(crate::input::SpecialKey::ArrowLeft);
        assert_eq!(editor.cursor_position(), 3);
        
        editor.handle_special_key(crate::input::SpecialKey::ArrowRight);
        assert_eq!(editor.cursor_position(), 4);
    }

    #[test]
    fn test_focus_handling() {
        let mut editor = new_with_buffer();
        
        // Editor should start unfocused
        assert_eq!(editor.is_focused(), false);
        
        // Test setting focus
        editor.set_focus(true);
        assert_eq!(editor.is_focused(), true);
        
        // Test removing focus
        editor.set_focus(false);
        assert_eq!(editor.is_focused(), false);
    }
    
    #[test]
    fn test_insert_text() {
        let mut editor = new_with_buffer();
        editor.document_mut().insert_text("Hello");
        assert_eq!(editor.get_content(), "Hello");
        assert_eq!(editor.cursor_position(), 5);
    }
    
    #[test]
    fn test_delete_char() {
        let mut editor = MarkdownEditor::new_with_content("Hello".to_string());
        editor.delete_char();
        assert_eq!(editor.get_content(), "Hell");
        assert_eq!(editor.cursor_position(), 4);
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut editor = MarkdownEditor::new_with_content("Hello".to_string());
        editor.document_mut().set_cursor_position(3);
        
        editor.handle_input_event(InputEvent::ArrowLeft);
        assert_eq!(editor.cursor_position(), 2);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 3);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 5);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 5); // Should not go beyond content length
    }

    #[test]
    fn test_editor_with_text_buffer() {
        let mut editor = new_with_buffer();
        editor.insert_char('H');
        assert_eq!(editor.get_content(), "H");
        assert_eq!(editor.cursor_position(), 1);
        
        editor.delete_char();
        assert_eq!(editor.get_content(), "");
        assert_eq!(editor.cursor_position(), 0);
    }
}