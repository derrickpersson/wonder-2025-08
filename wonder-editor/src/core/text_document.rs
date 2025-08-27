use super::{cursor::Cursor, selection::Selection};

#[derive(Debug, Clone, PartialEq)]
pub struct TextDocument {
    content: String,
    cursor: Cursor,
    selection: Selection,
    clipboard: Option<String>,
}

impl TextDocument {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: Cursor::new(),
            selection: Selection::new(),
            clipboard: None,
        }
    }

    pub fn with_content(content: String) -> Self {
        let mut cursor = Cursor::new();
        cursor.set_position(content.chars().count());
        Self {
            content,
            cursor,
            selection: Selection::new(),
            clipboard: None,
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
        self.content.chars().count()
    }

    // Cursor operations
    pub fn cursor_position(&self) -> usize {
        self.cursor.position()
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        let max_pos = self.content.chars().count();
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
        self.cursor.set_position(self.content.chars().count());
        self.selection.start(0);
    }

    // Formatting operations
    pub fn toggle_bold(&mut self) {
        if self.has_selection() {
            if self.selection_has_formatting("**", "**") {
                self.remove_formatting_from_selection("**", "**");
            } else {
                self.wrap_selection_with("**", "**");
            }
        } else {
            self.insert_text("****");
            // Move cursor between the markers
            self.move_cursor_left();
            self.move_cursor_left();
        }
    }

    pub fn toggle_italic(&mut self) {
        if self.has_selection() {
            if self.selection_has_formatting("*", "*") {
                self.remove_formatting_from_selection("*", "*");
            } else {
                self.wrap_selection_with("*", "*");
            }
        } else {
            self.insert_text("**");
            // Move cursor between the markers
            self.move_cursor_left();
        }
    }

    fn wrap_selection_with(&mut self, start_marker: &str, end_marker: &str) {
        if let Some((start, end)) = self.selection_range() {
            let selected_content = self.content[start..end].to_string();
            let wrapped_content = format!("{}{}{}", start_marker, selected_content, end_marker);
            
            // Replace selected text with wrapped content
            let char_start = self.byte_to_char_position(start);
            let _char_end = self.byte_to_char_position(end);
            
            self.content.replace_range(start..end, &wrapped_content);
            
            // Update cursor position to end of wrapped content
            let new_position = char_start + wrapped_content.chars().count();
            self.cursor.set_position(new_position);
            
            // Clear selection
            self.clear_selection();
        }
    }

    fn byte_to_char_position(&self, byte_position: usize) -> usize {
        self.content[..byte_position].chars().count()
    }

    fn selection_has_formatting(&self, start_marker: &str, end_marker: &str) -> bool {
        if let Some((start, end)) = self.selection_range() {
            let start_marker_len = start_marker.len();
            let end_marker_len = end_marker.len();
            
            // Check if there's enough content before and after selection for markers
            if start < start_marker_len || end + end_marker_len > self.content.len() {
                return false;
            }
            
            let before_selection = &self.content[start - start_marker_len..start];
            let after_selection = &self.content[end..end + end_marker_len];
            
            before_selection == start_marker && after_selection == end_marker
        } else {
            false
        }
    }

    fn remove_formatting_from_selection(&mut self, start_marker: &str, end_marker: &str) {
        if let Some((start, end)) = self.selection_range() {
            let start_marker_len = start_marker.len();
            let end_marker_len = end_marker.len();
            
            // Remove end marker first (so positions don't change)
            self.content.replace_range(end..end + end_marker_len, "");
            
            // Remove start marker
            self.content.replace_range(start - start_marker_len..start, "");
            
            // Update cursor position
            let char_start = self.byte_to_char_position(start - start_marker_len);
            let selected_text_len = self.byte_to_char_position(end) - self.byte_to_char_position(start);
            let new_position = char_start + selected_text_len;
            self.cursor.set_position(new_position);
            
            // Clear selection
            self.clear_selection();
        }
    }

    // Text modification
    pub fn insert_char(&mut self, ch: char) {
        if self.has_selection() {
            self.delete_selection();
        }
        
        let char_position = self.cursor.position();
        let byte_position = self.char_position_to_byte_position(char_position);
        self.content.insert(byte_position, ch);
        // Move cursor forward by exactly 1 character position
        let max_position = self.content.chars().count();
        self.cursor.set_position((char_position + 1).min(max_position));
    }

    pub fn insert_text(&mut self, text: &str) {
        if self.has_selection() {
            self.delete_selection();
        }
        
        let char_position = self.cursor.position();
        let byte_position = self.char_position_to_byte_position(char_position);
        self.content.insert_str(byte_position, text);
        let new_char_position = char_position + text.chars().count();
        self.cursor.set_position(new_char_position);
    }

    pub fn delete_char(&mut self) -> bool {
        if self.has_selection() {
            return self.delete_selection();
        }
        
        let char_position = self.cursor.position();
        let char_count = self.content.chars().count();
        if char_position < char_count {
            let byte_position = self.char_position_to_byte_position(char_position);
            self.content.remove(byte_position);
            true
        } else {
            false
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.has_selection() {
            return self.delete_selection();
        }
        
        let char_position = self.cursor.position();
        if char_position > 0 {
            self.cursor.move_left();
            let byte_position = self.char_position_to_byte_position(char_position - 1);
            self.content.remove(byte_position);
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
        self.cursor.move_right(self.content.chars().count());
        self.selection.clear();
    }

    // Selection extension methods
    pub fn extend_selection_left(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        // Move cursor left
        self.cursor.move_left();
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    pub fn extend_selection_right(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        // Move cursor right
        self.cursor.move_right(self.content.chars().count());
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        let (line_index, column) = self.get_cursor_line_and_column();
        if line_index > 0 {
            let new_position = self.get_position_from_line_and_column(line_index - 1, column);
            self.set_cursor_position(new_position);
        } else {
            // When on first line, move to start of document
            self.set_cursor_position(0);
        }
    }

    pub fn move_cursor_down(&mut self) {
        let (line_index, column) = self.get_cursor_line_and_column();
        let lines: Vec<&str> = self.content.lines().collect();
        if line_index + 1 < lines.len() {
            let new_position = self.get_position_from_line_and_column(line_index + 1, column);
            self.set_cursor_position(new_position);
        } else {
            // When on last line, move to end of document
            self.set_cursor_position(self.content.chars().count());
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
        let line_length = lines.get(line_index).map(|line| line.chars().count()).unwrap_or(0);
        let position = self.get_position_from_line_and_column(line_index, line_length);
        self.set_cursor_position(position);
    }

    pub fn move_to_document_start(&mut self) {
        self.set_cursor_position(0);
    }

    pub fn move_to_document_end(&mut self) {
        self.set_cursor_position(self.content.chars().count());
    }

    // Word navigation methods
    pub fn move_to_word_start(&mut self) {
        let position = self.find_word_start(self.cursor.position());
        self.set_cursor_position(position);
        self.selection.clear();
    }

    pub fn move_to_word_end(&mut self) {
        let position = self.find_word_end(self.cursor.position());
        self.set_cursor_position(position);
        self.selection.clear();
    }

    pub fn extend_selection_to_word_start(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        let position = self.find_word_start(self.cursor.position());
        self.set_cursor_position(position);
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    pub fn extend_selection_to_word_end(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        let position = self.find_word_end(self.cursor.position());
        self.set_cursor_position(position);
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    // Page navigation methods
    pub fn move_page_up(&mut self) {
        // For now, implement as moving up by 10 lines (typical page size)
        let (line_index, column) = self.get_cursor_line_and_column();
        let target_line = line_index.saturating_sub(10);
        let new_position = self.get_position_from_line_and_column(target_line, column);
        self.set_cursor_position(new_position);
        self.selection.clear();
    }

    pub fn move_page_down(&mut self) {
        // For now, implement as moving down by 10 lines (typical page size)
        let (line_index, column) = self.get_cursor_line_and_column();
        let lines: Vec<&str> = self.content.lines().collect();
        let target_line = (line_index + 10).min(lines.len().saturating_sub(1));
        let new_position = self.get_position_from_line_and_column(target_line, column);
        self.set_cursor_position(new_position);
        self.selection.clear();
    }

    pub fn extend_selection_to_document_start(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        self.set_cursor_position(0);
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    pub fn extend_selection_to_document_end(&mut self) {
        // Start selection if not active
        if !self.selection.is_active() {
            self.selection.start(self.cursor.position());
        }
        
        self.set_cursor_position(self.content.chars().count());
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    // Helper methods for word boundary detection
    fn find_word_start(&self, position: usize) -> usize {
        let chars: Vec<char> = self.content.chars().collect();
        
        // If at start of document, stay there
        if position == 0 {
            return 0;
        }
        
        let mut pos = position;
        
        // First, skip back over any non-word characters if we're on them
        while pos > 0 {
            if let Some(ch) = chars.get(pos - 1) {
                if !ch.is_alphabetic() && !ch.is_numeric() {
                    pos -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        // Then, move back to the start of the word
        while pos > 0 {
            if let Some(ch) = chars.get(pos - 1) {
                if ch.is_alphabetic() || ch.is_numeric() {
                    pos -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        pos
    }
    
    fn find_word_end(&self, position: usize) -> usize {
        let chars: Vec<char> = self.content.chars().collect();
        let max_position = chars.len();
        
        // If at end of document, stay there
        if position >= max_position {
            return max_position;
        }
        
        let mut pos = position;
        
        // First, skip forward over any non-word characters if we're on them
        while pos < max_position {
            if let Some(ch) = chars.get(pos) {
                if !ch.is_alphabetic() && !ch.is_numeric() {
                    pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        // Then, move forward to the end of the word
        while pos < max_position {
            if let Some(ch) = chars.get(pos) {
                if ch.is_alphabetic() || ch.is_numeric() {
                    pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        pos
    }

    // Helper methods for line/column calculations
    fn get_cursor_line_and_column(&self) -> (usize, usize) {
        let char_position = self.cursor.position();
        let chars: Vec<char> = self.content.chars().collect();
        let content_up_to_cursor: String = chars.iter().take(char_position).collect();
        let line_index = content_up_to_cursor.matches('\n').count();
        let column = content_up_to_cursor
            .lines()
            .last()
            .map(|line| line.chars().count())
            .unwrap_or(char_position);
        (line_index, column)
    }

    fn get_position_from_line_and_column(&self, line_index: usize, column: usize) -> usize {
        let lines: Vec<&str> = self.content.lines().collect();
        let mut position = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if i == line_index {
                position += column.min(line.chars().count());
                break;
            }
            position += line.chars().count() + 1; // +1 for newline
        }
        
        position.min(self.content.chars().count())
    }

    fn char_position_to_byte_position(&self, char_position: usize) -> usize {
        self.content
            .char_indices()
            .nth(char_position)
            .map(|(byte_pos, _)| byte_pos)
            .unwrap_or(self.content.len())
    }
}

impl Default for TextDocument {
    fn default() -> Self {
        Self::new()
    }
}

// Implement ActionHandler for TextDocument
use crate::input::{ActionHandler, EditorAction, Movement, FormatType};

impl ActionHandler for TextDocument {
    fn handle_action(&mut self, action: EditorAction) -> bool {
        match action {
            EditorAction::InsertChar(ch) => {
                self.insert_char(ch);
                true
            }
            EditorAction::InsertText(text) => {
                self.insert_text(&text);
                true
            }
            EditorAction::Backspace => {
                self.backspace()
            }
            EditorAction::Delete => {
                self.delete_char()
            }
            EditorAction::MoveCursor(movement) => {
                self.handle_cursor_movement(movement);
                true
            }
            EditorAction::ExtendSelection(movement) => {
                self.handle_selection_extension(movement);
                true
            }
            EditorAction::SelectAll => {
                self.select_all();
                true
            }
            EditorAction::ClearSelection => {
                self.clear_selection();
                true
            }
            EditorAction::ToggleFormat(format_type) => {
                match format_type {
                    FormatType::Bold => {
                        self.toggle_bold();
                        true
                    }
                    FormatType::Italic => {
                        self.toggle_italic();
                        true
                    }
                    FormatType::Code => {
                        // TODO: Implement code formatting
                        false
                    }
                }
            }
            EditorAction::MoveToPosition(position) => {
                self.set_cursor_position(position);
                true
            }
            EditorAction::PageUp => {
                self.move_page_up();
                true
            }
            EditorAction::PageDown => {
                self.move_page_down();
                true
            }
            EditorAction::Copy => {
                // For copy/cut/paste, we need special handling in the editor
                // since we need access to the GPUI context for system clipboard
                // The ActionHandler just prepares the data
                self.copy();
                true
            }
            EditorAction::Cut => {
                self.cut();
                true
            }
            EditorAction::Paste => {
                // Paste with no external text (will use internal clipboard)
                self.paste(None);
                true
            }
        }
    }
}

impl TextDocument {
    /// Handle cursor movement actions
    fn handle_cursor_movement(&mut self, movement: Movement) {
        match movement {
            Movement::Left => self.move_cursor_left(),
            Movement::Right => self.move_cursor_right(),
            Movement::Up => self.move_cursor_up(),
            Movement::Down => self.move_cursor_down(),
            Movement::WordStart => self.move_to_word_start(),
            Movement::WordEnd => self.move_to_word_end(),
            Movement::LineStart => self.move_to_line_start(),
            Movement::LineEnd => self.move_to_line_end(),
            Movement::DocumentStart => self.move_to_document_start(),
            Movement::DocumentEnd => self.move_to_document_end(),
        }
    }

    /// Handle selection extension actions
    fn handle_selection_extension(&mut self, movement: Movement) {
        match movement {
            Movement::Left => self.extend_selection_left(),
            Movement::Right => self.extend_selection_right(),
            Movement::Up => {
                // Start selection if not active, then move up
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_cursor_up();
            }
            Movement::Down => {
                // Start selection if not active, then move down
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_cursor_down();
            }
            Movement::WordStart => self.extend_selection_to_word_start(),
            Movement::WordEnd => self.extend_selection_to_word_end(),
            Movement::LineStart => {
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_to_line_start();
            }
            Movement::LineEnd => {
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_to_line_end();
            }
            Movement::DocumentStart => self.extend_selection_to_document_start(),
            Movement::DocumentEnd => self.extend_selection_to_document_end(),
        }
    }

    // Clipboard operations - returns text to be copied to system clipboard
    pub fn copy(&mut self) -> Option<String> {
        if self.has_selection() {
            // Copy selected text
            if let Some(text) = self.selected_text() {
                self.clipboard = Some(text.clone());
                Some(text)
            } else {
                None
            }
        } else {
            // Copy current line
            let line_text = self.get_current_line_with_newline();
            self.clipboard = Some(line_text.clone());
            Some(line_text)
        }
    }

    pub fn cut(&mut self) -> Option<String> {
        if self.has_selection() {
            // Cut selected text
            if let Some(text) = self.selected_text() {
                self.clipboard = Some(text.clone());
                self.delete_selection();
                Some(text)
            } else {
                None
            }
        } else {
            // Cut current line
            let line_text = self.get_current_line_with_newline();
            self.clipboard = Some(line_text.clone());
            self.delete_current_line();
            Some(line_text)
        }
    }

    pub fn get_clipboard_content(&self) -> Option<String> {
        self.clipboard.clone()
    }

    pub fn copy_text_to_clipboard(&mut self, text: String) {
        self.clipboard = Some(text);
    }

    pub fn paste(&mut self, clipboard_text: Option<String>) {
        // Try clipboard_text first (system clipboard), fallback to internal clipboard
        let content = clipboard_text.or_else(|| self.clipboard.clone());
        
        if let Some(text) = content {
            if self.has_selection() {
                // Replace selection with pasted content
                self.delete_selection();
            }
            self.insert_text(&text);
        }
    }

    fn get_current_line_with_newline(&self) -> String {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.chars().collect();
        
        // Find line start
        let mut line_start = cursor_pos;
        while line_start > 0 && content_chars.get(line_start - 1) != Some(&'\n') {
            line_start -= 1;
        }
        
        // Find line end (including newline)
        let mut line_end = cursor_pos;
        while line_end < content_chars.len() && content_chars.get(line_end) != Some(&'\n') {
            line_end += 1;
        }
        if line_end < content_chars.len() {
            line_end += 1; // Include the newline
        }
        
        content_chars[line_start..line_end].iter().collect()
    }

    fn delete_current_line(&mut self) {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.chars().collect();
        
        // Find line start
        let mut line_start = cursor_pos;
        while line_start > 0 && content_chars.get(line_start - 1) != Some(&'\n') {
            line_start -= 1;
        }
        
        // Find line end (including newline)
        let mut line_end = cursor_pos;
        while line_end < content_chars.len() && content_chars.get(line_end) != Some(&'\n') {
            line_end += 1;
        }
        if line_end < content_chars.len() {
            line_end += 1; // Include the newline
        }
        
        // Remove the line
        let new_content: String = content_chars[..line_start]
            .iter()
            .chain(content_chars[line_end..].iter())
            .collect();
        
        self.content = new_content;
        // Set cursor to start of next line (or end if this was last line)
        let new_cursor_pos = if line_start < self.content.chars().count() {
            line_start
        } else {
            self.content.chars().count()
        };
        self.set_cursor_position(new_cursor_pos);
    }

    pub fn extend_selection_to_line_start(&mut self) {
        if !self.has_selection() {
            self.start_selection();
        }
        self.move_to_line_start();
    }

    pub fn extend_selection_to_line_end(&mut self) {
        if !self.has_selection() {
            self.start_selection();
        }
        self.move_to_line_end();
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

    #[test]
    fn test_cursor_up_down_edge_cases() {
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        
        // Test: cursor up from first line goes to start of document
        doc.set_cursor_position(3); // Middle of "Line 1"
        doc.move_cursor_up();
        assert_eq!(doc.cursor_position(), 0); // Should go to start of document
        
        // Test: cursor down from last line goes to end of document  
        doc.set_cursor_position(17); // Middle of "Line 3" 
        doc.move_cursor_down();
        assert_eq!(doc.cursor_position(), 20); // Should go to end of document (chars count)
        
        // Test: cursor up from first character goes to start
        doc.set_cursor_position(0); // Start of document
        doc.move_cursor_up();
        assert_eq!(doc.cursor_position(), 0); // Should stay at start
        
        // Test: cursor down from last character goes to end
        doc.set_cursor_position(20); // End of document
        doc.move_cursor_down();
        assert_eq!(doc.cursor_position(), 20); // Should stay at end
    }

    #[test]
    fn test_word_navigation() {
        let mut doc = TextDocument::with_content("Hello world, this is a test".to_string());
        doc.set_cursor_position(8); // Position in middle of "world"
        
        // Test move to word start - should go to start of current word "world"
        doc.move_to_word_start();
        assert_eq!(doc.cursor_position(), 6); // Start of "world"
        
        // Test move to word end - should go to end of current word "world"  
        doc.move_to_word_end();
        assert_eq!(doc.cursor_position(), 11); // End of "world"
        
        // Test from punctuation - cursor at comma
        doc.set_cursor_position(11); // At comma after "world"
        doc.move_to_word_start();
        assert_eq!(doc.cursor_position(), 6); // Should go to start of "world"
    }

    #[test]
    fn test_word_selection_extension() {
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // In middle of "world"
        
        // Test extend selection to word start
        doc.extend_selection_to_word_start();
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("wo".to_string())); // "wo" from "world"
        
        // Clear selection and test extend to word end
        doc.clear_selection();
        doc.set_cursor_position(8);
        doc.extend_selection_to_word_end();
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("rld".to_string())); // "rld" from "world"
    }

    #[test]
    fn test_word_navigation_edge_cases() {
        let mut doc = TextDocument::with_content("start, middle; end".to_string());
        
        // Test from punctuation 
        doc.set_cursor_position(5); // At comma - should go back to start of "start"
        doc.move_to_word_start();
        assert_eq!(doc.cursor_position(), 0); // Should go to start of "start"
        
        doc.set_cursor_position(5); // At comma - should jump forward to end of "middle"  
        doc.move_to_word_end();
        assert_eq!(doc.cursor_position(), 13); // Should go to end of "middle" (position 13)
        
        // Test at word boundaries
        doc.set_cursor_position(0); // Start of document
        doc.move_to_word_start();
        assert_eq!(doc.cursor_position(), 0); // Should stay at start
        
        doc.set_cursor_position(18); // End of document
        doc.move_to_word_end();
        assert_eq!(doc.cursor_position(), 18); // Should stay at end
    }

    #[test]
    fn test_page_navigation() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15".to_string();
        let mut doc = TextDocument::with_content(content);
        doc.set_cursor_position(50); // Somewhere in the middle
        
        // Test page up (should move up by 10 lines)
        doc.move_page_up();
        assert!(doc.cursor_position() < 50); // Should move up
        
        let up_pos = doc.cursor_position();
        
        // Test page down
        doc.move_page_down();
        assert!(doc.cursor_position() > up_pos); // Should move down
    }

    #[test]
    fn test_document_selection_extension() {
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(8); // In middle of "world"
        
        // Test extend selection to document start
        doc.extend_selection_to_document_start();
        assert!(doc.has_selection());
        assert_eq!(doc.cursor_position(), 0);
        assert_eq!(doc.selected_text(), Some("Hello wo".to_string()));
        
        // Clear and test extend to document end
        doc.clear_selection();
        doc.set_cursor_position(8);
        doc.extend_selection_to_document_end();
        assert!(doc.has_selection());
        assert_eq!(doc.cursor_position(), 16);
        assert_eq!(doc.selected_text(), Some("rld test".to_string()));
    }

    #[test]
    fn test_cmd_up_down_navigation() {
        let mut doc = TextDocument::with_content("Line 1\nLine 2\nLine 3".to_string());
        doc.set_cursor_position(8); // In "Line 2"
        
        // Test Cmd+Up (move to document start)
        doc.move_to_document_start();
        assert_eq!(doc.cursor_position(), 0);
        
        // Test Cmd+Down (move to document end)
        doc.move_to_document_end();
        assert_eq!(doc.cursor_position(), 20); // End of document
    }
}