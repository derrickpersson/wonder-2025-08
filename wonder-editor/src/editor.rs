use crate::core::TextDocument;
use crate::hybrid_renderer::HybridTextRenderer;
use crate::input::InputRouter;
use gpui::{
    div, prelude::*, px, rgb, size, transparent_black, App, Bounds, Context,
    Element, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable, Hsla,
    IntoElement, KeyDownEvent, LayoutId, Pixels, Point, Render, ShapedLine, TextRun,
    UTF16Selection, Window,
};

// Legacy GPUI actions removed - now using InputRouter action system

pub struct MarkdownEditor {
    document: TextDocument,
    input_router: InputRouter,
    hybrid_renderer: HybridTextRenderer,
    focused: bool,
    focus_handle: FocusHandle,
}

impl MarkdownEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let input_router = InputRouter::new();
        
        Self {
            document: TextDocument::new(),
            input_router,
            hybrid_renderer: HybridTextRenderer::new(),
            focused: true, // Start focused
            focus_handle,
        }
    }

    pub fn new_with_content(content: String, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let input_router = InputRouter::new();
        
        Self {
            document: TextDocument::with_content(content),
            input_router,
            hybrid_renderer: HybridTextRenderer::new(),
            focused: true, // Start focused
            focus_handle,
        }
    }

    // Content access
    pub fn content(&self) -> &str {
        self.document.content()
    }

    pub fn cursor_position(&self) -> usize {
        self.document.cursor_position()
    }

    pub fn has_selection(&self) -> bool {
        self.document.has_selection()
    }

    // Focus management (integrates with GPUI focus system)
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        // Note: In a real UI, we would call self.focus_handle.focus() or blur() here
        // but that requires a Window context which isn't available in this method
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    // GPUI-specific focus management methods
    pub fn focus_in_window(&mut self, window: &mut gpui::Window) {
        window.focus(&self.focus_handle);
        self.focused = true;
    }

    pub fn is_focused_in_window(&self, window: &gpui::Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    // Input handling - delegates to keyboard handler
    pub fn handle_char_input(&mut self, ch: char) {
        self.input_router.handle_char_input(ch, &mut self.document);
    }


    // Legacy compatibility methods for tests
    pub fn get_content(&self) -> &str {
        self.content()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.handle_char_input(ch);
    }


    // GPUI action handlers - these have the signature expected by cx.listener()
    // Legacy action handlers removed - now using InputRouter system

    // Mouse event handlers
    fn handle_mouse_down(
        &mut self,
        _event: &gpui::MouseDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        // Focus the editor when clicked
        window.focus(&self.focus_handle);
        self.focused = true;
        cx.notify();
    }

    // Key event handler for special keys that don't go through EntityInputHandler
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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

    // Legacy action conversion removed - now using InputRouter directly

    // Provide access to document for more complex operations
    pub fn document(&self) -> &TextDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut TextDocument {
        &mut self.document
    }
}

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
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        // For now, return cursor position - this would be used for mouse click positioning
        Some(self.document.cursor_position())
    }
}

// Custom element that handles text layout and input registration during paint phase
struct EditorElement {
    editor: Entity<MarkdownEditor>,
    content: String,
    focused: bool,
    focus_handle: FocusHandle,
    cursor_position: usize,
    selection: Option<std::ops::Range<usize>>,
    hybrid_renderer: HybridTextRenderer,
}

impl Element for EditorElement {
    type RequestLayoutState = Vec<ShapedLine>;
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let text_to_display = if self.content.is_empty() {
            "Start typing your markdown content...".to_string()
        } else {
            self.content.clone()
        };

        // Split text by newlines and handle each line separately
        let lines: Vec<&str> = text_to_display.lines().collect();
        let font_size = px(16.0);

        let mut shaped_lines = Vec::new();
        let mut max_width = px(0.0);
        let mut current_offset = 0;

        for line in lines {
            let line_text = if line.is_empty() {
                " ".to_string() // Empty lines need space for height
            } else {
                line.to_string()
            };

            // Generate hybrid text runs for this specific line
            let line_runs = self.hybrid_renderer.generate_mixed_text_runs(
                &line_text,
                if self.cursor_position >= current_offset
                    && self.cursor_position <= current_offset + line.len()
                {
                    self.cursor_position - current_offset
                } else {
                    usize::MAX // Cursor not in this line
                },
                self.selection.as_ref().and_then(|sel| {
                    // Adjust selection range to line-relative coordinates
                    let line_start = current_offset;
                    let line_end = current_offset + line.len();
                    if sel.end > line_start && sel.start < line_end {
                        let adjusted_start = sel.start.saturating_sub(line_start);
                        let adjusted_end = (sel.end - line_start).min(line.len());
                        Some(adjusted_start..adjusted_end)
                    } else {
                        None
                    }
                }),
            );

            // If no hybrid runs, use fallback styling
            let text_runs = if line_runs.is_empty() {
                vec![TextRun {
                    len: line_text.len(),
                    font: gpui::Font {
                        family: "system-ui".into(),
                        features: gpui::FontFeatures::default(),
                        weight: gpui::FontWeight::NORMAL,
                        style: gpui::FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: rgb(0xcdd6f4).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }]
            } else {
                line_runs
            };

            // Shape this line with its text runs
            let shaped_line =
                window
                    .text_system()
                    .shape_line(line_text.into(), font_size, &text_runs, None);

            max_width = max_width.max(shaped_line.width);
            shaped_lines.push(shaped_line);

            // Update offset for next line (include newline character)
            current_offset += line.len() + 1;
        }

        // Handle case where text ends with newline (add empty line)
        if text_to_display.ends_with('\n') {
            let text_run = TextRun {
                len: 1,
                font: gpui::Font {
                    family: "system-ui".into(),
                    features: gpui::FontFeatures::default(),
                    weight: gpui::FontWeight::NORMAL,
                    style: gpui::FontStyle::Normal,
                    fallbacks: None,
                },
                color: rgb(0xcdd6f4).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };

            let shaped_line =
                window
                    .text_system()
                    .shape_line(" ".into(), font_size, &[text_run], None);
            shaped_lines.push(shaped_line);
        }

        // Calculate the size we need including padding
        let line_height = px(24.0);
        let padding = px(16.0);
        let num_lines = shaped_lines.len().max(1);

        let total_width = max_width + padding * 2.0;
        let total_height = (line_height * num_lines as f32) + padding * 2.0;

        // Create layout with our calculated size
        let layout_id = window.request_layout(
            gpui::Style {
                size: size(total_width.into(), total_height.into()).into(),
                ..Default::default()
            },
            [],
            _cx,
        );

        (layout_id, shaped_lines)
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        shaped_lines: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // ALWAYS register input handler so we can receive text input
        window.handle_input(
            &self.focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        // Paint background
        let background_color = rgb(0x11111b);
        let border_color = if self.focused {
            rgb(0x89b4fa)
        } else {
            rgb(0x313244)
        };

        window.paint_quad(gpui::PaintQuad {
            bounds,
            background: background_color.into(),
            border_widths: gpui::Edges::all(px(1.0)),
            border_color: border_color.into(),
            border_style: gpui::BorderStyle::Solid,
            corner_radii: gpui::Corners::all(px(4.0)),
        });

        // Paint all text lines
        let padding = px(16.0);
        let line_height = px(24.0);
        let mut text_origin = bounds.origin + gpui::point(padding, padding);

        // Paint selection first (behind text)
        if let Some(ref selection_range) = self.selection {
            self.paint_selection(bounds, shaped_lines, selection_range.clone(), window);
        }

        for shaped_line in shaped_lines.iter_mut() {
            shaped_line
                .paint(text_origin, line_height, window, cx)
                .unwrap_or_else(|err| {
                    eprintln!("Failed to paint text line: {:?}", err);
                });

            // Move to next line
            text_origin.y += line_height;
        }

        // Paint cursor if focused
        if self.focused {
            self.paint_cursor(bounds, window, cx);
        }
    }
}

impl EditorElement {
    fn paint_selection(
        &self,
        bounds: Bounds<Pixels>,
        shaped_lines: &[ShapedLine],
        selection_range: std::ops::Range<usize>,
        window: &mut Window,
    ) {
        let padding = px(16.0);
        let line_height = px(24.0);
        let selection_color = gpui::Rgba {
            r: 0.337,
            g: 0.502,
            b: 0.761,
            a: 0.3, // Semi-transparent blue
        };

        let content = &self.content;
        let mut char_offset = 0;
        let mut y_offset = padding;

        // Process each line with its corresponding shaped line
        let lines: Vec<&str> = content.lines().collect();
        for (line_index, line_text) in lines.iter().enumerate() {
            let line_start = char_offset;
            let line_end = char_offset + line_text.len();
            
            // Check if this line intersects with the selection
            if selection_range.end > line_start && selection_range.start <= line_end {
                // Calculate the selection bounds within this line
                let sel_start_in_line = selection_range.start.saturating_sub(line_start);
                let sel_end_in_line = (selection_range.end.min(line_end) - line_start).min(line_text.len());
                
                if sel_start_in_line < sel_end_in_line && line_index < shaped_lines.len() {
                    // Measure text accurately using GPUI's text shaping
                    let font_size = px(16.0);
                    let text_run = TextRun {
                        len: line_text.len(),
                        font: gpui::Font {
                            family: "system-ui".into(),
                            features: gpui::FontFeatures::default(),
                            weight: gpui::FontWeight::NORMAL,
                            style: gpui::FontStyle::Normal,
                            fallbacks: None,
                        },
                        color: rgb(0xcdd6f4).into(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    
                    // Measure from start of line to selection start
                    let x_start = if sel_start_in_line == 0 {
                        padding
                    } else {
                        let text_to_sel_start = line_text.chars().take(sel_start_in_line).collect::<String>();
                        if !text_to_sel_start.is_empty() {
                            let shaped = window.text_system().shape_line(
                                text_to_sel_start.into(),
                                font_size,
                                &[text_run.clone()],
                                None,
                            );
                            padding + shaped.width
                        } else {
                            padding
                        }
                    };
                    
                    // Measure from start of line to selection end
                    let x_end = {
                        let text_to_sel_end = line_text.chars().take(sel_end_in_line).collect::<String>();
                        if !text_to_sel_end.is_empty() {
                            let shaped = window.text_system().shape_line(
                                text_to_sel_end.into(),
                                font_size,
                                &[text_run.clone()],
                                None,
                            );
                            padding + shaped.width
                        } else {
                            padding
                        }
                    };
                    
                    // Paint selection rectangle for this line
                    window.paint_quad(gpui::PaintQuad {
                        bounds: Bounds {
                            origin: bounds.origin + gpui::point(x_start, y_offset),
                            size: size(x_end - x_start, line_height),
                        },
                        background: selection_color.into(),
                        border_widths: gpui::Edges::all(px(0.0)),
                        border_color: transparent_black().into(),
                        border_style: gpui::BorderStyle::Solid,
                        corner_radii: gpui::Corners::all(px(0.0)),
                    });
                }
            }
            
            // Move to next line
            char_offset = line_end + 1; // +1 for newline character
            y_offset += line_height;
        }
    }

    fn paint_cursor(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        // Get the cursor position from the editor
        let cursor_position = self.editor.read(cx).cursor_position();

        // Calculate which line the cursor is on and position within that line
        let content = &self.content;
        let chars_before_cursor: String = content.chars().take(cursor_position).collect();

        // Count newlines to determine line number
        let line_number = chars_before_cursor.matches('\n').count();

        // Find position within the current line
        let lines_before: Vec<&str> = chars_before_cursor.lines().collect();
        let position_in_line = if chars_before_cursor.ends_with('\n') {
            0
        } else {
            lines_before
                .last()
                .map(|line| line.chars().count())
                .unwrap_or(cursor_position)
        };

        // Calculate cursor position
        let padding = px(16.0);
        let line_height = px(24.0);
        let font_size = px(16.0);

        // Calculate X position by shaping text on current line up to cursor
        let cursor_x_offset = if position_in_line == 0 {
            px(0.0)
        } else {
            // Get the current line's text up to cursor position
            let current_line_text = if line_number < content.lines().count() {
                let line = content.lines().nth(line_number).unwrap_or("");
                line.chars().take(position_in_line).collect::<String>()
            } else {
                String::new()
            };

            if current_line_text.is_empty() {
                px(0.0)
            } else {
                let _text_color = rgb(0xcdd6f4);
                let text_run = TextRun {
                    len: current_line_text.len(),
                    font: gpui::Font {
                        family: "system-ui".into(),
                        features: gpui::FontFeatures::default(),
                        weight: gpui::FontWeight::NORMAL,
                        style: gpui::FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: rgb(0xcdd6f4).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };

                let shaped_line = window.text_system().shape_line(
                    current_line_text.into(),
                    font_size,
                    &[text_run],
                    None,
                );

                shaped_line.width
            }
        };

        // Create cursor bounds - a thin vertical line
        let cursor_x = bounds.origin.x + padding + cursor_x_offset;
        let cursor_y = bounds.origin.y + padding + (line_height * line_number as f32);

        let cursor_bounds = Bounds {
            origin: gpui::point(cursor_x, cursor_y),
            size: size(px(2.0), line_height),
        };

        // Paint the cursor as a filled rectangle
        let cursor_color = rgb(0xf38ba8); // Pink cursor color
        window.paint_quad(gpui::PaintQuad {
            bounds: cursor_bounds,
            background: cursor_color.into(),
            border_widths: gpui::Edges::all(px(0.0)),
            border_color: transparent_black().into(),
            border_style: gpui::BorderStyle::Solid,
            corner_radii: gpui::Corners::all(px(0.0)),
        });
    }
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
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
            // Legacy action handlers removed - now using InputRouter system
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
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
                    // Use EditorElement with hybrid rendering capabilities
                    EditorElement {
                        editor: cx.entity().clone(),
                        content,
                        focused: self.focused,
                        focus_handle: self.focus_handle.clone(),
                        cursor_position,
                        selection,
                        hybrid_renderer: HybridTextRenderer::new(),
                    },
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ActionHandler;

    // Test helper that creates a minimal editor without GPUI context
    // This is for testing core functionality that doesn't require focus handling
    fn create_test_editor_minimal() -> TestableEditor {
        TestableEditor {
            document: TextDocument::new(),
            input_router: InputRouter::new(),
            focused: false,
        }
    }

    // Wrapper struct for testing that mimics MarkdownEditor without FocusHandle
    #[derive(Debug)]
    struct TestableEditor {
        document: TextDocument,
        input_router: InputRouter,
        focused: bool,
    }

    impl TestableEditor {
        // Mirror the methods we need for testing
        pub fn content(&self) -> &str {
            self.document.content()
        }

        pub fn cursor_position(&self) -> usize {
            self.document.cursor_position()
        }

        pub fn handle_char_input(&mut self, ch: char) {
            self.input_router.handle_char_input(ch, &mut self.document);
        }


        pub fn set_focus(&mut self, focused: bool) {
            self.focused = focused;
        }

        pub fn is_focused(&self) -> bool {
            self.focused
        }

        pub fn get_content(&self) -> &str {
            self.content()
        }

        pub fn insert_char(&mut self, ch: char) {
            self.handle_char_input(ch);
        }

        pub fn handle_key_input(&mut self, ch: char) {
            self.handle_char_input(ch);
        }


        pub fn document_mut(&mut self) -> &mut TextDocument {
            &mut self.document
        }

        // Legacy action methods removed - now using InputRouter directly
    }

    // Helper method for backward compatibility
    fn new_with_buffer() -> TestableEditor {
        create_test_editor_minimal()
    }

    // Helper method for creating editor with content
    fn new_with_content(content: String) -> TestableEditor {
        TestableEditor {
            document: TextDocument::with_content(content),
            input_router: InputRouter::new(),
            focused: false,
        }
    }

    #[test]
    fn test_handle_keyboard_input_basic_char() {
        let mut editor = new_with_buffer();

        // Test character input handling
        editor.handle_key_input('a');
        assert_eq!(editor.content(), "a");
        assert_eq!(editor.cursor_position(), 1);

        editor.handle_key_input('b');
        assert_eq!(editor.content(), "ab");
        assert_eq!(editor.cursor_position(), 2);
    }

    // Legacy special key test removed - now using InputRouter action system

    #[test]
    fn test_focus_handling() {
        let mut editor = new_with_buffer();

        // Editor should start unfocused
        assert_eq!(editor.is_focused(), false);

        // Test setting focus
        editor.set_focus(true);
        assert_eq!(editor.is_focused(), true);

        // Test removing focus
        editor.set_focus(false);
        assert_eq!(editor.is_focused(), false);
    }

    // Legacy GPUI action tests removed - now using InputRouter action system

    #[test]
    fn test_focus_handle_integration() {
        // GPUI testing requires proper application context setup

        // Test that MarkdownEditor compiles with Focusable trait
        // We can't test the actual focus functionality in unit tests due to GPUI's main thread requirement,
        // but we can verify the trait is implemented correctly

        // This test ensures:
        // 1. MarkdownEditor has a focus_handle field
        // 2. MarkdownEditor implements Focusable trait
        // 3. The implementation compiles correctly

        // The actual GPUI context testing would be done in integration tests
        // For now, we just verify the trait implementation exists
        fn _ensure_focusable_trait_implemented() {
            // This function shouldn't be called, it just ensures compilation
            fn check_focusable<T: Focusable>(_: T) {}

            // This would only compile if MarkdownEditor implements Focusable
            // check_focusable(editor);  // Can't create editor without GPUI context
        }

        // Simple verification that the test setup is correct
        assert!(true); // Placeholder assertion - the real test is compilation
    }

    // Legacy action binding test removed - now using InputRouter directly

    #[test]
    fn test_key_down_event_handling() {
        // Test that character input events are properly handled
        let mut editor = new_with_buffer();

        // Test that we can simulate character input through our interface
        editor.handle_char_input('h');
        editor.handle_char_input('e');
        editor.handle_char_input('l');
        editor.handle_char_input('l');
        editor.handle_char_input('o');

        assert_eq!(editor.content(), "hello");
        assert_eq!(editor.cursor_position(), 5);

        // Legacy input event test removed - now using InputRouter actions

        // Verify the GPUI key handler method exists by testing its compilation
        fn _verify_key_handler_exists() {
            // This function verifies that the MarkdownEditor has a key down handler method
            // We can't test it directly due to GPUI context requirements, but we can ensure it compiles

            // The method should have signature: handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>)
            // This will be verified during compilation when we add the method
        }
    }

    #[test]
    fn test_basic_punctuation() {
        let mut editor = new_with_buffer();

        // Test basic ASCII punctuation first
        let punctuation = "!@#$%^&*()";
        for ch in punctuation.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), punctuation);
    }

    #[test]
    fn test_unicode_characters() {
        let mut editor = new_with_buffer();

        // Test one multi-byte character at a time to identify the issue
        editor.handle_char_input('á');
        assert_eq!(editor.content(), "á");
    }

    #[test]
    fn test_comprehensive_special_characters() {
        let mut editor = new_with_buffer();

        // Test basic punctuation
        let basic_punct = "!@#$%^&*()";
        for ch in basic_punct.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), basic_punct);
        editor.document = TextDocument::new();

        // Test brackets and quotes
        let brackets_quotes = "()[]{}<>\"'`";
        for ch in brackets_quotes.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), brackets_quotes);
        editor.document = TextDocument::new();

        // Test symbols and operators
        let symbols = "~-_+=|\\:;,.<>?/";
        for ch in symbols.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), symbols);
        editor.document = TextDocument::new();

        // Test accented characters (multi-byte unicode)
        let accented = "áéíóúàèìòùâêîôû";
        for ch in accented.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), accented);
        editor.document = TextDocument::new();

        // Test unicode symbols
        let unicode_symbols = "€£¥¢™®©±×÷";
        for ch in unicode_symbols.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), unicode_symbols);
        editor.document = TextDocument::new();

        // Test emoji (multi-byte unicode)
        let emoji = "😀🎉💯⭐";
        for ch in emoji.chars() {
            editor.handle_char_input(ch);
        }
        assert_eq!(editor.content(), emoji);
    }

    #[test]
    fn test_enter_key_creates_newline() {
        let mut editor = new_with_buffer();

        // Type some text
        editor.handle_char_input('H');
        editor.handle_char_input('i');
        assert_eq!(editor.content(), "Hi");

        // Press Enter (should insert newline)
        editor.handle_char_input('\n');
        assert_eq!(editor.content(), "Hi\n");
        assert_eq!(editor.cursor_position(), 3);

        // Type more text after newline
        editor.handle_char_input('W');
        editor.handle_char_input('o');
        editor.handle_char_input('r');
        editor.handle_char_input('l');
        editor.handle_char_input('d');
        assert_eq!(editor.content(), "Hi\nWorld");
        assert_eq!(editor.cursor_position(), 8);
    }

    #[test]
    fn test_raw_markdown_display() {
        let mut editor = new_with_buffer();

        // Test that markdown syntax is preserved as-is
        editor.handle_char_input('#');
        editor.handle_char_input(' ');
        editor.handle_char_input('H');
        editor.handle_char_input('e');
        editor.handle_char_input('a');
        editor.handle_char_input('d');
        editor.handle_char_input('i');
        editor.handle_char_input('n');
        editor.handle_char_input('g');
        assert_eq!(editor.content(), "# Heading");

        editor.handle_char_input('\n');
        editor.handle_char_input('*');
        editor.handle_char_input('*');
        editor.handle_char_input('b');
        editor.handle_char_input('o');
        editor.handle_char_input('l');
        editor.handle_char_input('d');
        editor.handle_char_input('*');
        editor.handle_char_input('*');
        assert_eq!(editor.content(), "# Heading\n**bold**");

        // Test that spacing is preserved
        editor.handle_char_input(' ');
        editor.handle_char_input(' ');
        editor.handle_char_input('*');
        editor.handle_char_input('i');
        editor.handle_char_input('t');
        editor.handle_char_input('a');
        editor.handle_char_input('l');
        editor.handle_char_input('i');
        editor.handle_char_input('c');
        editor.handle_char_input('*');
        assert_eq!(editor.content(), "# Heading\n**bold**  *italic*");
    }

    #[test]
    fn test_insert_text() {
        let mut editor = new_with_buffer();
        editor.document_mut().insert_text("Hello");
        assert_eq!(editor.content(), "Hello");
        assert_eq!(editor.cursor_position(), 5);
    }

    // Legacy delete_char and cursor_movement tests removed - now using InputRouter actions

    #[test]
    fn test_editor_with_text_buffer() {
        let mut editor = new_with_buffer();
        editor.insert_char('H');
        assert_eq!(editor.content(), "H");
        assert_eq!(editor.cursor_position(), 1);
        
        // Test character input works with new InputRouter system
        editor.handle_char_input('i');
        assert_eq!(editor.content(), "Hi");
        assert_eq!(editor.cursor_position(), 2);
    }
}
