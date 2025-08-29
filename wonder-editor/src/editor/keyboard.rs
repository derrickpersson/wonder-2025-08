use gpui::{ClipboardItem, Context, KeyDownEvent, Window};

use super::MarkdownEditor;

impl MarkdownEditor {
    // Key event handler for special keys that don't go through EntityInputHandler
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        
        // Use the new InputRouter for keyboard handling
        let handled = self.input_router.handle_key_event(event, &mut self.document);
        
        // Special handling for Enter key (newline)
        if event.keystroke.key == "enter" {
            self.input_router.handle_char_input('\n', &mut self.document);
            cx.notify();
            return;
        }
        
        // Special handling for Tab key
        if event.keystroke.key == "tab" {
            self.input_router.handle_char_input('\t', &mut self.document);
            cx.notify();
            return;
        }
        
        if handled {
            cx.notify();
        }
    }
}