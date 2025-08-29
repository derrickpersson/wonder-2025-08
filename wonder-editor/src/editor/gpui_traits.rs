use gpui::{
    div, px, rgb, Bounds, Context, EntityInputHandler, Focusable, FocusHandle, IntoElement, 
    InteractiveElement, ParentElement, Pixels, Point, Render, Styled, UTF16Selection, Window,
};

use super::{element::EditorElement, MarkdownEditor};

// GPUI trait implementations for MarkdownEditor
impl Focusable for MarkdownEditor {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for MarkdownEditor {
    fn text_for_range(
        &mut self,
        range_utf16: std::ops::Range<usize>,
        _actual_range: &mut Option<std::ops::Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        // Convert UTF-16 range to UTF-8 and return text
        let content = self.document.content();
        if range_utf16.start >= content.len() {
            return Some(String::new());
        }
        let end = range_utf16.end.min(content.len());
        Some(content[range_utf16.start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        // Return current cursor position as selection
        let cursor_pos = self.document.cursor_position();
        Some(UTF16Selection {
            range: cursor_pos..cursor_pos,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<std::ops::Range<usize>> {
        // No marked text support for now
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        // No marked text support for now
    }

    fn replace_text_in_range(
        &mut self,
        _range_utf16: Option<std::ops::Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Check if we're actually focused - if not, focus first
        let is_focused = self.focus_handle.is_focused(window);

        if !is_focused {
            window.focus(&self.focus_handle);
            self.focused = true;
        }

        // Insert each character from the new text
        for ch in new_text.chars() {
            self.handle_char_input(ch);
        }

        // CRITICAL: Notify GPUI that the entity state has changed so it re-renders
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<std::ops::Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<std::ops::Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // For now, just delegate to replace_text_in_range
        self.replace_text_in_range(range_utf16, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        _range: std::ops::Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // For now, return None - this would be used for IME candidate positioning
        None
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        // ENG-137: Basic click-to-position implementation
        // Convert screen point to character index in the document
        let padding = px(16.0);
        let line_height = px(24.0);
        
        // Calculate which line was clicked based on y-coordinate
        let relative_y = point.y - padding;
        let line_index = (relative_y / line_height).floor() as usize;
        
        let content = self.document.content();
        let lines: Vec<&str> = content.lines().collect();
        
        // If clicked beyond last line, return end of document
        if line_index >= lines.len() {
            let end_pos = content.chars().count();
            self.handle_click_at_position(end_pos);
            return Some(end_pos);
        }
        
        // Calculate character position within the clicked line
        let line_content = lines[line_index];
        let relative_x = point.x - padding;
        
        // Simple character width approximation (will be improved in later tickets)
        let font_size = px(16.0);
        let approx_char_width = font_size * 0.6; // Rough approximation for now
        let char_index_in_line = ((relative_x / approx_char_width).floor() as usize).min(line_content.chars().count());
        
        // Calculate absolute position in document
        let chars_before_line: usize = lines.iter().take(line_index).map(|l| l.chars().count() + 1).sum(); // +1 for newline
        let absolute_position = chars_before_line + char_index_in_line;
        
        // Update cursor position directly (this is called on the MarkdownEditor itself)
        self.handle_click_at_position(absolute_position);
        
        Some(absolute_position)
    }
}

impl Render for MarkdownEditor {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.document.content().to_string();
        let cursor_position = self.document.cursor_position();
        let selection = if self.document.has_selection() {
            if let Some((start, end)) = self.document.selection_range() {
                Some(start..end)
            } else {
                None
            }
        } else {
            None
        };

        // Sync our internal focused state with GPUI's focus system
        let is_gpui_focused = self.focus_handle.is_focused(window);
        self.focused = is_gpui_focused;

        // Always ensure the editor is focused on startup
        if !is_gpui_focused {
            window.focus(&self.focus_handle);
        }
        self.focused = true; // Force focused state

        // Use a simple div with action handlers that wraps our hybrid editor
        div()
            .track_focus(&self.focus_handle)
            // ENG-137/138: Mouse event handlers for click-to-position and drag selection
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_up),
            )
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_key_down(cx.listener(Self::handle_key_down))
            .size_full()
            .flex()
            .flex_col()
            .child(
                // Status bar
                div()
                    .h(px(30.0))
                    .w_full()
                    .bg(rgb(0x1e1e2e))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .flex()
                    .items_center()
                    .px_4()
                    .child(
                        div()
                            .text_color(rgb(0xa6adc8))
                            .text_size(px(14.0))
                            .child("Hybrid Preview - Edit anywhere!"),
                    ),
            )
            .child(
                // Main content area with hybrid editor
                div().flex_1().w_full().p_4().child(
                    // Use EditorElement with hybrid rendering capabilities - USE THE EDITOR'S RENDERER
                    EditorElement {
                        editor: cx.entity().clone(),
                        content,
                        focused: self.focused,
                        focus_handle: self.focus_handle.clone(),
                        cursor_position,
                        selection,
                        hybrid_renderer: self.hybrid_renderer.clone(),
                    },
                ),
            )
    }
}