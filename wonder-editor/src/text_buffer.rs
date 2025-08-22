#[derive(Debug, Clone, PartialEq)]
pub struct TextBuffer {
    content: String,
    cursor_position: usize,
    selection_anchor: Option<usize>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            selection_anchor: None,
        }
    }

    pub fn with_content(content: String) -> Self {
        let cursor_position = content.len();
        Self {
            content,
            cursor_position,
            selection_anchor: None,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        // If there's a selection, delete it first
        if self.selection_anchor.is_some() {
            self.delete_selection();
        }
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn insert_text(&mut self, text: &str) {
        // If there's a selection, delete it first
        if self.selection_anchor.is_some() {
            self.delete_selection();
        }
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    pub fn delete_char(&mut self) {
        // If there's a selection, delete it instead
        if self.selection_anchor.is_some() {
            self.delete_selection();
            return;
        }
        
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    pub fn backspace(&mut self) {
        // Same as delete_char for now
        self.delete_char();
    }

    pub fn delete_selection(&mut self) {
        if let (Some(start), Some(end)) = (self.selection_start(), self.selection_end()) {
            self.content.drain(start..end);
            self.cursor_position = start;
            self.selection_anchor = None;
        }
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = std::cmp::min(position, self.content.len());
    }

    pub fn move_cursor_up(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.len() <= 1 {
            return; // Single line, do nothing
        }

        let (current_line_index, column) = self.get_cursor_line_and_column();
        if current_line_index > 0 {
            let prev_line = lines[current_line_index - 1];
            let new_column = std::cmp::min(column, prev_line.len());
            self.cursor_position = self.get_position_from_line_and_column(current_line_index - 1, new_column);
        }
    }

    pub fn move_cursor_down(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.len() <= 1 {
            return; // Single line, do nothing
        }

        let (current_line_index, column) = self.get_cursor_line_and_column();
        if current_line_index < lines.len() - 1 {
            let next_line = lines[current_line_index + 1];
            let new_column = std::cmp::min(column, next_line.len());
            self.cursor_position = self.get_position_from_line_and_column(current_line_index + 1, new_column);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            let mut new_position = self.cursor_position - 1;
            while new_position > 0 && !self.content.is_char_boundary(new_position) {
                new_position -= 1;
            }
            self.cursor_position = new_position;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            let mut new_position = self.cursor_position + 1;
            while new_position < self.content.len() && !self.content.is_char_boundary(new_position) {
                new_position += 1;
            }
            self.cursor_position = new_position;
        }
    }

    pub fn move_to_line_start(&mut self) {
        let (current_line_index, _column) = self.get_cursor_line_and_column();
        self.cursor_position = self.get_position_from_line_and_column(current_line_index, 0);
    }

    pub fn move_to_line_end(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        let (current_line_index, _column) = self.get_cursor_line_and_column();
        
        if current_line_index < lines.len() {
            let line_length = lines[current_line_index].len();
            // If not the last line, position at the newline
            if current_line_index < lines.len() - 1 {
                self.cursor_position = self.get_position_from_line_and_column(current_line_index, line_length);
            } else {
                // Last line, position at end of content
                self.cursor_position = self.content.len();
            }
        }
    }

    pub fn move_to_word_start(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let chars: Vec<char> = self.content.chars().collect();
        let mut pos = self.cursor_position;
        
        // Convert byte position to char index
        let mut char_pos = self.content[..pos].chars().count();
        if char_pos > 0 {
            char_pos -= 1;
        }

        // Skip any whitespace backwards
        while char_pos > 0 && chars[char_pos].is_whitespace() {
            char_pos -= 1;
        }

        // If we're in the middle of a word, go to the start of current word
        if char_pos > 0 && !chars[char_pos].is_whitespace() {
            while char_pos > 0 && !chars[char_pos - 1].is_whitespace() {
                char_pos -= 1;
            }
        }

        // Convert char index back to byte position
        self.cursor_position = chars.iter().take(char_pos).map(|c| c.len_utf8()).sum();
    }

    pub fn move_to_word_end(&mut self) {
        let chars: Vec<char> = self.content.chars().collect();
        if chars.is_empty() {
            return;
        }

        // Convert byte position to char index
        let mut char_pos = self.content[..self.cursor_position].chars().count();
        
        // If at end, stay there
        if char_pos >= chars.len() {
            return;
        }

        // If we're in a word, skip to end of current word
        if char_pos < chars.len() && !chars[char_pos].is_whitespace() {
            while char_pos < chars.len() && !chars[char_pos].is_whitespace() {
                char_pos += 1;
            }
        } else {
            // We're in whitespace, skip it
            while char_pos < chars.len() && chars[char_pos].is_whitespace() {
                char_pos += 1;
            }
            // Then move to end of next word
            while char_pos < chars.len() && !chars[char_pos].is_whitespace() {
                char_pos += 1;
            }
        }

        // Convert char index back to byte position
        self.cursor_position = chars.iter().take(char_pos).map(|c| c.len_utf8()).sum();
    }

    pub fn move_to_document_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_to_document_end(&mut self) {
        self.cursor_position = self.content.len();
    }

    pub fn start_selection(&mut self) {
        self.selection_anchor = Some(self.cursor_position);
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn selection_start(&self) -> Option<usize> {
        self.selection_anchor.map(|anchor| {
            std::cmp::min(anchor, self.cursor_position)
        })
    }

    pub fn selection_end(&self) -> Option<usize> {
        self.selection_anchor.map(|anchor| {
            std::cmp::max(anchor, self.cursor_position)
        })
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let (Some(start), Some(end)) = (self.selection_start(), self.selection_end()) {
            Some(self.content[start..end].to_string())
        } else {
            None
        }
    }

    pub fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor_position = self.content.len();
    }

    fn get_cursor_line_and_column(&self) -> (usize, usize) {
        let mut line_index = 0;
        let mut char_count = 0;
        
        for (i, line) in self.content.split('\n').enumerate() {
            if char_count + line.len() >= self.cursor_position {
                return (i, self.cursor_position - char_count);
            }
            char_count += line.len() + 1; // +1 for the newline character
            line_index = i + 1;
        }
        
        (line_index, 0)
    }

    fn get_position_from_line_and_column(&self, line_index: usize, column: usize) -> usize {
        let mut position = 0;
        
        for (i, line) in self.content.split('\n').enumerate() {
            if i == line_index {
                return position + column;
            }
            position += line.len() + 1; // +1 for the newline character
        }
        
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_text_buffer_is_empty() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.content, "");
        assert_eq!(buffer.cursor_position, 0);
    }

    #[test]
    fn test_insert_char_in_empty_buffer() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('H');
        assert_eq!(buffer.content, "H");
        assert_eq!(buffer.cursor_position, 1);
    }

    #[test]
    fn test_delete_char_from_end() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char('H');
        buffer.delete_char();
        assert_eq!(buffer.content, "");
        assert_eq!(buffer.cursor_position, 0);
    }

    #[test]
    fn test_delete_char_from_empty_buffer() {
        let mut buffer = TextBuffer::new();
        buffer.delete_char();
        assert_eq!(buffer.content, "");
        assert_eq!(buffer.cursor_position, 0);
    }

    #[test]
    fn test_cursor_position_access() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.cursor_position(), 0);
    }

    #[test]
    fn test_content_access() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.content(), "");
    }

    #[test]
    fn test_move_cursor_up_single_line() {
        let mut buffer = TextBuffer::with_content("Hello".to_string());
        buffer.set_cursor_position(3);
        
        buffer.move_cursor_up();
        // Should stay at same position in single line
        assert_eq!(buffer.cursor_position(), 3);
    }

    #[test]
    fn test_move_cursor_down_single_line() {
        let mut buffer = TextBuffer::with_content("Hello".to_string());
        buffer.set_cursor_position(3);
        
        buffer.move_cursor_down();
        // Should stay at same position in single line
        assert_eq!(buffer.cursor_position(), 3);
    }

    #[test]
    fn test_move_cursor_up_multiline() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        // "Hello\nWorld" positions: H=0, e=1, l=2, l=3, o=4, \n=5, W=6, o=7, r=8, l=9, d=10
        buffer.set_cursor_position(8); // Position 8 is "r" in "World" (column 2 in line 1)
        
        buffer.move_cursor_up();
        // Should move to position 2 ("l" in "Hello") - same column on previous line
        assert_eq!(buffer.cursor_position(), 2);
    }

    #[test] 
    fn test_move_cursor_down_multiline() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        // "Hello\nWorld" positions: H=0, e=1, l=2, l=3, o=4, \n=5, W=6, o=7, r=8, l=9, d=10
        buffer.set_cursor_position(2); // Position 2 is first "l" in "Hello" (column 2 in line 0)
        
        buffer.move_cursor_down();
        // Should move to position 8 ("r" in "World") - same column on next line
        assert_eq!(buffer.cursor_position(), 8);
    }

    #[test]
    fn test_cursor_navigation_edge_cases() {
        // Test with empty buffer
        let mut buffer = TextBuffer::new();
        buffer.move_cursor_up();
        assert_eq!(buffer.cursor_position(), 0);
        buffer.move_cursor_down();
        assert_eq!(buffer.cursor_position(), 0);
        
        // Test moving up from first line (should stay)
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        buffer.set_cursor_position(2);
        buffer.move_cursor_up();
        assert_eq!(buffer.cursor_position(), 2); // Should stay in place
        
        // Test moving down from last line (should stay)
        buffer.set_cursor_position(8);
        buffer.move_cursor_down();
        assert_eq!(buffer.cursor_position(), 8); // Should stay in place
    }

    #[test]
    fn test_cursor_left_right_with_lines() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        buffer.set_cursor_position(5); // At the newline character
        
        buffer.move_cursor_left();
        assert_eq!(buffer.cursor_position(), 4); // Should move to 'o' in Hello
        
        buffer.move_cursor_right();
        assert_eq!(buffer.cursor_position(), 5); // Back to newline
        
        buffer.move_cursor_right();
        assert_eq!(buffer.cursor_position(), 6); // Should move to 'W' in World
    }

    #[test]
    fn test_move_to_line_start() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        
        // Test from middle of first line
        buffer.set_cursor_position(3);
        buffer.move_to_line_start();
        assert_eq!(buffer.cursor_position(), 0);
        
        // Test from middle of second line
        buffer.set_cursor_position(8);
        buffer.move_to_line_start();
        assert_eq!(buffer.cursor_position(), 6); // Start of "World"
    }

    #[test]
    fn test_move_to_line_end() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld".to_string());
        
        // Test from middle of first line
        buffer.set_cursor_position(2);
        buffer.move_to_line_end();
        assert_eq!(buffer.cursor_position(), 5); // At newline
        
        // Test from middle of second line
        buffer.set_cursor_position(8);
        buffer.move_to_line_end();
        assert_eq!(buffer.cursor_position(), 11); // End of "World"
    }

    #[test]
    fn test_move_to_word_start() {
        let mut buffer = TextBuffer::with_content("Hello world from rust".to_string());
        
        // From middle of word, move to start of current word
        buffer.set_cursor_position(7); // 'o' in "world"
        buffer.move_to_word_start();
        assert_eq!(buffer.cursor_position(), 6); // Start of "world"
        
        // From start of word, move to start of previous word
        buffer.move_to_word_start();
        assert_eq!(buffer.cursor_position(), 0); // Start of "Hello"
        
        // Test with spaces
        buffer.set_cursor_position(13); // 'o' in "from"
        buffer.move_to_word_start();
        assert_eq!(buffer.cursor_position(), 12); // Start of "from"
    }

    #[test]
    fn test_move_to_word_end() {
        let mut buffer = TextBuffer::with_content("Hello world from rust".to_string());
        
        // From middle of word, move to end of current word
        buffer.set_cursor_position(2); // 'l' in "Hello"
        buffer.move_to_word_end();
        assert_eq!(buffer.cursor_position(), 5); // After "Hello"
        
        // Move to end of next word
        buffer.move_to_word_end();
        assert_eq!(buffer.cursor_position(), 11); // After "world"
        
        // From end of content
        buffer.set_cursor_position(21); // End of string
        buffer.move_to_word_end();
        assert_eq!(buffer.cursor_position(), 21); // Should stay at end
    }

    #[test]
    fn test_move_to_document_start() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld\nRust".to_string());
        
        buffer.set_cursor_position(10); // Somewhere in middle
        buffer.move_to_document_start();
        assert_eq!(buffer.cursor_position(), 0);
        
        // Already at start
        buffer.move_to_document_start();
        assert_eq!(buffer.cursor_position(), 0);
    }

    #[test]
    fn test_move_to_document_end() {
        let mut buffer = TextBuffer::with_content("Hello\nWorld\nRust".to_string());
        
        buffer.set_cursor_position(5); // Somewhere in middle
        buffer.move_to_document_end();
        assert_eq!(buffer.cursor_position(), 16); // End of "Rust"
        
        // Already at end
        buffer.move_to_document_end();
        assert_eq!(buffer.cursor_position(), 16);
    }

    #[test]
    fn test_text_selection() {
        let mut buffer = TextBuffer::with_content("Hello World".to_string());
        
        // Start selection from position 0
        buffer.set_cursor_position(0);
        buffer.start_selection();
        
        // Extend selection to position 5
        buffer.set_cursor_position(5);
        
        // Check selection range
        assert_eq!(buffer.selection_start(), Some(0));
        assert_eq!(buffer.selection_end(), Some(5));
        assert_eq!(buffer.get_selected_text(), Some("Hello".to_string()));
        
        // Clear selection
        buffer.clear_selection();
        assert_eq!(buffer.selection_start(), None);
        assert_eq!(buffer.selection_end(), None);
    }

    #[test]
    fn test_select_all() {
        let mut buffer = TextBuffer::with_content("Hello World".to_string());
        
        buffer.select_all();
        assert_eq!(buffer.selection_start(), Some(0));
        assert_eq!(buffer.selection_end(), Some(11));
        assert_eq!(buffer.get_selected_text(), Some("Hello World".to_string()));
    }

    #[test]
    fn test_typing_replaces_selection() {
        let mut buffer = TextBuffer::with_content("Hello World".to_string());
        
        // Select "World"
        buffer.set_cursor_position(6);
        buffer.start_selection();
        buffer.set_cursor_position(11);
        
        // Type "Rust" to replace "World"
        buffer.insert_text("Rust");
        
        assert_eq!(buffer.content(), "Hello Rust");
        assert_eq!(buffer.cursor_position(), 10);
        assert_eq!(buffer.selection_start(), None); // Selection should be cleared
    }

    #[test]
    fn test_delete_with_selection() {
        let mut buffer = TextBuffer::with_content("Hello World".to_string());
        
        // Select "Hello "
        buffer.set_cursor_position(0);
        buffer.start_selection();
        buffer.set_cursor_position(6);
        
        // Delete selection
        buffer.delete_selection();
        
        assert_eq!(buffer.content(), "World");
        assert_eq!(buffer.cursor_position(), 0);
        assert_eq!(buffer.selection_start(), None);
    }

    #[test]
    fn test_backspace_with_selection() {
        let mut buffer = TextBuffer::with_content("Hello World".to_string());
        
        // Select " World"
        buffer.set_cursor_position(5);
        buffer.start_selection();
        buffer.set_cursor_position(11);
        
        // Backspace should delete selection
        buffer.backspace();
        
        assert_eq!(buffer.content(), "Hello");
        assert_eq!(buffer.cursor_position(), 5);
        assert_eq!(buffer.selection_start(), None);
    }
}