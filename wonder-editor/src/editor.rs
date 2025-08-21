use gpui::{
    div, prelude::*, rgb, Context, Render,
};

pub struct MarkdownEditor {
    content: String,
    cursor_position: usize,
}

impl MarkdownEditor {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
        }
    }
    
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
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
        &self.content
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                    .border_color(rgb(0x313244))
                    .p_4()
                    .text_color(rgb(0xcdd6f4))
                    .child(if self.content.is_empty() {
                        "Start typing your markdown content...".to_string()
                    } else {
                        self.content.clone()
                    })
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_insert_text() {
        let mut editor = MarkdownEditor {
            content: String::new(),
            cursor_position: 0,
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
}