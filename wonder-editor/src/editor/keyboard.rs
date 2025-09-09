use gpui::{ClipboardItem, Context, KeyDownEvent, Window};

use super::MarkdownEditor;
use super::cursor_diagnostics::{
    log_keyboard_input, log_cursor_movement, log_selection_change,
    set_diagnostics_enabled, diagnostics_enabled, generate_diagnostic_report
};
use crate::core::{CoordinateConversion, RopeCoordinateMapper};
use ropey::Rope;

impl MarkdownEditor {
    // Key event handler for special keys that don't go through EntityInputHandler
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Special handling for diagnostics toggle (Ctrl+Shift+D)
        if event.keystroke.modifiers.control && event.keystroke.modifiers.shift {
            if event.keystroke.key == "d" || event.keystroke.key == "D" {
                // Toggle diagnostics
                let enabled = !diagnostics_enabled();
                set_diagnostics_enabled(enabled);
                
                if enabled {
                    // Generate and print diagnostic report
                    let content = self.document.content();
                    let rope = Rope::from_str(&content);
                    let mapper = RopeCoordinateMapper::new(rope);
                    let cursor_offset = self.document.cursor_position();
                    let cursor_point = mapper.offset_to_point(cursor_offset);
                    let selection = self.document.selection_range();
                    
                    let report = generate_diagnostic_report(
                        cursor_offset,
                        cursor_point,
                        selection,
                        &content,
                        {
                            // TODO: Get actual viewport height - using placeholder for now
                            500.0 
                        }
                    );
                    
                    eprintln!("{}", report);
                }
                
                cx.notify();
                return;
            }
        }
        
        // Special handling for clipboard operations that need GPUI context
        let is_cmd_or_ctrl = event.keystroke.modifiers.platform || event.keystroke.modifiers.control;
        
        if is_cmd_or_ctrl {
            match event.keystroke.key.as_str() {
                "c" => {
                    // Copy to system clipboard
                    if let Some(text) = self.document.copy() {
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                    }
                    cx.notify();
                    return;
                }
                "x" => {
                    // Cut to system clipboard
                    if let Some(text) = self.document.cut() {
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                    }
                    cx.notify();
                    return;
                }
                "v" => {
                    // Paste from system clipboard
                    let clipboard_text = cx.read_from_clipboard().and_then(|item| {
                        item.text()
                    });
                    self.document.paste(clipboard_text);
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }
        
        // Capture state before handling
        let old_position = self.document.cursor_position();
        let old_selection = self.document.selection_range();
        let content = self.document.content();
        let rope = Rope::from_str(&content);
        let mapper = RopeCoordinateMapper::new(rope);
        let old_point = mapper.offset_to_point(old_position);
        
        // The CursorMovementService handles visual cursor position tracking internally
        
        // Check if this is a navigation command that should use visual lines
        let key_binding = crate::input::keymap::KeyBinding {
            key: event.keystroke.key.clone(),
            modifiers: crate::input::keymap::Modifiers::from_gpui(&event.keystroke.modifiers),
        };
        
        if let Some(action) = self.input_router.keymap().get(&key_binding) {
            match action {
                crate::input::actions::EditorAction::MoveCursor(movement) => {
                    if self.cursor_movement.move_cursor(
                        movement.clone(),
                        &mut self.document,
                        self.hybrid_renderer.line_wrapper(),
                        &self.visual_line_manager,
                        false, // not extending selection
                    ) {
                        cx.notify();
                        
                        // Log the movement
                        let new_position = self.document.cursor_position();
                        let new_content = self.document.content();
                        let new_rope = Rope::from_str(&new_content);
                        let new_mapper = RopeCoordinateMapper::new(new_rope);
                        let _new_point = new_mapper.offset_to_point(new_position);
                        
                        log_keyboard_input(
                            &event.keystroke.key,
                            &format!("{:?}", event.keystroke.modifiers),
                            old_position,
                            new_position,
                            self.document.has_selection()
                        );
                        
                        log_cursor_movement(
                            &format!("Visual {} key", event.keystroke.key),
                            old_position,
                            new_position,
                            old_point,
                            _new_point,
                            &new_content
                        );
                        
                        // CRITICAL: Ensure cursor remains visible after movement
                        self.ensure_cursor_visible();
                        
                        return; // Skip normal InputRouter handling
                    }
                }
                crate::input::actions::EditorAction::ExtendSelection(movement) => {
                    if self.cursor_movement.move_cursor(
                        movement.clone(),
                        &mut self.document,
                        self.hybrid_renderer.line_wrapper(),
                        &self.visual_line_manager,
                        true, // extending selection
                    ) {
                        cx.notify();
                        
                        // Log the movement
                        let new_position = self.document.cursor_position();
                        let new_content = self.document.content();
                        let new_rope = Rope::from_str(&new_content);
                        let new_mapper = RopeCoordinateMapper::new(new_rope);
                        let _new_point = new_mapper.offset_to_point(new_position);
                        
                        log_keyboard_input(
                            &event.keystroke.key,
                            &format!("{:?}", event.keystroke.modifiers),
                            old_position,
                            new_position,
                            self.document.has_selection()
                        );
                        
                        log_cursor_movement(
                            &format!("Visual selection {} key", event.keystroke.key),
                            old_position,
                            new_position,
                            old_point,
                            _new_point,
                            &new_content
                        );
                        
                        // CRITICAL: Ensure cursor remains visible after selection extension
                        self.ensure_cursor_visible();
                        
                        return; // Skip normal InputRouter handling
                    }
                }
                _ => {} // Let other actions fall through to normal handling
            }
        }
        
        // Use the new InputRouter for keyboard handling
        let handled = self.input_router.handle_key_event(event, &mut self.document);
        
        // If the keyboard event was handled, ensure cursor visibility
        // This covers actions like Delete, Backspace, and other cursor-moving operations
        if handled {
            self.ensure_cursor_visible();
        }
        
        // Special handling for Enter key (newline)
        if event.keystroke.key == "enter" {
            self.input_router.handle_char_input('\n', &mut self.document);
            cx.notify();
            // Ensure cursor remains visible after Enter
            self.ensure_cursor_visible();
            
            // Log the operation
            let new_position = self.document.cursor_position();
            let new_content = self.document.content();
            let new_rope = Rope::from_str(&new_content);
            let new_mapper = RopeCoordinateMapper::new(new_rope);
            let _new_point = new_mapper.offset_to_point(new_position);
            
            log_keyboard_input(
                "enter",
                &format!("{:?}", event.keystroke.modifiers),
                old_position,
                new_position,
                self.document.has_selection()
            );
            
            log_cursor_movement(
                "Enter key",
                old_position,
                new_position,
                old_point,
                _new_point,
                &new_content
            );
            
            return;
        }
        
        // Special handling for Tab key
        if event.keystroke.key == "tab" {
            self.input_router.handle_char_input('\t', &mut self.document);
            cx.notify();
            
            // Log the operation
            let new_position = self.document.cursor_position();
            let new_content = self.document.content();
            let new_rope = Rope::from_str(&new_content);
            let new_mapper = RopeCoordinateMapper::new(new_rope);
            let _new_point = new_mapper.offset_to_point(new_position);
            
            log_keyboard_input(
                "tab",
                &format!("{:?}", event.keystroke.modifiers),
                old_position,
                new_position,
                self.document.has_selection()
            );
            
            return;
        }
        
        if handled {
            // Log the operation
            let new_position = self.document.cursor_position();
            let new_selection = self.document.selection_range();
            let new_content = self.document.content();
            let new_rope = Rope::from_str(&new_content);
            let new_mapper = RopeCoordinateMapper::new(new_rope);
            let _new_point = new_mapper.offset_to_point(new_position);
            
            // Log keyboard input
            log_keyboard_input(
                &event.keystroke.key,
                &format!("{:?}", event.keystroke.modifiers),
                old_position,
                new_position,
                self.document.has_selection()
            );
            
            // Log cursor movement if position changed
            if old_position != new_position {
                log_cursor_movement(
                    &format!("Keyboard: {}", event.keystroke.key),
                    old_position,
                    new_position,
                    old_point,
                    _new_point,
                    &new_content
                );
            }
            
            // Log selection change if selection changed
            if old_selection != new_selection {
                log_selection_change(
                    &format!("Keyboard: {}", event.keystroke.key),
                    new_selection.map(|(s, _)| s),
                    new_selection.map(|(_, e)| e),
                    new_position,
                    &new_content
                );
            }
            
            cx.notify();
        }
    }
}