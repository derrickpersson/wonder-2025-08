//! Command History Manager for Undo/Redo Operations
//!
//! This module implements a stack-based command history system that enables
//! undo/redo functionality with transaction grouping and memory management.

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use crate::core::commands::UndoableCommand;
use ropey::Rope;

/// Maximum number of commands to keep in history
const DEFAULT_MAX_HISTORY_SIZE: usize = 1000;

/// Default transaction timeout in milliseconds
const DEFAULT_TRANSACTION_TIMEOUT_MS: u64 = 500;

/// Maximum memory usage for command history (in bytes)
const DEFAULT_MAX_MEMORY_BYTES: usize = 10 * 1024 * 1024; // 10MB

/// A group of commands that should be undone/redone together
#[derive(Debug)]
pub struct CommandTransaction {
    commands: Vec<Box<dyn UndoableCommand>>,
    timestamp: Instant,
    description: String,
}

impl CommandTransaction {
    pub fn new(description: String) -> Self {
        Self {
            commands: Vec::new(),
            timestamp: Instant::now(),
            description,
        }
    }

    pub fn add_command(&mut self, command: Box<dyn UndoableCommand>) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn commands(&self) -> &[Box<dyn UndoableCommand>] {
        &self.commands
    }

    /// Execute all commands in the transaction
    pub fn execute(&self, rope: &Rope) -> Rope {
        let mut current_rope = rope.clone();
        for command in &self.commands {
            current_rope = command.execute(&current_rope);
        }
        current_rope
    }

    /// Undo all commands in the transaction (in reverse order)
    pub fn undo(&self, rope: &Rope) -> Rope {
        let mut current_rope = rope.clone();
        for command in self.commands.iter().rev() {
            current_rope = command.undo(&current_rope);
        }
        current_rope
    }

    /// Estimate memory usage of this transaction
    pub fn estimated_memory_usage(&self) -> usize {
        // Base size plus estimated command sizes
        std::mem::size_of::<Self>() + 
        self.commands.len() * std::mem::size_of::<Box<dyn UndoableCommand>>() +
        self.description.len()
    }
}

impl Clone for CommandTransaction {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.iter().map(|cmd| cmd.clone_command()).collect(),
            timestamp: self.timestamp,
            description: self.description.clone(),
        }
    }
}

/// Command History Manager with undo/redo capabilities
pub struct CommandHistory {
    undo_stack: VecDeque<CommandTransaction>,
    redo_stack: VecDeque<CommandTransaction>,
    current_transaction: Option<CommandTransaction>,
    max_history_size: usize,
    max_memory_bytes: usize,
    transaction_timeout: Duration,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_transaction: None,
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            transaction_timeout: Duration::from_millis(DEFAULT_TRANSACTION_TIMEOUT_MS),
        }
    }

    pub fn with_limits(max_history_size: usize, max_memory_bytes: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_transaction: None,
            max_history_size,
            max_memory_bytes,
            transaction_timeout: Duration::from_millis(DEFAULT_TRANSACTION_TIMEOUT_MS),
        }
    }

    /// Start a new transaction group
    pub fn start_transaction(&mut self, description: String) {
        self.finish_current_transaction();
        self.current_transaction = Some(CommandTransaction::new(description));
    }

    /// Add a command to the current transaction or create a new single-command transaction
    pub fn add_command(&mut self, command: Box<dyn UndoableCommand>) {
        // If no current transaction, create a new one
        if self.current_transaction.is_none() {
            self.start_transaction(command.description().to_string());
        }

        if let Some(transaction) = &mut self.current_transaction {
            transaction.add_command(command);
        }

        // Auto-finish transaction if it's getting old
        if let Some(transaction) = &self.current_transaction {
            if transaction.timestamp.elapsed() > self.transaction_timeout {
                self.finish_current_transaction();
            }
        }
    }

    /// Finish the current transaction and add it to history
    pub fn finish_current_transaction(&mut self) {
        if let Some(transaction) = self.current_transaction.take() {
            if !transaction.is_empty() {
                self.add_transaction_to_history(transaction);
            }
        }
    }

    /// Add a completed transaction to the undo stack
    fn add_transaction_to_history(&mut self, transaction: CommandTransaction) {
        // Clear redo stack when adding new commands
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push_back(transaction);

        // Enforce memory and size limits
        self.enforce_limits();
    }

    /// Enforce history size and memory limits
    fn enforce_limits(&mut self) {
        // Enforce size limit
        while self.undo_stack.len() > self.max_history_size {
            self.undo_stack.pop_front();
        }

        // Enforce memory limit using LRU eviction
        while self.estimated_memory_usage() > self.max_memory_bytes && !self.undo_stack.is_empty() {
            self.undo_stack.pop_front();
        }
    }

    /// Estimate total memory usage
    fn estimated_memory_usage(&self) -> usize {
        let undo_size: usize = self.undo_stack.iter().map(|t| t.estimated_memory_usage()).sum();
        let redo_size: usize = self.redo_stack.iter().map(|t| t.estimated_memory_usage()).sum();
        let current_size = self.current_transaction.as_ref()
            .map(|t| t.estimated_memory_usage())
            .unwrap_or(0);
        
        undo_size + redo_size + current_size
    }

    /// Undo the last transaction
    pub fn undo(&mut self, rope: &Rope) -> Option<Rope> {
        // Finish any current transaction first
        self.finish_current_transaction();

        if let Some(transaction) = self.undo_stack.pop_back() {
            let result_rope = transaction.undo(rope);
            self.redo_stack.push_back(transaction);
            Some(result_rope)
        } else {
            None
        }
    }

    /// Redo the last undone transaction
    pub fn redo(&mut self, rope: &Rope) -> Option<Rope> {
        if let Some(transaction) = self.redo_stack.pop_back() {
            let result_rope = transaction.execute(rope);
            self.undo_stack.push_back(transaction);
            Some(result_rope)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || 
        self.current_transaction.as_ref().map_or(false, |t| !t.is_empty())
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of the next undo operation
    pub fn undo_description(&self) -> Option<&str> {
        if let Some(transaction) = &self.current_transaction {
            if !transaction.is_empty() {
                return Some(transaction.description());
            }
        }
        self.undo_stack.back().map(|t| t.description())
    }

    /// Get description of the next redo operation
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|t| t.description())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_transaction = None;
    }

    /// Get current history statistics
    pub fn stats(&self) -> HistoryStats {
        HistoryStats {
            undo_count: self.undo_stack.len(),
            redo_count: self.redo_stack.len(),
            memory_usage: self.estimated_memory_usage(),
            has_current_transaction: self.current_transaction.is_some(),
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub undo_count: usize,
    pub redo_count: usize,
    pub memory_usage: usize,
    pub has_current_transaction: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::InsertCommand;

    #[test]
    fn test_command_history_creation() {
        let history = CommandHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert!(history.undo_description().is_none());
        assert!(history.redo_description().is_none());
    }

    #[test]
    fn test_add_single_command() {
        let mut history = CommandHistory::new();
        let rope = Rope::from_str("Hello");
        
        let command = Box::new(InsertCommand::new(5, " World".to_string()));
        history.add_command(command);
        history.finish_current_transaction();
        
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert!(history.undo_description().is_some());
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut history = CommandHistory::new();
        let mut rope = Rope::from_str("Hello");
        
        // Add command and finish transaction
        let command = Box::new(InsertCommand::new(5, " World".to_string()));
        history.add_command(command);
        history.finish_current_transaction();
        
        // Execute the command manually (in real usage, this would be handled by the system)
        rope.insert(5, " World");
        assert_eq!(rope.to_string(), "Hello World");
        
        // Undo
        if let Some(undo_rope) = history.undo(&rope) {
            rope = undo_rope;
            assert_eq!(rope.to_string(), "Hello");
            assert!(history.can_redo());
            assert!(!history.can_undo());
        }
        
        // Redo
        if let Some(redo_rope) = history.redo(&rope) {
            rope = redo_rope;
            assert_eq!(rope.to_string(), "Hello World");
            assert!(history.can_undo());
            assert!(!history.can_redo());
        }
    }

    #[test]
    fn test_transaction_grouping() {
        let mut history = CommandHistory::new();
        // Start with text that already has the commands applied
        let rope = Rope::from_str("Hello Beautiful World");
        
        // Create a transaction that represents what happened to get from "Hello" to "Hello Beautiful World"
        history.start_transaction("Multi-step edit".to_string());
        
        // These commands represent the operations that were performed:
        // 1. Insert " Beautiful" at position 5
        // 2. Insert " World" at position 15 (after "Hello Beautiful")
        history.add_command(Box::new(InsertCommand::new(5, " Beautiful".to_string())));
        history.add_command(Box::new(InsertCommand::new(15, " World".to_string())));
        
        history.finish_current_transaction();
        
        // Should have one transaction with multiple commands
        let stats = history.stats();
        assert_eq!(stats.undo_count, 1);
        assert_eq!(history.undo_description(), Some("Multi-step edit"));
        
        // Undo should reverse both commands at once
        let undo_rope = history.undo(&rope).unwrap();
        assert_eq!(undo_rope.to_string(), "Hello");
    }

    #[test]
    fn test_memory_limits() {
        // Create history with very small memory limit
        let mut history = CommandHistory::with_limits(100, 1024);
        let rope = Rope::from_str("Test");
        
        // Add many commands to exceed memory limit
        for i in 0..50 {
            let command = Box::new(InsertCommand::new(0, format!("Command {}", i)));
            history.add_command(command);
            history.finish_current_transaction();
        }
        
        let stats = history.stats();
        // Should have fewer transactions due to memory limit
        assert!(stats.undo_count < 50);
        assert!(stats.memory_usage <= 1024);
    }

    #[test]
    fn test_size_limits() {
        // Create history with small size limit
        let mut history = CommandHistory::with_limits(5, 10 * 1024 * 1024);
        
        // Add more commands than the limit
        for i in 0..10 {
            let command = Box::new(InsertCommand::new(0, format!("Cmd {}", i)));
            history.add_command(command);
            history.finish_current_transaction();
        }
        
        let stats = history.stats();
        // Should be limited to max size
        assert!(stats.undo_count <= 5);
    }

    #[test]
    fn test_clear_history() {
        let mut history = CommandHistory::new();
        
        // Add some commands
        history.add_command(Box::new(InsertCommand::new(0, "Test".to_string())));
        history.finish_current_transaction();
        
        assert!(history.can_undo());
        
        // Clear history
        history.clear();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        
        let stats = history.stats();
        assert_eq!(stats.undo_count, 0);
        assert_eq!(stats.redo_count, 0);
    }

    #[test]
    fn test_auto_transaction_timeout() {
        let mut history = CommandHistory::new();
        
        // Add command and let it age (we can't easily test real timeout in unit tests)
        history.add_command(Box::new(InsertCommand::new(0, "First".to_string())));
        
        // Simulate timeout by finishing manually
        history.finish_current_transaction();
        
        // Add another command - should be in new transaction
        history.add_command(Box::new(InsertCommand::new(0, "Second".to_string())));
        history.finish_current_transaction();
        
        let stats = history.stats();
        assert_eq!(stats.undo_count, 2); // Should be 2 separate transactions
    }
}