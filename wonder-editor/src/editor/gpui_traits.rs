use gpui::{
    div, Bounds, Context, EntityInputHandler, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Pixels, Point, Render, Styled, UTF16Selection, Window,
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
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        // ENG-137: Delegate to our sophisticated mouse positioning logic
        // This ensures consistent behavior with our actual mouse event handlers

        eprintln!(
            "🎯 GPUI character_index_for_point called with: ({:.1}, {:.1})px",
            point.x.0, point.y.0
        );

        // Use our accurate convert_point_to_character_index method
        let character_position = self.convert_point_to_character_index(point, window);
        self.handle_click_at_position(character_position);
        Some(character_position)
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
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_key_down(cx.listener(Self::handle_key_down))
            .size_full()
            .flex()
            .flex_col()
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
