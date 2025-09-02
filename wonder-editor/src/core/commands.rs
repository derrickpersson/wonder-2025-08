//! Command Pattern Implementation for Undo/Redo System
//!
//! This module implements the Command Pattern foundation that will enable
//! undo/redo functionality throughout the editor. All text operations
//! will be converted to commands that can be executed, undone, and redone.

use ropey::Rope;

/// Trait for commands that can be executed, undone, and redone
/// Commands must be cloneable and debuggable for use in the command history system
pub trait UndoableCommand: std::fmt::Debug + Send + Sync {
    /// Execute the command on the given rope, returning the modified rope
    fn execute(&self, rope: &Rope) -> Rope;
    
    /// Undo the command on the given rope, returning the previous state
    fn undo(&self, rope: &Rope) -> Rope;
    
    /// Get a description of this command for debugging
    fn description(&self) -> &str;
    
    /// Clone this command
    fn clone_command(&self) -> Box<dyn UndoableCommand>;
}

/// Insert text at a specific position
#[derive(Debug, Clone)]
pub struct InsertCommand {
    position: usize,
    text: String,
}

impl InsertCommand {
    pub fn new(position: usize, text: String) -> Self {
        Self { position, text }
    }
}

impl UndoableCommand for InsertCommand {
    fn execute(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        new_rope.insert(self.position, &self.text);
        new_rope
    }
    
    fn undo(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        let end_pos = self.position + self.text.chars().count();
        new_rope.remove(self.position..end_pos);
        new_rope
    }
    
    fn description(&self) -> &str {
        "Insert text"
    }
    
    fn clone_command(&self) -> Box<dyn UndoableCommand> {
        Box::new(self.clone())
    }
}

/// Delete text within a specific range
#[derive(Debug, Clone)]
pub struct DeleteCommand {
    start: usize,
    end: usize,
    deleted_text: String,
}

impl DeleteCommand {
    pub fn new(start: usize, end: usize, deleted_text: String) -> Self {
        Self { start, end, deleted_text }
    }
}

impl UndoableCommand for DeleteCommand {
    fn execute(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        new_rope.remove(self.start..self.end);
        new_rope
    }
    
    fn undo(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        new_rope.insert(self.start, &self.deleted_text);
        new_rope
    }
    
    fn description(&self) -> &str {
        "Delete text"
    }
    
    fn clone_command(&self) -> Box<dyn UndoableCommand> {
        Box::new(self.clone())
    }
}

/// Replace text within a specific range
#[derive(Debug, Clone)]
pub struct ReplaceCommand {
    start: usize,
    end: usize,
    old_text: String,
    new_text: String,
}

impl ReplaceCommand {
    pub fn new(start: usize, end: usize, old_text: String, new_text: String) -> Self {
        Self { start, end, old_text, new_text }
    }
}

impl UndoableCommand for ReplaceCommand {
    fn execute(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        new_rope.remove(self.start..self.end);
        new_rope.insert(self.start, &self.new_text);
        new_rope
    }
    
    fn undo(&self, rope: &Rope) -> Rope {
        let mut new_rope = rope.clone();
        let end_pos = self.start + self.new_text.chars().count();
        new_rope.remove(self.start..end_pos);
        new_rope.insert(self.start, &self.old_text);
        new_rope
    }
    
    fn description(&self) -> &str {
        "Replace text"
    }
    
    fn clone_command(&self) -> Box<dyn UndoableCommand> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_command_execute() {
        let rope = Rope::from_str("Hello World");
        let command = InsertCommand::new(6, "Beautiful ".to_string());
        
        let result = command.execute(&rope);
        assert_eq!(result.to_string(), "Hello Beautiful World");
    }

    #[test]
    fn test_insert_command_undo() {
        let rope = Rope::from_str("Hello Beautiful World");
        let command = InsertCommand::new(6, "Beautiful ".to_string());
        
        let result = command.undo(&rope);
        assert_eq!(result.to_string(), "Hello World");
    }

    #[test]
    fn test_delete_command_execute() {
        let rope = Rope::from_str("Hello Beautiful World");
        let command = DeleteCommand::new(6, 16, "Beautiful ".to_string());
        
        let result = command.execute(&rope);
        assert_eq!(result.to_string(), "Hello World");
    }

    #[test]
    fn test_delete_command_undo() {
        let rope = Rope::from_str("Hello World");
        let command = DeleteCommand::new(6, 6, "Beautiful ".to_string());
        
        let result = command.undo(&rope);
        assert_eq!(result.to_string(), "Hello Beautiful World");
    }

    #[test]
    fn test_replace_command_execute() {
        let rope = Rope::from_str("Hello World");
        let command = ReplaceCommand::new(6, 11, "World".to_string(), "Universe".to_string());
        
        let result = command.execute(&rope);
        assert_eq!(result.to_string(), "Hello Universe");
    }

    #[test]
    fn test_replace_command_undo() {
        // Start with text after the execute command has been applied
        let rope = Rope::from_str("Hello World");
        let command = ReplaceCommand::new(6, 14, "Universe".to_string(), "World".to_string());
        
        let result = command.undo(&rope);
        assert_eq!(result.to_string(), "Hello Universe");
    }

    #[test]
    fn test_command_descriptions() {
        let insert_cmd = InsertCommand::new(0, "test".to_string());
        let delete_cmd = DeleteCommand::new(0, 4, "test".to_string());
        let replace_cmd = ReplaceCommand::new(0, 4, "old".to_string(), "new".to_string());
        
        assert_eq!(insert_cmd.description(), "Insert text");
        assert_eq!(delete_cmd.description(), "Delete text");
        assert_eq!(replace_cmd.description(), "Replace text");
    }
}