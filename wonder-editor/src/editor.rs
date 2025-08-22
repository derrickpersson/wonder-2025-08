use gpui::{
    div, prelude::*, rgb, Context, Render,
};
use crate::text_buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialKey {
    Backspace,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

pub struct MarkdownEditor {
    content: String,
    cursor_position: usize,
    text_buffer: Option<TextBuffer>,
    focused: bool,
}

impl MarkdownEditor {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            text_buffer: None,
            focused: false,
        }
    }

    pub fn new_with_buffer() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            text_buffer: Some(TextBuffer::new()),
            focused: false,
        }
    }
    
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }
    
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }
    
    pub fn get_content(&self) -> &str {
        if let Some(ref buffer) = self.text_buffer {
            buffer.content()
        } else {
            &self.content
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(ref mut buffer) = self.text_buffer {
            buffer.insert_char(ch);
        }
    }

    pub fn cursor_position(&self) -> usize {
        if let Some(ref buffer) = self.text_buffer {
            buffer.cursor_position()
        } else {
            self.cursor_position
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(ref mut buffer) = self.text_buffer {
            buffer.delete_char();
        } else {
            if self.cursor_position > 0 {
                self.cursor_position -= 1;
                self.content.remove(self.cursor_position);
            }
        }
    }

    pub fn handle_key_input(&mut self, ch: char) {
        self.insert_char(ch);
    }

    pub fn handle_special_key(&mut self, key: SpecialKey) {
        match key {
            SpecialKey::Backspace => {
                self.delete_char();
            },
            SpecialKey::ArrowLeft => {
                if let Some(ref mut buffer) = self.text_buffer {
                    buffer.move_cursor_left();
                } else {
                    self.move_cursor_left();
                }
            },
            SpecialKey::ArrowRight => {
                if let Some(ref mut buffer) = self.text_buffer {
                    buffer.move_cursor_right();
                } else {
                    self.move_cursor_right();
                }
            },
            SpecialKey::ArrowUp => {
                if let Some(ref mut buffer) = self.text_buffer {
                    buffer.move_cursor_up();
                }
            },
            SpecialKey::ArrowDown => {
                if let Some(ref mut buffer) = self.text_buffer {
                    buffer.move_cursor_down();
                }
            },
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content = if let Some(ref buffer) = self.text_buffer {
            buffer.content().to_string()
        } else if self.content.is_empty() {
            "Start typing your markdown content...".to_string()
        } else {
            self.content.clone()
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

    #[test]
    fn test_handle_keyboard_input_basic_char() {
        let mut editor = MarkdownEditor::new_with_buffer();
        
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
        let mut editor = MarkdownEditor::new_with_buffer();
        
        // Type some text first
        editor.handle_key_input('h');
        editor.handle_key_input('e');
        editor.handle_key_input('l');
        editor.handle_key_input('l');
        editor.handle_key_input('o');
        assert_eq!(editor.get_content(), "hello");
        assert_eq!(editor.cursor_position(), 5);
        
        // Test backspace key
        editor.handle_special_key(SpecialKey::Backspace);
        assert_eq!(editor.get_content(), "hell");
        assert_eq!(editor.cursor_position(), 4);
        
        // Test arrow keys
        editor.handle_special_key(SpecialKey::ArrowLeft);
        assert_eq!(editor.cursor_position(), 3);
        
        editor.handle_special_key(SpecialKey::ArrowRight);
        assert_eq!(editor.cursor_position(), 4);
    }

    #[test]
    fn test_focus_handling() {
        let mut editor = MarkdownEditor::new_with_buffer();
        
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
        let mut editor = MarkdownEditor {
            content: String::new(),
            cursor_position: 0,
            text_buffer: None,
            focused: false,
        };
        
        editor.insert_text("Hello");
        assert_eq!(editor.get_content(), "Hello");
        assert_eq!(editor.cursor_position, 5);
    }
    
    #[test]
    fn test_delete_char() {
        let mut editor = MarkdownEditor {
            content: "Hello".to_string(),
            cursor_position: 5,
            text_buffer: None,
            focused: false,
        };
        
        editor.delete_char();
        assert_eq!(editor.get_content(), "Hell");
        assert_eq!(editor.cursor_position, 4);
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut editor = MarkdownEditor {
            content: "Hello".to_string(),
            cursor_position: 3,
            text_buffer: None,
            focused: false,
        };
        
        editor.move_cursor_left();
        assert_eq!(editor.cursor_position, 2);
        
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position, 3);
        
        editor.move_cursor_right();
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position, 5);
        
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position, 5); // Should not go beyond content length
    }

    #[test]
    fn test_editor_with_text_buffer() {
        let mut editor = MarkdownEditor::new_with_buffer();
        editor.insert_char('H');
        assert_eq!(editor.get_content(), "H");
        assert_eq!(editor.cursor_position(), 1);
        
        editor.delete_char();
        assert_eq!(editor.get_content(), "");
        assert_eq!(editor.cursor_position(), 0);
    }
}