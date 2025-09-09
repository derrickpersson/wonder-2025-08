//! Keymap system for mapping keyboard inputs to editor actions
//!
//! This module provides a configurable system for binding keyboard shortcuts
//! to editor actions, allowing for customizable and extensible input handling.

use super::actions::{EditorAction, Movement, FormatType};
use std::collections::HashMap;

/// Represents a keyboard shortcut with key and modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Modifiers,
}

/// Modifier keys that can be held during a keypress
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,     // Command key on Mac, Windows key on Windows
}

impl KeyBinding {
    /// Create a new key binding with no modifiers
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            modifiers: Modifiers::default(),
        }
    }

    /// Create a new key binding with modifiers
    pub fn with_modifiers(key: &str, modifiers: Modifiers) -> Self {
        Self {
            key: key.to_string(),
            modifiers,
        }
    }
}

impl Modifiers {
    /// No modifiers
    pub fn none() -> Self {
        Self::default()
    }
    
    /// Only Shift
    pub fn shift() -> Self {
        Self { shift: true, ..Default::default() }
    }
    
    /// Only Cmd (Command/Windows)
    pub fn cmd() -> Self {
        Self { cmd: true, ..Default::default() }
    }
    
    /// Only Alt
    pub fn alt() -> Self {
        Self { alt: true, ..Default::default() }
    }
    
    /// Only Ctrl
    pub fn ctrl() -> Self {
        Self { ctrl: true, ..Default::default() }
    }
    
    /// Cmd + Shift
    pub fn cmd_shift() -> Self {
        Self { cmd: true, shift: true, ..Default::default() }
    }
    
    /// Ctrl + Shift
    pub fn ctrl_shift() -> Self {
        Self { ctrl: true, shift: true, ..Default::default() }
    }

    /// Convert from GPUI modifiers
    pub fn from_gpui(gpui_modifiers: &gpui::Modifiers) -> Self {
        Self {
            ctrl: gpui_modifiers.control,
            alt: gpui_modifiers.alt,
            shift: gpui_modifiers.shift,
            cmd: gpui_modifiers.platform, // platform = cmd on macOS
        }
    }
}

/// Maps keyboard shortcuts to editor actions
#[derive(Debug)]
pub struct Keymap {
    bindings: HashMap<KeyBinding, EditorAction>,
}

impl Keymap {
    /// Create a new empty keymap
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Create the default keymap with standard editor shortcuts
    pub fn default() -> Self {
        let mut keymap = Self::new();
        keymap.add_default_bindings();
        keymap
    }

    /// Add a key binding
    pub fn bind(&mut self, key_binding: KeyBinding, action: EditorAction) {
        self.bindings.insert(key_binding, action);
    }

    /// Remove a key binding
    pub fn unbind(&mut self, key_binding: &KeyBinding) -> Option<EditorAction> {
        self.bindings.remove(key_binding)
    }

    /// Get the action for a key binding
    pub fn get(&self, key_binding: &KeyBinding) -> Option<&EditorAction> {
        self.bindings.get(key_binding)
    }

    /// Add all default key bindings
    fn add_default_bindings(&mut self) {
        // Basic movement
        self.bind(KeyBinding::new("left"), EditorAction::MoveCursor(Movement::Left));
        self.bind(KeyBinding::new("right"), EditorAction::MoveCursor(Movement::Right));
        self.bind(KeyBinding::new("up"), EditorAction::MoveCursor(Movement::Up));
        self.bind(KeyBinding::new("down"), EditorAction::MoveCursor(Movement::Down));

        // Line movement on macOS (Cmd + Arrow) - Fixed from word movement
        self.bind(
            KeyBinding::with_modifiers("left", Modifiers::cmd()),
            EditorAction::MoveCursor(Movement::LineStart)
        );
        self.bind(
            KeyBinding::with_modifiers("right", Modifiers::cmd()),
            EditorAction::MoveCursor(Movement::LineEnd)
        );
        
        // Word movement on macOS (Option + Arrow) - New
        self.bind(
            KeyBinding::with_modifiers("left", Modifiers::alt()),
            EditorAction::MoveCursor(Movement::WordStart)
        );
        self.bind(
            KeyBinding::with_modifiers("right", Modifiers::alt()),
            EditorAction::MoveCursor(Movement::WordEnd)
        );

        // Document movement (Cmd/Ctrl + Up/Down)
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers::cmd()),
            EditorAction::MoveCursor(Movement::DocumentStart)
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers::cmd()),
            EditorAction::MoveCursor(Movement::DocumentEnd)
        );

        // Line movement (Home/End)
        self.bind(KeyBinding::new("home"), EditorAction::MoveCursor(Movement::LineStart));
        self.bind(KeyBinding::new("end"), EditorAction::MoveCursor(Movement::LineEnd));

        // Selection extension (Shift + movement)
        self.bind(
            KeyBinding::with_modifiers("left", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::Left)
        );
        self.bind(
            KeyBinding::with_modifiers("right", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::Right)
        );
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::Up)
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::Down)
        );

        // Line selection on macOS (Cmd + Shift + Arrow) - Fixed from word selection
        self.bind(
            KeyBinding::with_modifiers("left", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::LineStart)
        );
        self.bind(
            KeyBinding::with_modifiers("right", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::LineEnd)
        );
        
        // Word selection on macOS (Option + Shift + Arrow) - New  
        self.bind(
            KeyBinding::with_modifiers("left", Modifiers { alt: true, shift: true, ..Default::default() }),
            EditorAction::ExtendSelection(Movement::WordStart)
        );
        self.bind(
            KeyBinding::with_modifiers("right", Modifiers { alt: true, shift: true, ..Default::default() }),
            EditorAction::ExtendSelection(Movement::WordEnd)
        );

        // Document selection (Cmd/Ctrl + Shift + Up/Down)
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::DocumentStart)
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::DocumentEnd)
        );

        // Text operations
        self.bind(KeyBinding::new("backspace"), EditorAction::Backspace);
        self.bind(KeyBinding::new("delete"), EditorAction::Delete);

        // Page navigation
        self.bind(KeyBinding::new("pageup"), EditorAction::PageUp);
        self.bind(KeyBinding::new("pagedown"), EditorAction::PageDown);

        // ENG-191: Keyboard scroll navigation
        // Alt+Up/Down - Scroll without moving cursor (commonly used in editors)
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers::alt()),
            EditorAction::ScrollUp
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers::alt()),
            EditorAction::ScrollDown
        );
        
        // Function keys for scroll navigation (avoiding conflicts)
        // F6/F7 - Scroll up/down (alternative bindings)
        self.bind(
            KeyBinding::new("f6"),
            EditorAction::ScrollUp
        );
        self.bind(
            KeyBinding::new("f7"),
            EditorAction::ScrollDown
        );
        
        // Alt+PageUp/PageDown - Scroll by page (avoiding Ctrl conflicts)
        self.bind(
            KeyBinding::with_modifiers("pageup", Modifiers::alt()),
            EditorAction::ScrollPageUp
        );
        self.bind(
            KeyBinding::with_modifiers("pagedown", Modifiers::alt()),
            EditorAction::ScrollPageDown
        );
        
        // Ctrl/Cmd + Alt + Up/Down - Scroll to top/bottom (unique combination)
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers { cmd: true, alt: true, ..Default::default() }),
            EditorAction::ScrollToTop
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers { cmd: true, alt: true, ..Default::default() }),
            EditorAction::ScrollToBottom
        );
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers { ctrl: true, alt: true, ..Default::default() }),
            EditorAction::ScrollToTop
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers { ctrl: true, alt: true, ..Default::default() }),
            EditorAction::ScrollToBottom
        );

        // Select all (Cmd/Ctrl + A)
        self.bind(
            KeyBinding::with_modifiers("a", Modifiers::cmd()),
            EditorAction::SelectAll
        );

        // Text formatting (Cmd/Ctrl + B/I)
        self.bind(
            KeyBinding::with_modifiers("b", Modifiers::cmd()),
            EditorAction::ToggleFormat(FormatType::Bold)
        );
        self.bind(
            KeyBinding::with_modifiers("i", Modifiers::cmd()),
            EditorAction::ToggleFormat(FormatType::Italic)
        );
        
        // Clipboard operations (Cmd/Ctrl + C/X/V)
        self.bind(
            KeyBinding::with_modifiers("c", Modifiers::cmd()),
            EditorAction::Copy
        );
        self.bind(
            KeyBinding::with_modifiers("x", Modifiers::cmd()),
            EditorAction::Cut
        );
        self.bind(
            KeyBinding::with_modifiers("v", Modifiers::cmd()),
            EditorAction::Paste
        );
        
        // Also bind Ctrl versions for cross-platform support
        self.bind(
            KeyBinding::with_modifiers("c", Modifiers::ctrl()),
            EditorAction::Copy
        );
        self.bind(
            KeyBinding::with_modifiers("x", Modifiers::ctrl()),
            EditorAction::Cut
        );
        self.bind(
            KeyBinding::with_modifiers("v", Modifiers::ctrl()),
            EditorAction::Paste
        );
        
        // Undo/Redo operations (Cmd/Ctrl + Z, Shift+Cmd/Ctrl + Z)
        self.bind(
            KeyBinding::with_modifiers("z", Modifiers::cmd()),
            EditorAction::Undo
        );
        self.bind(
            KeyBinding::with_modifiers("z", Modifiers::cmd_shift()),
            EditorAction::Redo
        );
        
        // Also bind Ctrl versions for cross-platform support
        self.bind(
            KeyBinding::with_modifiers("z", Modifiers::ctrl()),
            EditorAction::Undo
        );
        self.bind(
            KeyBinding::with_modifiers("z", Modifiers::ctrl_shift()),
            EditorAction::Redo
        );
        
        // Alternative Redo shortcut (Ctrl/Cmd + Y) - common on Windows
        self.bind(
            KeyBinding::with_modifiers("y", Modifiers::cmd()),
            EditorAction::Redo
        );
        self.bind(
            KeyBinding::with_modifiers("y", Modifiers::ctrl()),
            EditorAction::Redo
        );
        
        // Missing navigation shortcuts from ENG-133
        
        // Shift+Home/End - Extend selection to line boundaries
        self.bind(
            KeyBinding::with_modifiers("home", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::LineStart)
        );
        self.bind(
            KeyBinding::with_modifiers("end", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::LineEnd)
        );
        
        // Ctrl+Shift+Home/End - Extend selection to document boundaries (Windows/Linux)
        self.bind(
            KeyBinding::with_modifiers("home", Modifiers::ctrl_shift()),
            EditorAction::ExtendSelection(Movement::DocumentStart)
        );
        self.bind(
            KeyBinding::with_modifiers("end", Modifiers::ctrl_shift()),
            EditorAction::ExtendSelection(Movement::DocumentEnd)
        );
        
        // Cmd+Shift+Home/End - Extend selection to document boundaries (macOS) 
        self.bind(
            KeyBinding::with_modifiers("home", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::DocumentStart)
        );
        self.bind(
            KeyBinding::with_modifiers("end", Modifiers::cmd_shift()),
            EditorAction::ExtendSelection(Movement::DocumentEnd)
        );
        
        // Escape - Clear selection
        self.bind(
            KeyBinding::new("escape"),
            EditorAction::ClearSelection
        );
        
        // Additional page navigation shortcuts
        // Shift+PageUp/Down - Extend selection by page
        self.bind(
            KeyBinding::with_modifiers("pageup", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::PageUp)
        );
        self.bind(
            KeyBinding::with_modifiers("pagedown", Modifiers::shift()),
            EditorAction::ExtendSelection(Movement::PageDown)
        );
        
        // Ctrl+Shift+Up/Down - Page selection on macOS (alternative to Shift+PageUp/Down)
        self.bind(
            KeyBinding::with_modifiers("up", Modifiers::ctrl_shift()),
            EditorAction::ExtendSelection(Movement::PageUp)
        );
        self.bind(
            KeyBinding::with_modifiers("down", Modifiers::ctrl_shift()),
            EditorAction::ExtendSelection(Movement::PageDown)
        );

        // Advanced deletion shortcuts
        
        // Word deletion: Option+Backspace/Delete (macOS) and Ctrl+Backspace/Delete (cross-platform)
        self.bind(
            KeyBinding::with_modifiers("backspace", Modifiers::alt()),
            EditorAction::DeletePreviousWord
        );
        self.bind(
            KeyBinding::with_modifiers("backspace", Modifiers::ctrl()),
            EditorAction::DeletePreviousWord
        );
        self.bind(
            KeyBinding::with_modifiers("delete", Modifiers::alt()),
            EditorAction::DeleteNextWord
        );
        self.bind(
            KeyBinding::with_modifiers("delete", Modifiers::ctrl()),
            EditorAction::DeleteNextWord
        );
        
        // Line deletion: Cmd+Backspace/Delete (macOS only)
        self.bind(
            KeyBinding::with_modifiers("backspace", Modifiers::cmd()),
            EditorAction::DeleteToLineStart
        );
        self.bind(
            KeyBinding::with_modifiers("delete", Modifiers::cmd()),
            EditorAction::DeleteToLineEnd
        );
        
        // Current line deletion: Cmd+Shift+K (macOS) and Ctrl+Shift+K (cross-platform)
        self.bind(
            KeyBinding::with_modifiers("k", Modifiers::cmd_shift()),
            EditorAction::DeleteCurrentLine
        );
        self.bind(
            KeyBinding::with_modifiers("k", Modifiers::ctrl_shift()),
            EditorAction::DeleteCurrentLine
        );
    }

    /// Get all key bindings (for debugging/inspection)
    pub fn all_bindings(&self) -> &HashMap<KeyBinding, EditorAction> {
        &self.bindings
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding_creation() {
        let binding = KeyBinding::new("left");
        assert_eq!(binding.key, "left");
        assert_eq!(binding.modifiers, Modifiers::default());
    }

    #[test]
    fn test_key_binding_with_modifiers() {
        let binding = KeyBinding::with_modifiers("a", Modifiers::cmd());
        assert_eq!(binding.key, "a");
        assert!(binding.modifiers.cmd);
        assert!(!binding.modifiers.shift);
    }

    #[test]
    fn test_modifiers() {
        let cmd_shift = Modifiers::cmd_shift();
        assert!(cmd_shift.cmd);
        assert!(cmd_shift.shift);
        assert!(!cmd_shift.ctrl);
        assert!(!cmd_shift.alt);
    }

    #[test]
    fn test_keymap_basic_operations() {
        let mut keymap = Keymap::new();
        let binding = KeyBinding::new("left");
        let action = EditorAction::MoveCursor(Movement::Left);

        keymap.bind(binding.clone(), action.clone());
        assert_eq!(keymap.get(&binding), Some(&action));

        let removed = keymap.unbind(&binding);
        assert_eq!(removed, Some(action));
        assert_eq!(keymap.get(&binding), None);
    }

    #[test]
    fn test_default_keymap() {
        let keymap = Keymap::default();
        
        // Test basic movement
        let left_binding = KeyBinding::new("left");
        assert_eq!(
            keymap.get(&left_binding),
            Some(&EditorAction::MoveCursor(Movement::Left))
        );

        // Test line movement on macOS (Cmd+Left/Right - fixed from word movement)
        let cmd_left_binding = KeyBinding::with_modifiers("left", Modifiers::cmd());
        assert_eq!(
            keymap.get(&cmd_left_binding),
            Some(&EditorAction::MoveCursor(Movement::LineStart))
        );

        // Test word movement on macOS (Option+Left/Right - new)
        let alt_left_binding = KeyBinding::with_modifiers("left", Modifiers::alt());
        assert_eq!(
            keymap.get(&alt_left_binding),
            Some(&EditorAction::MoveCursor(Movement::WordStart))
        );

        // Test line selection on macOS (Cmd+Shift+Left/Right - fixed from word selection)
        let cmd_shift_left_binding = KeyBinding::with_modifiers("left", Modifiers::cmd_shift());
        assert_eq!(
            keymap.get(&cmd_shift_left_binding),
            Some(&EditorAction::ExtendSelection(Movement::LineStart))
        );

        // Test new shortcuts from ENG-133
        
        // Test Shift+Home/End for line selection
        let shift_home_binding = KeyBinding::with_modifiers("home", Modifiers::shift());
        assert_eq!(
            keymap.get(&shift_home_binding),
            Some(&EditorAction::ExtendSelection(Movement::LineStart))
        );
        
        // Test Escape for clearing selection
        let escape_binding = KeyBinding::new("escape");
        assert_eq!(
            keymap.get(&escape_binding),
            Some(&EditorAction::ClearSelection)
        );
        
        // Test Shift+Up/Down for vertical selection (bug fix)
        let shift_up_binding = KeyBinding::with_modifiers("up", Modifiers::shift());
        assert_eq!(
            keymap.get(&shift_up_binding),
            Some(&EditorAction::ExtendSelection(Movement::Up))
        );
        
        let shift_down_binding = KeyBinding::with_modifiers("down", Modifiers::shift());
        assert_eq!(
            keymap.get(&shift_down_binding),
            Some(&EditorAction::ExtendSelection(Movement::Down))
        );

        // Test advanced deletion shortcuts from ENG-132
        
        // Test word deletion shortcuts (Option+Backspace for macOS, Ctrl+Backspace for cross-platform)
        let alt_backspace_binding = KeyBinding::with_modifiers("backspace", Modifiers::alt());
        assert_eq!(
            keymap.get(&alt_backspace_binding),
            Some(&EditorAction::DeletePreviousWord)
        );
        
        let ctrl_backspace_binding = KeyBinding::with_modifiers("backspace", Modifiers::ctrl());
        assert_eq!(
            keymap.get(&ctrl_backspace_binding),
            Some(&EditorAction::DeletePreviousWord)
        );
        
        let alt_delete_binding = KeyBinding::with_modifiers("delete", Modifiers::alt());
        assert_eq!(
            keymap.get(&alt_delete_binding),
            Some(&EditorAction::DeleteNextWord)
        );
        
        // Test line deletion shortcuts (Cmd+Backspace/Delete for macOS)
        let cmd_backspace_binding = KeyBinding::with_modifiers("backspace", Modifiers::cmd());
        assert_eq!(
            keymap.get(&cmd_backspace_binding),
            Some(&EditorAction::DeleteToLineStart)
        );
        
        let cmd_delete_binding = KeyBinding::with_modifiers("delete", Modifiers::cmd());
        assert_eq!(
            keymap.get(&cmd_delete_binding),
            Some(&EditorAction::DeleteToLineEnd)
        );
        
        // Test current line deletion shortcut (Cmd+Shift+K for macOS, Ctrl+Shift+K for cross-platform)
        let cmd_shift_k_binding = KeyBinding::with_modifiers("k", Modifiers::cmd_shift());
        assert_eq!(
            keymap.get(&cmd_shift_k_binding),
            Some(&EditorAction::DeleteCurrentLine)
        );
        
        let ctrl_shift_k_binding = KeyBinding::with_modifiers("k", Modifiers::ctrl_shift());
        assert_eq!(
            keymap.get(&ctrl_shift_k_binding),
            Some(&EditorAction::DeleteCurrentLine)
        );

        // Test enhanced page selection shortcuts from ENG-134
        
        // Test fixed Shift+PageUp/Down for proper page selection (not just Up/Down)
        let shift_pageup_binding = KeyBinding::with_modifiers("pageup", Modifiers::shift());
        assert_eq!(
            keymap.get(&shift_pageup_binding),
            Some(&EditorAction::ExtendSelection(Movement::PageUp))
        );
        
        let shift_pagedown_binding = KeyBinding::with_modifiers("pagedown", Modifiers::shift());
        assert_eq!(
            keymap.get(&shift_pagedown_binding),
            Some(&EditorAction::ExtendSelection(Movement::PageDown))
        );
        
        // Test Ctrl+Shift+Up/Down for macOS page selection
        let ctrl_shift_up_binding = KeyBinding::with_modifiers("up", Modifiers::ctrl_shift());
        assert_eq!(
            keymap.get(&ctrl_shift_up_binding),
            Some(&EditorAction::ExtendSelection(Movement::PageUp))
        );
        
        let ctrl_shift_down_binding = KeyBinding::with_modifiers("down", Modifiers::ctrl_shift());
        assert_eq!(
            keymap.get(&ctrl_shift_down_binding),
            Some(&EditorAction::ExtendSelection(Movement::PageDown))
        );
        
        // Test undo/redo keyboard shortcuts from ENG-176
        
        // Test Cmd+Z for undo (macOS)
        let cmd_z_binding = KeyBinding::with_modifiers("z", Modifiers::cmd());
        assert_eq!(
            keymap.get(&cmd_z_binding),
            Some(&EditorAction::Undo)
        );
        
        // Test Ctrl+Z for undo (cross-platform)
        let ctrl_z_binding = KeyBinding::with_modifiers("z", Modifiers::ctrl());
        assert_eq!(
            keymap.get(&ctrl_z_binding),
            Some(&EditorAction::Undo)
        );
        
        // Test Cmd+Shift+Z for redo (macOS)
        let cmd_shift_z_binding = KeyBinding::with_modifiers("z", Modifiers::cmd_shift());
        assert_eq!(
            keymap.get(&cmd_shift_z_binding),
            Some(&EditorAction::Redo)
        );
        
        // Test Ctrl+Shift+Z for redo (cross-platform)
        let ctrl_shift_z_binding = KeyBinding::with_modifiers("z", Modifiers::ctrl_shift());
        assert_eq!(
            keymap.get(&ctrl_shift_z_binding),
            Some(&EditorAction::Redo)
        );
        
        // Test alternative Cmd+Y for redo (macOS)
        let cmd_y_binding = KeyBinding::with_modifiers("y", Modifiers::cmd());
        assert_eq!(
            keymap.get(&cmd_y_binding),
            Some(&EditorAction::Redo)
        );
        
        // Test alternative Ctrl+Y for redo (Windows/Linux)
        let ctrl_y_binding = KeyBinding::with_modifiers("y", Modifiers::ctrl());
        assert_eq!(
            keymap.get(&ctrl_y_binding),
            Some(&EditorAction::Redo)
        );
    }

    #[test]
    fn test_gpui_modifiers_conversion() {
        let gpui_modifiers = gpui::Modifiers {
            control: false,
            alt: false,
            shift: true,
            platform: true,
            function: false,
        };

        let modifiers = Modifiers::from_gpui(&gpui_modifiers);
        assert!(!modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(modifiers.shift);
        assert!(modifiers.cmd);
    }
}