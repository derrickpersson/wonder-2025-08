//! Input routing system that maps keyboard events to editor actions
//!
//! This module provides the InputRouter which serves as the central dispatcher
//! for keyboard events, converting them to actions and routing them to handlers.

use super::actions::{EditorAction, ActionHandler};
use super::keymap::{Keymap, KeyBinding, Modifiers};
use gpui::KeyDownEvent;

/// Central router for handling keyboard input events
#[derive(Debug)]
pub struct InputRouter {
    keymap: Keymap,
    debug_mode: bool,
}

impl InputRouter {
    /// Create a new input router with the default keymap
    pub fn new() -> Self {
        Self {
            keymap: Keymap::default(),
            debug_mode: false, // Debug mode off by default
        }
    }

    /// Create a new input router with a custom keymap
    pub fn with_keymap(keymap: Keymap) -> Self {
        Self {
            keymap,
            debug_mode: false, // Debug mode off by default
        }
    }

    /// Enable or disable debug mode (prints key events to console)
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    /// Handle a keyboard event and route it to the appropriate action handler
    pub fn handle_key_event<T: ActionHandler>(
        &self,
        event: &KeyDownEvent,
        target: &mut T,
    ) -> bool {
        let key_binding = KeyBinding {
            key: event.keystroke.key.clone(),
            modifiers: Modifiers::from_gpui(&event.keystroke.modifiers),
        };

        if self.debug_mode {
            println!("InputRouter: Key event: {:?}", key_binding);
        }

        if let Some(action) = self.keymap.get(&key_binding) {
            if self.debug_mode {
                println!("InputRouter: Executing action: {:?}", action);
            }
            target.handle_action(action.clone())
        } else {
            if self.debug_mode {
                println!("InputRouter: No action found for key binding: {:?}", key_binding);
            }
            false
        }
    }

    /// Handle character input (for printable characters)
    pub fn handle_char_input<T: ActionHandler>(
        &self,
        ch: char,
        target: &mut T,
    ) -> bool {
        if self.debug_mode {
            println!("InputRouter: Character input: '{}'", ch);
        }
        target.handle_action(EditorAction::InsertChar(ch))
    }

    /// Get a reference to the current keymap
    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// Get a mutable reference to the current keymap for customization
    pub fn keymap_mut(&mut self) -> &mut Keymap {
        &mut self.keymap
    }

    /// Replace the current keymap
    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.keymap = keymap;
    }

    /// Add a single key binding
    pub fn bind_key(&mut self, key_binding: KeyBinding, action: EditorAction) {
        self.keymap.bind(key_binding, action);
    }

    /// Remove a key binding
    pub fn unbind_key(&mut self, key_binding: &KeyBinding) -> Option<EditorAction> {
        self.keymap.unbind(key_binding)
    }
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::actions::Movement;
    use crate::core::TextDocument;

    #[test]
    fn test_input_router_creation() {
        let router = InputRouter::new();
        assert!(!router.debug_mode);
        assert!(router.keymap().get(&KeyBinding::new("left")).is_some());
    }

    #[test]
    fn test_custom_keymap() {
        let mut keymap = Keymap::new();
        keymap.bind(
            KeyBinding::new("x"),
            EditorAction::MoveCursor(Movement::Right),
        );

        let router = InputRouter::with_keymap(keymap);
        assert_eq!(
            router.keymap().get(&KeyBinding::new("x")),
            Some(&EditorAction::MoveCursor(Movement::Right))
        );
    }

    #[test]
    fn test_debug_mode() {
        let mut router = InputRouter::new();
        assert!(!router.debug_mode);
        
        router.set_debug_mode(true);
        assert!(router.debug_mode);
        
        router.set_debug_mode(false);
        assert!(!router.debug_mode);
    }

    #[test]
    fn test_char_input_handling() {
        let router = InputRouter::new();
        let mut document = TextDocument::new();

        let handled = router.handle_char_input('a', &mut document);
        assert!(handled);
        assert_eq!(document.content(), "a");
        assert_eq!(document.cursor_position(), 1);
    }

    #[test]
    fn test_key_binding_management() {
        let mut router = InputRouter::new();
        let binding = KeyBinding::new("test");
        let action = EditorAction::SelectAll;

        // Add binding
        router.bind_key(binding.clone(), action.clone());
        assert_eq!(router.keymap().get(&binding), Some(&action));

        // Remove binding
        let removed = router.unbind_key(&binding);
        assert_eq!(removed, Some(action));
        assert!(router.keymap().get(&binding).is_none());
    }

    #[test]
    fn test_keymap_replacement() {
        let mut router = InputRouter::new();
        let mut new_keymap = Keymap::new();
        new_keymap.bind(KeyBinding::new("custom"), EditorAction::SelectAll);

        router.set_keymap(new_keymap);
        assert!(router.keymap().get(&KeyBinding::new("left")).is_none()); // Old binding gone
        assert!(router.keymap().get(&KeyBinding::new("custom")).is_some()); // New binding present
    }

    // Note: Testing handle_key_event would require creating GPUI KeyDownEvent objects,
    // which is complex in a unit test environment. Integration tests would be better
    // for testing the full key event handling pipeline.
}