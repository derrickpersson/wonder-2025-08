use super::{cursor::Cursor, selection::Selection, command_history::CommandHistory, commands::{UndoableCommand, InsertCommand, DeleteCommand, ReplaceCommand}};
use ropey::Rope;

#[derive(Debug)]
pub struct TextDocument {
    content: Rope,
    cursor: Cursor,
    selection: Selection,
    clipboard: Option<String>,
    command_history: CommandHistory,
}

impl TextDocument {
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
            cursor: Cursor::new(),
            selection: Selection::new(),
            clipboard: None,
            command_history: CommandHistory::new(),
        }
    }

    pub fn with_content(content: String) -> Self {
        let mut cursor = Cursor::new();
        cursor.set_position(content.chars().count());
        Self {
            content: Rope::from_str(&content),
            cursor,
            selection: Selection::new(),
            clipboard: None,
            command_history: CommandHistory::new(),
        }
    }

    // Content access
    pub fn content(&self) -> String {
        self.content.to_string()
    }

    pub fn is_empty(&self) -> bool {
        self.content.len_chars() == 0
    }

    pub fn len(&self) -> usize {
        self.content.len_chars()
    }

    // Cursor operations
    pub fn cursor_position(&self) -> usize {
        self.cursor.position()
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        let max_pos = self.content.len_chars();
        let clamped_position = position.min(max_pos);
        self.cursor.set_position(clamped_position);
        // Note: Deliberately NOT clearing selection here to support selection operations
    }

    // Selection operations
    pub fn has_selection(&self) -> bool {
        self.selection.is_active()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        // Ensure cursor position is within bounds before getting range
        let cursor_pos = self.cursor.position().min(self.content.len_chars());
        self.selection.range(cursor_pos).map(|(start, end)| {
            // Ensure the range is within bounds of current content
            let max_pos = self.content.len_chars();
            let safe_start = start.min(max_pos);
            let safe_end = end.min(max_pos);
            (safe_start, safe_end)
        })
    }
    
    /// Safe rope slicing with bounds checking
    fn safe_slice(&self, start: usize, end: usize) -> String {
        let max_pos = self.content.len_chars();
        let safe_start = start.min(max_pos);
        let safe_end = end.min(max_pos).max(safe_start); // Ensure end >= start
        
        if safe_start == safe_end {
            String::new()
        } else {
            self.content.slice(safe_start..safe_end).to_string()
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(start, end)| {
            self.safe_slice(start, end)
        })
    }

    pub fn start_selection(&mut self) {
        self.selection.start(self.cursor.position());
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select_all(&mut self) {
        self.cursor.set_position(self.content.len_chars());
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
            let selected_content = self.safe_slice(start, end);
            let wrapped_content = format!("{}{}{}", start_marker, selected_content, end_marker);
            
            // Replace selected text with wrapped content
            let char_start = self.byte_to_char_position(start);
            let _char_end = self.byte_to_char_position(end);
            
            self.content.remove(start..end);
            self.content.insert(start, &wrapped_content);
            
            // Update cursor position to end of wrapped content
            let new_position = char_start + wrapped_content.chars().count();
            self.cursor.set_position(new_position);
            
            // Clear selection
            self.clear_selection();
        }
    }

    fn byte_to_char_position(&self, byte_position: usize) -> usize {
        byte_position.min(self.content.len_chars())
    }

    fn selection_has_formatting(&self, start_marker: &str, end_marker: &str) -> bool {
        if let Some((start, end)) = self.selection_range() {
            let start_marker_len = start_marker.len();
            let end_marker_len = end_marker.len();
            
            // Check if there's enough content before and after selection for markers
            if start < start_marker_len || end + end_marker_len > self.content.len_chars() {
                return false;
            }
            
            let before_selection = self.safe_slice(start.saturating_sub(start_marker_len), start);
            let after_selection = self.safe_slice(end, end + end_marker_len);
            
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
            self.content.remove(end..end + end_marker_len);
            
            // Remove start marker
            self.content.remove(start - start_marker_len..start);
            
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
        // Handle selection deletion first with command
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                let deleted_text = self.safe_slice(start, end);
                let delete_command = Box::new(DeleteCommand::new(start, end, deleted_text));
                self.execute_command_in_transaction(delete_command);
            }
        }
        
        // Create and execute insert command
        let position = self.cursor.position();
        let insert_command = Box::new(InsertCommand::new(position, ch.to_string()));
        self.execute_command_in_transaction(insert_command);
        
        // Update cursor position
        let new_position = position + 1;
        let max_position = self.content.len_chars();
        self.cursor.set_position(new_position.min(max_position));
    }

    pub fn insert_text(&mut self, text: &str) {
        // Handle selection deletion first with command
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                let deleted_text = self.safe_slice(start, end);
                let delete_command = Box::new(DeleteCommand::new(start, end, deleted_text));
                self.execute_command_in_transaction(delete_command);
            }
        }
        
        // Create and execute insert command
        let position = self.cursor.position();
        let insert_command = Box::new(InsertCommand::new(position, text.to_string()));
        self.execute_command_in_transaction(insert_command);
        
        // Update cursor position
        let new_position = position + text.chars().count();
        self.cursor.set_position(new_position);
    }

    pub fn delete_char(&mut self) -> bool {
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                let deleted_text = self.safe_slice(start, end);
                let delete_command = Box::new(DeleteCommand::new(start, end, deleted_text));
                self.execute_command_in_transaction(delete_command);
                return true;
            }
        }
        
        let position = self.cursor.position();
        if position < self.content.len_chars() {
            // Get the character that will be deleted
            let deleted_char = self.safe_slice(position, position + 1);
            let delete_command = Box::new(DeleteCommand::new(position, position + 1, deleted_char));
            self.execute_command_in_transaction(delete_command);
            true
        } else {
            false
        }
    }

    pub fn backspace(&mut self) -> bool {
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                let deleted_text = self.safe_slice(start, end);
                let delete_command = Box::new(DeleteCommand::new(start, end, deleted_text));
                self.execute_command_in_transaction(delete_command);
                return true;
            }
        }
        
        let position = self.cursor.position();
        if position > 0 {
            // Get the character that will be deleted
            let deleted_char = self.safe_slice(position - 1, position);
            let delete_command = Box::new(DeleteCommand::new(position - 1, position, deleted_char));
            self.execute_command_in_transaction(delete_command);
            
            // Update cursor position
            self.cursor.move_left();
            true
        } else {
            false
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            self.content.remove(start..end);
            self.cursor.set_position(start);
            self.selection.clear();
            true
        } else {
            false
        }
    }

    // Cursor movement
    pub fn move_cursor_left(&mut self) {
        let before = self.cursor.position();
        let (line, col) = self.get_cursor_line_and_column();
        eprintln!("DEBUG: move_cursor_left - Before: pos={}, line={}, col={}", before, line, col);
        
        self.cursor.move_left();
        
        let after = self.cursor.position();
        let (line_after, col_after) = self.get_cursor_line_and_column();
        eprintln!("DEBUG: move_cursor_left - After: pos={}, line={}, col={}", after, line_after, col_after);
    }

    pub fn move_cursor_right(&mut self) {
        let before = self.cursor.position();
        let (line, col) = self.get_cursor_line_and_column();
        let max_pos = self.content.len_chars();
        eprintln!("DEBUG: move_cursor_right - Before: pos={}, line={}, col={}, max={}", before, line, col, max_pos);
        
        // Check if we're at end of line
        if line < self.content.len_lines() {
            let line_slice = self.content.line(line);
            let line_len = line_slice.len_chars();
            let line_content_len = if line_len > 0 && line_slice.char(line_len - 1) == '\n' {
                line_len - 1
            } else {
                line_len
            };
            eprintln!("DEBUG: Line {} has length {} (content: {})", line, line_len, line_content_len);
            eprintln!("DEBUG: At column {}, line ends at column {}", col, line_content_len);
        }
        
        self.cursor.move_right(max_pos);
        
        let after = self.cursor.position();
        let (line_after, col_after) = self.get_cursor_line_and_column();
        eprintln!("DEBUG: move_cursor_right - After: pos={}, line={}, col={}", after, line_after, col_after);
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
        self.cursor.move_right(self.content.len_chars());
        
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
        // Use efficient Ropey method to check if next line exists
        if line_index + 1 < self.content.len_lines() {
            let new_position = self.get_position_from_line_and_column(line_index + 1, column);
            self.set_cursor_position(new_position);
        } else {
            // When on last line, move to end of document
            self.set_cursor_position(self.content.len_chars());
        }
    }

    pub fn move_to_line_start(&mut self) {
        let (line_index, _) = self.get_cursor_line_and_column();
        let position = self.get_position_from_line_and_column(line_index, 0);
        self.set_cursor_position(position);
    }

    pub fn move_to_line_end(&mut self) {
        let (line_index, _) = self.get_cursor_line_and_column();
        // Use efficient Ropey method to get line length
        if line_index < self.content.len_lines() {
            let line_slice = self.content.line(line_index);
            let line_len = line_slice.len_chars();
            // Don't include newline in positioning
            let line_content_len = if line_len > 0 && line_slice.char(line_len - 1) == '\n' {
                line_len - 1
            } else {
                line_len
            };
            let position = self.get_position_from_line_and_column(line_index, line_content_len);
            self.set_cursor_position(position);
        }
    }

    pub fn move_to_document_start(&mut self) {
        self.set_cursor_position(0);
    }

    pub fn move_to_document_end(&mut self) {
        self.set_cursor_position(self.content.len_chars());
    }

    // Word navigation methods
    pub fn move_to_word_start(&mut self) {
        let position = self.find_word_start(self.cursor.position());
        self.set_cursor_position(position);
    }

    pub fn move_to_word_end(&mut self) {
        let position = self.find_word_end(self.cursor.position());
        self.set_cursor_position(position);
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
    }

    pub fn move_page_down(&mut self) {
        // For now, implement as moving down by 10 lines (typical page size)
        let (line_index, column) = self.get_cursor_line_and_column();
        let content_string = self.content.to_string();
        let lines: Vec<&str> = content_string.lines().collect();
        let target_line = (line_index + 10).min(lines.len().saturating_sub(1));
        let new_position = self.get_position_from_line_and_column(target_line, column);
        self.set_cursor_position(new_position);
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
        
        self.set_cursor_position(self.content.len_chars());
        
        // Clear selection if cursor returns to anchor
        if let Some(anchor) = self.selection.anchor() {
            if self.cursor.position() == anchor {
                self.selection.clear();
            }
        }
    }

    // Helper methods for word boundary detection
    fn find_word_start(&self, position: usize) -> usize {
        // If at start of document, stay there
        if position == 0 {
            return 0;
        }
        
        let mut pos = position;
        let max_pos = self.content.len_chars();
        
        // First, skip back over any non-word characters if we're on them
        while pos > 0 {
            let ch = self.content.char(pos - 1);
            if !ch.is_alphabetic() && !ch.is_numeric() {
                pos -= 1;
            } else {
                break;
            }
        }
        
        // Then, move back to the start of the word
        while pos > 0 {
            let ch = self.content.char(pos - 1);
            if ch.is_alphabetic() || ch.is_numeric() {
                pos -= 1;
            } else {
                break;
            }
        }
        
        pos
    }
    
    fn find_word_end(&self, position: usize) -> usize {
        let max_position = self.content.len_chars();
        
        // If at end of document, stay there
        if position >= max_position {
            return max_position;
        }
        
        let mut pos = position;
        
        // First, skip forward over any non-word characters if we're on them
        while pos < max_position {
            let ch = self.content.char(pos);
            if !ch.is_alphabetic() && !ch.is_numeric() {
                pos += 1;
            } else {
                break;
            }
        }
        
        // Then, move forward to the end of the word
        while pos < max_position {
            let ch = self.content.char(pos);
            if ch.is_alphabetic() || ch.is_numeric() {
                pos += 1;
            } else {
                break;
            }
        }
        
        pos
    }

    // Helper methods for line/column calculations
    fn get_cursor_line_and_column(&self) -> (usize, usize) {
        let char_position = self.cursor.position();
        // Use efficient Ropey method O(log n) vs O(n)
        let line_index = self.content.char_to_line(char_position);
        let line_start_char = self.content.line_to_char(line_index);
        let column = char_position - line_start_char;
        (line_index, column)
    }

    fn get_position_from_line_and_column(&self, line_index: usize, column: usize) -> usize {
        // Use efficient Ropey methods O(log n) vs O(n)
        if line_index >= self.content.len_lines() {
            return self.content.len_chars();
        }
        
        let line_start_char = self.content.line_to_char(line_index);
        let line_slice = self.content.line(line_index);
        let line_len = line_slice.len_chars();
        // Don't include newline in line length for positioning
        let line_content_len = if line_len > 0 && line_slice.char(line_len - 1) == '\n' {
            line_len - 1
        } else {
            line_len
        };
        
        let position = line_start_char + column.min(line_content_len);
        position.min(self.content.len_chars())
    }

    fn char_position_to_byte_position(&self, char_position: usize) -> usize {
        // For rope, we can directly use char position since rope uses character indexing
        char_position.min(self.content.len_chars())
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
            // Advanced deletion operations
            EditorAction::DeletePreviousWord => {
                self.delete_previous_word();
                true
            }
            EditorAction::DeleteNextWord => {
                self.delete_next_word();
                true
            }
            EditorAction::DeleteToLineStart => {
                self.delete_to_line_start();
                true
            }
            EditorAction::DeleteToLineEnd => {
                self.delete_to_line_end();
                true
            }
            EditorAction::DeleteCurrentLine => {
                self.delete_current_line();
                true
            }
            EditorAction::Undo => {
                self.perform_undo()
            }
            EditorAction::Redo => {
                self.perform_redo()
            }
        }
    }
}

impl TextDocument {
    /// Handle cursor movement actions
    fn handle_cursor_movement(&mut self, movement: Movement) {
        // Clear selection before any cursor movement (without Shift key)
        self.selection.clear();
        
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
            Movement::PageUp => self.move_page_up(),
            Movement::PageDown => self.move_page_down(),
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
            Movement::PageUp => {
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_page_up();
            }
            Movement::PageDown => {
                if !self.has_selection() {
                    self.start_selection();
                }
                self.move_page_down();
            }
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
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
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
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
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
        
        self.content = Rope::from_str(&new_content);
        // Set cursor to start of next line (or end if this was last line)
        let new_cursor_pos = if line_start < self.content.len_chars() {
            line_start
        } else {
            self.content.len_chars()
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

    /// Delete the previous word from cursor position
    pub fn delete_previous_word(&mut self) {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
        if cursor_pos == 0 {
            return; // Nothing to delete
        }
        
        // Find start of previous word
        let word_start = self.find_word_boundary_backward(cursor_pos);
        
        // Delete from word start to cursor
        let new_content: String = content_chars[..word_start]
            .iter()
            .chain(content_chars[cursor_pos..].iter())
            .collect();
        
        self.content = Rope::from_str(&new_content);
        self.set_cursor_position(word_start);
    }

    /// Delete the next word from cursor position
    pub fn delete_next_word(&mut self) {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
        if cursor_pos >= content_chars.len() {
            return; // Nothing to delete
        }
        
        // Find end of next word
        let word_end = self.find_word_boundary_forward(cursor_pos);
        
        // Delete from cursor to word end
        let new_content: String = content_chars[..cursor_pos]
            .iter()
            .chain(content_chars[word_end..].iter())
            .collect();
        
        self.content = Rope::from_str(&new_content);
        // Cursor position stays the same
    }

    /// Delete from cursor to line start
    pub fn delete_to_line_start(&mut self) {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
        // Find line start
        let mut line_start = cursor_pos;
        while line_start > 0 && content_chars.get(line_start - 1) != Some(&'\n') {
            line_start -= 1;
        }
        
        // Delete from line start to cursor
        let new_content: String = content_chars[..line_start]
            .iter()
            .chain(content_chars[cursor_pos..].iter())
            .collect();
        
        self.content = Rope::from_str(&new_content);
        self.set_cursor_position(line_start);
    }

    /// Delete from cursor to line end
    pub fn delete_to_line_end(&mut self) {
        let cursor_pos = self.cursor_position();
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        
        // Find line end (not including newline)
        let mut line_end = cursor_pos;
        while line_end < content_chars.len() && content_chars.get(line_end) != Some(&'\n') {
            line_end += 1;
        }
        
        // Delete from cursor to line end
        let new_content: String = content_chars[..cursor_pos]
            .iter()
            .chain(content_chars[line_end..].iter())
            .collect();
        
        self.content = Rope::from_str(&new_content);
        // Cursor position stays the same
    }

    /// Find word boundary going backward from position
    fn find_word_boundary_backward(&self, from_pos: usize) -> usize {
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        let mut pos = from_pos;
        
        if pos == 0 {
            return 0;
        }
        
        // Skip any whitespace immediately before cursor
        while pos > 0 && content_chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        
        if pos == 0 {
            return 0;
        }
        
        // Now find the start of the current word
        let first_char = content_chars[pos - 1];
        if first_char.is_alphanumeric() || first_char == '_' {
            // Alphanumeric word
            while pos > 0 && (content_chars[pos - 1].is_alphanumeric() || content_chars[pos - 1] == '_') {
                pos -= 1;
            }
        } else {
            // Punctuation word
            while pos > 0 && !content_chars[pos - 1].is_alphanumeric() && !content_chars[pos - 1].is_whitespace() && content_chars[pos - 1] != '_' {
                pos -= 1;
            }
        }
        
        pos
    }

    /// Find word boundary going forward from position  
    fn find_word_boundary_forward(&self, from_pos: usize) -> usize {
        let content_chars: Vec<char> = self.content.to_string().chars().collect();
        let mut pos = from_pos;
        
        if pos >= content_chars.len() {
            return content_chars.len();
        }
        
        // Skip any whitespace at cursor
        while pos < content_chars.len() && content_chars[pos].is_whitespace() {
            pos += 1;
        }
        
        if pos >= content_chars.len() {
            return content_chars.len();
        }
        
        // Now find the end of the current word
        let first_char = content_chars[pos];
        if first_char.is_alphanumeric() || first_char == '_' {
            // Alphanumeric word
            while pos < content_chars.len() && (content_chars[pos].is_alphanumeric() || content_chars[pos] == '_') {
                pos += 1;
            }
        } else {
            // Punctuation word
            while pos < content_chars.len() && !content_chars[pos].is_alphanumeric() && !content_chars[pos].is_whitespace() && content_chars[pos] != '_' {
                pos += 1;
            }
        }
        
        pos
    }
    
    /// Perform undo operation
    pub fn perform_undo(&mut self) -> bool {
        if let Some(new_content) = self.command_history.undo(&self.content) {
            self.content = new_content;
            
            // Ensure cursor is within bounds after content change
            let current_position = self.cursor.position();
            let max_position = self.content.len_chars();
            if current_position > max_position {
                self.cursor.set_position(max_position);
            }
            
            // Clear selection if it extends beyond the new content
            if let Some((start, end)) = self.selection.range(current_position) {
                if start > max_position || end > max_position {
                    self.selection.clear();
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// Perform redo operation
    pub fn perform_redo(&mut self) -> bool {
        if let Some(new_content) = self.command_history.redo(&self.content) {
            self.content = new_content;
            
            // Ensure cursor is within bounds after content change
            let current_position = self.cursor.position();
            let max_position = self.content.len_chars();
            if current_position > max_position {
                self.cursor.set_position(max_position);
            }
            
            // Clear selection if it extends beyond the new content
            if let Some((start, end)) = self.selection.range(current_position) {
                if start > max_position || end > max_position {
                    self.selection.clear();
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// Execute a command and add it to the history
    pub fn execute_command(&mut self, command: Box<dyn UndoableCommand>) -> bool {
        // Execute the command
        let new_content = command.execute(&self.content);
        
        // For direct execute_command calls, ensure each is individually undoable
        // by finishing current transaction and starting a new one
        self.command_history.finish_current_transaction();
        self.command_history.add_command(command);
        self.command_history.finish_current_transaction();
        
        // Apply the new content
        self.content = new_content;
        true
    }
    
    /// Execute a command as part of a transaction (for user input)
    fn execute_command_in_transaction(&mut self, command: Box<dyn UndoableCommand>) -> bool {
        // Execute the command
        let new_content = command.execute(&self.content);
        
        // Add to current transaction (will be grouped with other operations)
        self.command_history.add_command(command);
        
        // Apply the new content
        self.content = new_content;
        true
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.command_history.can_undo()
    }
    
    /// Check if redo is available  
    pub fn can_redo(&self) -> bool {
        self.command_history.can_redo()
    }
}

// ENG-144: RopeTextDocument for O(log n) performance
#[derive(Debug, Clone)]
pub struct RopeTextDocument {
    content: Rope,
    cursor: Cursor,
    selection: Selection,
}

impl RopeTextDocument {
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
            cursor: Cursor::new(),
            selection: Selection::new(),
        }
    }

    pub fn with_content(content: String) -> Self {
        let mut cursor = Cursor::new();
        cursor.set_position(content.chars().count());
        Self {
            content: Rope::from_str(&content),
            cursor,
            selection: Selection::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.len_chars() == 0
    }

    pub fn len(&self) -> usize {
        self.content.len_chars()
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor.position()
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        let max_pos = self.content.len_chars();
        let clamped_position = position.min(max_pos);
        self.cursor.set_position(clamped_position);
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_active()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        // Ensure cursor position is within bounds before getting range
        let cursor_pos = self.cursor.position().min(self.content.len_chars());
        self.selection.range(cursor_pos).map(|(start, end)| {
            // Ensure the range is within bounds of current content
            let max_pos = self.content.len_chars();
            let safe_start = start.min(max_pos);
            let safe_end = end.min(max_pos);
            (safe_start, safe_end)
        })
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selection_range().map(|(start, end)| {
            // Safe slicing with bounds check
            let max_pos = self.content.len_chars();
            let safe_start = start.min(max_pos);
            let safe_end = end.min(max_pos).max(safe_start);
            
            if safe_start == safe_end {
                String::new()
            } else {
                self.content.slice(safe_start..safe_end).to_string()
            }
        })
    }

    pub fn start_selection(&mut self) {
        self.selection.start(self.cursor.position());
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn content(&self) -> String {
        self.content.to_string()
    }

    pub fn insert_char(&mut self, ch: char) {
        // Direct content manipulation - RopeTextDocument doesn't support commands
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                self.content.remove(start..end);
                self.cursor.set_position(start);
                self.selection.clear();
            }
        }
        
        let position = self.cursor.position();
        self.content.insert_char(position, ch);
        self.cursor.set_position(position + 1);
    }

    pub fn insert_text(&mut self, text: &str) {
        // Direct content manipulation - RopeTextDocument doesn't support commands
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                self.content.remove(start..end);
                self.cursor.set_position(start);
                self.selection.clear();
            }
        }
        
        let position = self.cursor.position();
        self.content.insert(position, text);
        self.cursor.set_position(position + text.chars().count());
    }

    pub fn delete_char(&mut self) -> bool {
        // Direct content manipulation - RopeTextDocument doesn't support commands
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                self.content.remove(start..end);
                self.cursor.set_position(start);
                self.selection.clear();
                return true;
            }
        }
        
        let position = self.cursor.position();
        if position < self.content.len_chars() {
            self.content.remove(position..position + 1);
            true
        } else {
            false
        }
    }

    pub fn backspace(&mut self) -> bool {
        // Direct content manipulation - RopeTextDocument doesn't support commands
        if self.has_selection() {
            if let Some((start, end)) = self.selection_range() {
                self.content.remove(start..end);
                self.cursor.set_position(start);
                self.selection.clear();
                return true;
            }
        }
        
        let position = self.cursor.position();
        if position > 0 {
            self.content.remove(position - 1..position);
            self.cursor.move_left();
            true
        } else {
            false
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            self.content.remove(start..end);
            self.cursor.set_position(start);
            self.selection.clear();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::actions::EditorAction;

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

    #[test]
    fn test_delete_previous_word() {
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(11); // After "world"
        doc.delete_previous_word();
        assert_eq!(doc.content(), "Hello  test");
        assert_eq!(doc.cursor_position(), 6); // Start of deleted word
    }

    #[test]
    fn test_delete_next_word() {
        let mut doc = TextDocument::with_content("Hello world test".to_string());
        doc.set_cursor_position(6); // Before "world"
        doc.delete_next_word();
        assert_eq!(doc.content(), "Hello  test");
        assert_eq!(doc.cursor_position(), 6); // Cursor stays in place
    }

    #[test]
    fn test_delete_to_line_start() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(6); // Before "world"
        doc.delete_to_line_start();
        assert_eq!(doc.content(), "world");
        assert_eq!(doc.cursor_position(), 0);
    }

    #[test]
    fn test_delete_to_line_end() {
        let mut doc = TextDocument::with_content("Hello world".to_string());
        doc.set_cursor_position(5); // After "Hello"
        doc.delete_to_line_end();
        assert_eq!(doc.content(), "Hello");
        assert_eq!(doc.cursor_position(), 5);
    }

    #[test]
    fn test_delete_current_line() {
        // Test deleting current line with multiple lines
        let mut doc = TextDocument::with_content("line1\nline2\nline3".to_string());
        doc.set_cursor_position(8); // In "line2"
        doc.delete_current_line();
        assert_eq!(doc.content(), "line1\nline3");
        assert_eq!(doc.cursor_position(), 6); // Start of "line3"

        // Test deleting first line
        let mut doc = TextDocument::with_content("first\nsecond\nthird".to_string());
        doc.set_cursor_position(2); // In "first"
        doc.delete_current_line();
        assert_eq!(doc.content(), "second\nthird");
        assert_eq!(doc.cursor_position(), 0); // Start of document

        // Test deleting last line
        let mut doc = TextDocument::with_content("first\nsecond\nlast".to_string());
        doc.set_cursor_position(15); // In "last"
        doc.delete_current_line();
        assert_eq!(doc.content(), "first\nsecond\n");
        assert_eq!(doc.cursor_position(), 13); // End of content

        // Test deleting single line
        let mut doc = TextDocument::with_content("only line".to_string());
        doc.set_cursor_position(5); // In "only line"
        doc.delete_current_line();
        assert_eq!(doc.content(), "");
        assert_eq!(doc.cursor_position(), 0);
    }

    #[test]
    fn test_cursor_movement_clears_selection() {
        use crate::input::actions::{EditorAction, Movement};
        
        let mut doc = TextDocument::with_content("line1\nline2\nline3".to_string());
        
        // Start at position 3 (in "line1")
        doc.set_cursor_position(3);
        
        // Create a selection by extending right
        doc.extend_selection_right(); // Select "e"
        assert!(doc.has_selection());
        assert_eq!(doc.selected_text(), Some("e".to_string()));
        
        // Test MoveCursor action (without Shift) clears selection
        doc.handle_action(EditorAction::MoveCursor(Movement::Up));
        assert!(!doc.has_selection(), "MoveCursor(Up) action should clear selection");
        
        // Create selection again
        doc.set_cursor_position(3);
        doc.extend_selection_right();
        assert!(doc.has_selection());
        
        // Test MoveCursor action (without Shift) clears selection  
        doc.handle_action(EditorAction::MoveCursor(Movement::Down));
        assert!(!doc.has_selection(), "MoveCursor(Down) action should clear selection");
        
        // Test that ExtendSelection action (with Shift) does NOT clear selection
        doc.set_cursor_position(3);
        doc.extend_selection_right();
        assert!(doc.has_selection());
        let initial_length = doc.selected_text().unwrap().len();
        
        doc.handle_action(EditorAction::ExtendSelection(Movement::Right));
        assert!(doc.has_selection(), "ExtendSelection action should NOT clear selection");
        let extended_length = doc.selected_text().unwrap().len();
        assert!(extended_length > initial_length, "Selection should be extended, not cleared");
        
        // Test other movement actions clear selection
        doc.set_cursor_position(3);
        doc.extend_selection_right();
        assert!(doc.has_selection());
        doc.handle_action(EditorAction::MoveCursor(Movement::LineStart));
        assert!(!doc.has_selection(), "MoveCursor(LineStart) should clear selection");
    }

    // ENG-144: Rope data structure integration tests
    #[test]
    fn test_rope_text_document_creation() {
        // RED: This test should fail because RopeTextDocument doesn't exist yet
        let rope_doc = RopeTextDocument::new();
        assert!(rope_doc.is_empty());
        assert_eq!(rope_doc.cursor_position(), 0);
        assert!(!rope_doc.has_selection());
    }

    #[test]
    fn test_rope_text_insertion() {
        // RED: This test should fail because insert_char method doesn't exist yet
        let mut rope_doc = RopeTextDocument::new();
        
        rope_doc.insert_char('H');
        assert_eq!(rope_doc.content(), "H");
        assert_eq!(rope_doc.cursor_position(), 1);
        
        rope_doc.insert_char('i');
        assert_eq!(rope_doc.content(), "Hi");
        assert_eq!(rope_doc.cursor_position(), 2);
    }

    #[test]
    fn test_rope_api_compatibility() {
        // Test that RopeTextDocument provides same API as TextDocument
        let mut string_doc = TextDocument::new();
        let mut rope_doc = RopeTextDocument::new();
        
        // Both should start empty
        assert_eq!(string_doc.is_empty(), rope_doc.is_empty());
        assert_eq!(string_doc.cursor_position(), rope_doc.cursor_position());
        
        // Both should handle identical operations
        string_doc.insert_char('A');
        rope_doc.insert_char('A');
        
        assert_eq!(string_doc.content(), rope_doc.content());
        assert_eq!(string_doc.cursor_position(), rope_doc.cursor_position());
        
        // Test insert_text method compatibility
        string_doc.insert_text("BC");
        rope_doc.insert_text("BC");
        
        assert_eq!(string_doc.content(), rope_doc.content());
        assert_eq!(string_doc.cursor_position(), rope_doc.cursor_position());
    }

    #[test]
    fn test_rope_with_content_creation() {
        let rope_doc = RopeTextDocument::with_content("Hello World".to_string());
        assert_eq!(rope_doc.content(), "Hello World");
        assert_eq!(rope_doc.cursor_position(), 11);
        assert!(!rope_doc.has_selection());
    }

    #[test]
    fn test_rope_deletion_operations() {
        let mut rope_doc = RopeTextDocument::with_content("Hello World".to_string());
        
        // Test delete_char
        rope_doc.set_cursor_position(5); // Before " World"
        let result = rope_doc.delete_char();
        assert!(result);
        assert_eq!(rope_doc.content(), "HelloWorld");
        assert_eq!(rope_doc.cursor_position(), 5);
        
        // Test backspace
        let result = rope_doc.backspace();
        assert!(result);
        assert_eq!(rope_doc.content(), "HellWorld");
        assert_eq!(rope_doc.cursor_position(), 4);
    }

    #[test]
    fn test_rope_selection_operations() {
        let mut rope_doc = RopeTextDocument::with_content("Hello World".to_string());
        
        // Test selection
        rope_doc.set_cursor_position(6);
        rope_doc.start_selection();
        rope_doc.set_cursor_position(11);
        
        assert!(rope_doc.has_selection());
        assert_eq!(rope_doc.selected_text(), Some("World".to_string()));
        
        // Test selection deletion
        let result = rope_doc.delete_selection();
        assert!(result);
        assert_eq!(rope_doc.content(), "Hello ");
        assert_eq!(rope_doc.cursor_position(), 6);
        assert!(!rope_doc.has_selection());
    }

    #[test]
    fn test_rope_insertion_with_selection() {
        let mut rope_doc = RopeTextDocument::with_content("Hello World".to_string());
        
        // Select "World"
        rope_doc.set_cursor_position(6);
        rope_doc.start_selection();
        rope_doc.set_cursor_position(11);
        
        // Insert should replace selection
        rope_doc.insert_char('!');
        assert_eq!(rope_doc.content(), "Hello !");
        assert_eq!(rope_doc.cursor_position(), 7);
        assert!(!rope_doc.has_selection());
    }

    // Performance test for rope-optimized cursor operations (ENG-146)
    #[test]
    fn test_cursor_performance_on_large_document() {
        // Create a large document (1000 lines)
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("This is line number {} with some sample text.\n", i));
        }
        
        let mut doc = TextDocument::with_content(content);
        
        // Test cursor operations on large document should be fast (O(log n))
        // Move to middle of document
        doc.set_cursor_position(25000);
        
        // Test line-based operations that should benefit from rope optimization
        doc.move_cursor_down();
        doc.move_cursor_up();
        doc.move_to_line_start();
        doc.move_to_line_end();
        
        // Test word navigation
        doc.move_to_word_start();
        doc.move_to_word_end();
        
        // Test selection operations
        doc.extend_selection_right();
        doc.extend_selection_left();
        
        // All operations should complete quickly without performance degradation
        assert!(doc.cursor_position() < doc.content.len_chars());
    }

    #[test]
    fn test_line_column_conversion_accuracy() {
        let content = "First line\nSecond line has more text\nThird\n\nFifth line";
        let mut doc = TextDocument::with_content(content.to_string());
        
        // Content: "First line\nSecond line has more text\nThird\n\nFifth line"
        //           0123456789012345678901234567890123456789012345678901
        //                     ^10         ^20         ^30         ^40
        
        // Test various positions
        doc.set_cursor_position(0);  // Start of first line
        assert_eq!(doc.get_cursor_line_and_column(), (0, 0));
        
        doc.set_cursor_position(5);  // Middle of first line  
        assert_eq!(doc.get_cursor_line_and_column(), (0, 5));
        
        doc.set_cursor_position(10); // End of first line
        assert_eq!(doc.get_cursor_line_and_column(), (0, 10));
        
        doc.set_cursor_position(11); // Start of second line (after \n)
        assert_eq!(doc.get_cursor_line_and_column(), (1, 0));
        
        doc.set_cursor_position(20); // Middle of second line
        assert_eq!(doc.get_cursor_line_and_column(), (1, 9));
        
        doc.set_cursor_position(35); // End of second line
        assert_eq!(doc.get_cursor_line_and_column(), (1, 24));
        
        // Position 36 is the newline character at end of second line
        doc.set_cursor_position(36); // Newline at end of second line
        assert_eq!(doc.get_cursor_line_and_column(), (1, 25)); // Still line 1, position 25
        
        doc.set_cursor_position(37); // Start of third line (after \n) 
        assert_eq!(doc.get_cursor_line_and_column(), (2, 0));
        
        doc.set_cursor_position(42); // Newline after "Third"
        assert_eq!(doc.get_cursor_line_and_column(), (2, 5)); // Line 2, at the newline
        
        doc.set_cursor_position(43); // Empty line (the second \n)
        assert_eq!(doc.get_cursor_line_and_column(), (3, 0)); // Line 3, column 0
        
        doc.set_cursor_position(44); // Start of "Fifth line"
        assert_eq!(doc.get_cursor_line_and_column(), (4, 0)); // Line 4, column 0
    }

    #[test]
    fn test_undo_redo_integration() {
        let mut doc = TextDocument::new();
        
        // Initially no undo/redo available
        assert!(!doc.can_undo());
        assert!(!doc.can_redo());
        
        // Execute a command using the command system
        let command = Box::new(InsertCommand::new(0, "Hello".to_string()));
        assert!(doc.execute_command(command));
        
        assert_eq!(doc.content(), "Hello");
        assert!(doc.can_undo());
        assert!(!doc.can_redo());
        
        // Undo the command
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "");
        assert!(!doc.can_undo());
        assert!(doc.can_redo());
        
        // Redo the command
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "Hello");
        assert!(doc.can_undo());
        assert!(!doc.can_redo());
    }

    #[test]
    fn test_multiple_commands_undo_redo() {
        let mut doc = TextDocument::new();
        
        // Execute multiple commands
        doc.execute_command(Box::new(InsertCommand::new(0, "Hello".to_string())));
        doc.execute_command(Box::new(InsertCommand::new(5, " World".to_string())));
        doc.execute_command(Box::new(InsertCommand::new(11, "!".to_string())));
        
        assert_eq!(doc.content(), "Hello World!");
        
        // Undo all commands
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "Hello World");
        
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "Hello");
        
        assert!(doc.perform_undo());
        assert_eq!(doc.content(), "");
        
        // No more undos available
        assert!(!doc.perform_undo());
        
        // Redo commands
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "Hello");
        
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "Hello World");
        
        assert!(doc.perform_redo());
        assert_eq!(doc.content(), "Hello World!");
        
        // No more redos available
        assert!(!doc.perform_redo());
    }

    #[test]
    fn test_command_clears_redo_stack() {
        let mut doc = TextDocument::new();
        
        // Execute and undo a command
        doc.execute_command(Box::new(InsertCommand::new(0, "Hello".to_string())));
        doc.perform_undo();
        
        assert!(doc.can_redo());
        
        // Execute a new command - should clear redo stack
        doc.execute_command(Box::new(InsertCommand::new(0, "World".to_string())));
        
        assert!(!doc.can_redo());
        assert_eq!(doc.content(), "World");
    }

    #[test]
    fn test_undo_redo_actions() {
        let mut doc = TextDocument::new();
        
        // Test that undo/redo actions return false when no history
        assert!(!doc.handle_action(EditorAction::Undo));
        assert!(!doc.handle_action(EditorAction::Redo));
        
        // Execute a command manually to have history
        doc.execute_command(Box::new(InsertCommand::new(0, "Test".to_string())));
        
        // Now undo action should work
        assert!(doc.handle_action(EditorAction::Undo));
        assert_eq!(doc.content(), "");
        
        // Now redo action should work
        assert!(doc.handle_action(EditorAction::Redo));
        assert_eq!(doc.content(), "Test");
    }
}