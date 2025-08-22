use super::{cursor::Cursor, selection::Selection};

#[derive(Debug, Clone, PartialEq)]
pub struct TextDocument {
    content: String,
    cursor: Cursor,
    selection: Selection,
}

impl TextDocument {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: Cursor::new(),
            selection: Selection::new(),
        }
    }

    pub fn with_content(content: String) -> Self {
        let mut cursor = Cursor::new();
        cursor.set_position(content.len());
        Self {
            content,
            cursor,
            selection: Selection::new(),
        }
    }

    // Content access
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    // Cursor operations
    pub fn cursor_position(&self) -> usize {
        self.cursor.position()
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        let max_pos = self.content.len();
        let clamped_position = position.min(max_pos);
        self.cursor.set_position(clamped_position);
        // Note: Deliberately NOT clearing selection here to support selection operations
    }

    // Selection operations
    pub fn has_selection(&self) -> bool {
        self.selection.is_active()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.range(self.cursor.position())
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(start, end)| {
            self.content[start..end].to_string()
        })
    }

    pub fn start_selection(&mut self) {
        self.selection.start(self.cursor.position());
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select_all(&mut self) {
        self.cursor.set_position(self.content.len());
        self.selection.start(0);
    }

    // Text modification
    pub fn insert_char(&mut self, ch: char) {
        if self.has_selection() {
            self.delete_selection();
        }
        
        let position = self.cursor.position();
        self.content.insert(position, ch);
        self.cursor.move_right(self.content.len());
    }

    pub fn insert_text(&mut self, text: &str) {
        if self.has_selection() {
            self.delete_selection();
        }
        
        let position = self.cursor.position();
        self.content.insert_str(position, text);
        let new_position = position + text.len();
        self.cursor.set_position(new_position);
    }

    pub fn delete_char(&mut self) -> bool {
        if self.has_selection() {
            return self.delete_selection();
        }
        
        let position = self.cursor.position();
        if position < self.content.len() {
            self.content.remove(position);
            true
        } else {
            false
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.has_selection() {
            return self.delete_selection();
        }
        
        let position = self.cursor.position();
        if position > 0 {
            self.cursor.move_left();
            self.content.remove(position - 1);
            true
        } else {
            false
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            self.content.drain(start..end);
            self.cursor.set_position(start);
            self.selection.clear();
            true
        } else {
            false
        }
    }

    // Cursor movement
    pub fn move_cursor_left(&mut self) {
        self.cursor.move_left();
        self.selection.clear();
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor.move_right(self.content.len());
        self.selection.clear();
    }

    pub fn move_cursor_up(&mut self) {
        let (line_index, column) = self.get_cursor_line_and_column();
        if line_index > 0 {
            let new_position = self.get_position_from_line_and_column(line_index - 1, column);
            self.set_cursor_position(new_position);
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (line_index, column) = self.get_cursor_line_and_column();
        let lines: Vec<&str> = self.content.lines().collect();
        if line_index + 1 < lines.len() {
            let new_position = self.get_position_from_line_and_column(line_index + 1, column);
            self.set_cursor_position(new_position);
        }
    }

    pub fn move_to_line_start(&mut self) {
        let (line_index, _) = self.get_cursor_line_and_column();
        let position = self.get_position_from_line_and_column(line_index, 0);
        self.set_cursor_position(position);
    }

    pub fn move_to_line_end(&mut self) {
        let (line_index, _) = self.get_cursor_line_and_column();
        let lines: Vec<&str> = self.content.lines().collect();
        let line_length = lines.get(line_index).map(|line| line.len()).unwrap_or(0);
        let position = self.get_position_from_line_and_column(line_index, line_length);
        self.set_cursor_position(position);
    }

    pub fn move_to_document_start(&mut self) {
        self.set_cursor_position(0);
    }

    pub fn move_to_document_end(&mut self) {
        self.set_cursor_position(self.content.len());
    }

    // Helper methods for line/column calculations
    fn get_cursor_line_and_column(&self) -> (usize, usize) {
        let position = self.cursor.position();
        let content_up_to_cursor = &self.content[..position];
        let line_index = content_up_to_cursor.matches('\n').count();
        let column = content_up_to_cursor
            .lines()
            .last()
            .map(|line| line.len())
            .unwrap_or(position);
        (line_index, column)
    }

    fn get_position_from_line_and_column(&self, line_index: usize, column: usize) -> usize {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut position = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if i == line_index {
                position += column.min(line.len());
                break;
            }
            position += line.len() + 1; // +1 for newline
        }
        
        position.min(self.content.len())
    }
}

impl Default for TextDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_document_creation() {
        let doc = TextDocument::new();
        assert!(doc.is_empty());
        assert_eq!(doc.cursor_position(), 0);
        assert!(!doc.has_selection());
    }

    #[test]
    fn test_text_document_with_content() {
        let doc = TextDocument::with_content("Hello".to_string());
        assert_eq!(doc.content(), "Hello");
        assert_eq!(doc.cursor_position(), 5);
    }

    #[test]
    fn test_insert_char() {
        let mut doc = TextDocument::new();
        doc.insert_char('H');
        assert_eq!(doc.content(), "H");
        assert_eq!(doc.cursor_position(), 1);
    }

    #[test]
    fn test_insert_text() {
        let mut doc = TextDocument::new();
        doc.insert_text("Hello");
        assert_eq!(doc.content(), "Hello");
        assert_eq!(doc.cursor_position(), 5);
    }

    #[test]
    fn test_backspace() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        let result = doc.backspace();
        assert!(result);
        assert_eq!(doc.content(), "Hell");
        assert_eq!(doc.cursor_position(), 4);
    }

    #[test]
    fn test_delete_char() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        doc.set_cursor_position(2);
        let result = doc.delete_char();
        assert!(result);
        assert_eq!(doc.content(), "Helo");
        assert_eq!(doc.cursor_position(), 2);
    }

    #[test]
    fn test_cursor_movement() {
        let mut doc = TextDocument::with_content("Hello".to_string());
        doc.set_cursor_position(3);
        
        doc.move_cursor_left();
        assert_eq!(doc.cursor_position(), 2);
        
        doc.move_cursor_right();
        assert_eq!(doc.cursor_position(), 3);
    }

    #[test]
    fn test_selection() {
        let mut doc = TextDocument::with_content("Hello World".to_string());
        doc.set_cursor_position(6);
        doc.start_selection();
        doc.set_cursor_position(11);
        
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("World".to_string()));
        
        doc.clear_selection();
        assert!(!doc.has_selection());
    }

    #[test]
    fn test_multiline_cursor_movement() {
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        doc.set_cursor_position(9); // Position in "Line 2"
        
        doc.move_cursor_up();
        assert_eq!(doc.cursor_position(), 2); // Should be in "Line 1"
        
        doc.move_cursor_down();
        assert_eq!(doc.cursor_position(), 9); // Back to "Line 2"
    }
}