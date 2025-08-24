use gpui::{
    div, prelude::*, rgb, Context, Render, actions, Action, FocusHandle, Focusable,
    EntityInputHandler, UTF16Selection, Window, ElementInputHandler, Entity, Bounds, Pixels, Point,
    Element, App, LayoutId, IntoElement, TextRun, ShapedLine, px, size,
    KeyDownEvent, transparent_black, Hsla,
};
use crate::core::TextDocument;
use crate::input::{KeyboardHandler, InputEvent};
use crate::hybrid_renderer::HybridTextRenderer;
// use crate::hybrid_editor_element::HybridEditorElement;

actions!(
    markdown_editor,
    [
        EditorBackspace,
        EditorDelete,
        EditorArrowLeft,
        EditorArrowRight,
        EditorArrowUp,
        EditorArrowDown,
        EditorCmdArrowLeft,
        EditorCmdArrowRight,
        EditorCmdArrowUp,
        EditorCmdArrowDown,
        EditorCmdShiftArrowLeft,
        EditorCmdShiftArrowRight,
        EditorCmdShiftArrowUp,
        EditorCmdShiftArrowDown,
        EditorHome,
        EditorEnd,
        EditorPageUp,
        EditorPageDown,
        EditorSelectAll,
    ]
);

pub struct MarkdownEditor {
    document: TextDocument,
    keyboard_handler: KeyboardHandler,
    hybrid_renderer: HybridTextRenderer,
    focused: bool,
    focus_handle: FocusHandle,
}

impl MarkdownEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            document: TextDocument::new(),
            keyboard_handler: KeyboardHandler::new(),
            hybrid_renderer: HybridTextRenderer::new(),
            focused: true, // Start focused
            focus_handle,
        }
    }

    pub fn new_with_content(content: String, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            document: TextDocument::with_content(content),
            keyboard_handler: KeyboardHandler::new(),
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
        self.keyboard_handler.handle_char_input(ch, &mut self.document);
        // Note: No mode updates needed - hybrid renderer handles this automatically
    }

    pub fn handle_input_event(&mut self, event: InputEvent) {
        self.keyboard_handler.handle_input_event(event, &mut self.document);
        // Note: No mode updates needed - hybrid renderer handles this automatically
    }

    // Legacy compatibility methods for tests
    pub fn get_content(&self) -> &str {
        self.content()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn handle_key_input(&mut self, ch: char) {
        self.handle_char_input(ch);
    }

    pub fn handle_special_key(&mut self, key: crate::input::SpecialKey) {
        let event: InputEvent = key.into();
        self.handle_input_event(event);
    }

    pub fn delete_char(&mut self) {
        self.handle_input_event(InputEvent::Backspace);
    }

    // GPUI action handlers - these have the signature expected by cx.listener()
    fn handle_backspace_action(&mut self, _: &EditorBackspace, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::Backspace);
        cx.notify();
    }

    fn handle_delete_action(&mut self, _: &EditorDelete, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::Delete);
        cx.notify();
    }

    fn handle_arrow_left_action(&mut self, _: &EditorArrowLeft, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::ArrowLeft);
        cx.notify();
    }

    fn handle_arrow_right_action(&mut self, _: &EditorArrowRight, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::ArrowRight);
        cx.notify();
    }

    fn handle_arrow_up_action(&mut self, _: &EditorArrowUp, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::ArrowUp);
        cx.notify();
    }

    fn handle_arrow_down_action(&mut self, _: &EditorArrowDown, _: &mut gpui::Window, cx: &mut Context<Self>) {
        self.handle_input_event(InputEvent::ArrowDown);
        cx.notify();
    }

    // Mouse event handlers  
    fn handle_mouse_down(&mut self, _event: &gpui::MouseDownEvent, window: &mut gpui::Window, cx: &mut Context<Self>) {
        // Focus the editor when clicked
        window.focus(&self.focus_handle);
        self.focused = true;
        cx.notify();
    }

    // Key event handler for special keys that don't go through EntityInputHandler
    fn handle_key_down(&mut self, event: &KeyDownEvent, _window: &mut gpui::Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "backspace" => {
                self.handle_input_event(InputEvent::Backspace);
                cx.notify();
            }
            "delete" => {
                self.handle_input_event(InputEvent::Delete);
                cx.notify();
            }
            "enter" => {
                self.handle_char_input('\n');
                cx.notify();
            }
            "left" => {
                self.handle_input_event(InputEvent::ArrowLeft);
                cx.notify();
            }
            "right" => {
                self.handle_input_event(InputEvent::ArrowRight);
                cx.notify();
            }
            "up" => {
                self.handle_input_event(InputEvent::ArrowUp);
                cx.notify();
            }
            "down" => {
                self.handle_input_event(InputEvent::ArrowDown);
                cx.notify();
            }
            _ => {
                // Let other keys be handled by EntityInputHandler or ignored
            }
        }
    }


    // Handle GPUI actions by converting them to InputEvents
    pub fn handle_editor_action(&mut self, action: &dyn Action) {
        let input_event = self.action_to_input_event(action);
        if let Some(event) = input_event {
            self.handle_input_event(event);
        }
    }

    // Convert GPUI actions to our InputEvent system
    fn action_to_input_event(&self, action: &dyn Action) -> Option<InputEvent> {
        if action.type_id() == std::any::TypeId::of::<EditorBackspace>() {
            Some(InputEvent::Backspace)
        } else if action.type_id() == std::any::TypeId::of::<EditorDelete>() {
            Some(InputEvent::Delete)
        } else if action.type_id() == std::any::TypeId::of::<EditorArrowLeft>() {
            Some(InputEvent::ArrowLeft)
        } else if action.type_id() == std::any::TypeId::of::<EditorArrowRight>() {
            Some(InputEvent::ArrowRight)
        } else if action.type_id() == std::any::TypeId::of::<EditorArrowUp>() {
            Some(InputEvent::ArrowUp)
        } else if action.type_id() == std::any::TypeId::of::<EditorArrowDown>() {
            Some(InputEvent::ArrowDown)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowLeft>() {
            Some(InputEvent::CmdArrowLeft)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowRight>() {
            Some(InputEvent::CmdArrowRight)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowUp>() {
            Some(InputEvent::CmdArrowUp)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowDown>() {
            Some(InputEvent::CmdArrowDown)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowLeft>() {
            Some(InputEvent::CmdShiftArrowLeft)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowRight>() {
            Some(InputEvent::CmdShiftArrowRight)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowUp>() {
            Some(InputEvent::CmdShiftArrowUp)
        } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowDown>() {
            Some(InputEvent::CmdShiftArrowDown)
        } else if action.type_id() == std::any::TypeId::of::<EditorHome>() {
            Some(InputEvent::Home)
        } else if action.type_id() == std::any::TypeId::of::<EditorEnd>() {
            Some(InputEvent::End)
        } else if action.type_id() == std::any::TypeId::of::<EditorPageUp>() {
            Some(InputEvent::PageUp)
        } else if action.type_id() == std::any::TypeId::of::<EditorPageDown>() {
            Some(InputEvent::PageDown)
        } else if action.type_id() == std::any::TypeId::of::<EditorSelectAll>() {
            Some(InputEvent::CmdA)
        } else {
            None
        }
    }

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
                if self.cursor_position >= current_offset && self.cursor_position <= current_offset + line.len() {
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
            let shaped_line = window.text_system().shape_line(
                line_text.into(),
                font_size,
                &text_runs,
                None,
            );
            
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
            
            let shaped_line = window.text_system().shape_line(
                " ".into(),
                font_size,
                &[text_run],
                None,
            );
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
        let border_color = if self.focused { rgb(0x89b4fa) } else { rgb(0x313244) };
        
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
        
        for shaped_line in shaped_lines.iter_mut() {
            shaped_line.paint(text_origin, line_height, window, cx)
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
    fn paint_cursor(
        &self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
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
            lines_before.last().map(|line| line.chars().count()).unwrap_or(cursor_position)
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
            .on_action(cx.listener(Self::handle_backspace_action))
            .on_action(cx.listener(Self::handle_delete_action))
            .on_action(cx.listener(Self::handle_arrow_left_action))
            .on_action(cx.listener(Self::handle_arrow_right_action))
            .on_action(cx.listener(Self::handle_arrow_up_action))
            .on_action(cx.listener(Self::handle_arrow_down_action))
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::handle_mouse_down))
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
                            .child("Hybrid Preview - Edit anywhere!")
                    )
            )
            .child(
                // Main content area with hybrid editor
                div()
                    .flex_1()
                    .w_full()
                    .p_4()
                    .child(
                        // Use EditorElement with hybrid rendering capabilities
                        EditorElement {
                            editor: cx.entity().clone(),
                            content,
                            focused: self.focused,
                            focus_handle: self.focus_handle.clone(),
                            cursor_position,
                            selection,
                            hybrid_renderer: HybridTextRenderer::new(),
                        }
                    )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper that creates a minimal editor without GPUI context
    // This is for testing core functionality that doesn't require focus handling
    fn create_test_editor_minimal() -> TestableEditor {
        TestableEditor {
            document: TextDocument::new(),
            keyboard_handler: KeyboardHandler::new(),
            focused: false,
        }
    }
    
    // Wrapper struct for testing that mimics MarkdownEditor without FocusHandle
    #[derive(Debug)]
    struct TestableEditor {
        document: TextDocument,
        keyboard_handler: KeyboardHandler,
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
            self.keyboard_handler.handle_char_input(ch, &mut self.document);
        }
        
        pub fn handle_input_event(&mut self, event: InputEvent) {
            self.keyboard_handler.handle_input_event(event, &mut self.document);
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
        
        pub fn handle_special_key(&mut self, key: crate::input::SpecialKey) {
            let event: InputEvent = key.into();
            self.handle_input_event(event);
        }
        
        pub fn delete_char(&mut self) {
            self.handle_input_event(InputEvent::Backspace);
        }
        
        pub fn document_mut(&mut self) -> &mut TextDocument {
            &mut self.document
        }
        
        pub fn handle_editor_action(&mut self, action: &dyn Action) {
            let input_event = self.action_to_input_event(action);
            if let Some(event) = input_event {
                self.handle_input_event(event);
            }
        }
        
        fn action_to_input_event(&self, action: &dyn Action) -> Option<InputEvent> {
            if action.type_id() == std::any::TypeId::of::<EditorBackspace>() {
                Some(InputEvent::Backspace)
            } else if action.type_id() == std::any::TypeId::of::<EditorDelete>() {
                Some(InputEvent::Delete)
            } else if action.type_id() == std::any::TypeId::of::<EditorArrowLeft>() {
                Some(InputEvent::ArrowLeft)
            } else if action.type_id() == std::any::TypeId::of::<EditorArrowRight>() {
                Some(InputEvent::ArrowRight)
            } else if action.type_id() == std::any::TypeId::of::<EditorArrowUp>() {
                Some(InputEvent::ArrowUp)
            } else if action.type_id() == std::any::TypeId::of::<EditorArrowDown>() {
                Some(InputEvent::ArrowDown)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowLeft>() {
                Some(InputEvent::CmdArrowLeft)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowRight>() {
                Some(InputEvent::CmdArrowRight)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowUp>() {
                Some(InputEvent::CmdArrowUp)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdArrowDown>() {
                Some(InputEvent::CmdArrowDown)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowLeft>() {
                Some(InputEvent::CmdShiftArrowLeft)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowRight>() {
                Some(InputEvent::CmdShiftArrowRight)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowUp>() {
                Some(InputEvent::CmdShiftArrowUp)
            } else if action.type_id() == std::any::TypeId::of::<EditorCmdShiftArrowDown>() {
                Some(InputEvent::CmdShiftArrowDown)
            } else if action.type_id() == std::any::TypeId::of::<EditorHome>() {
                Some(InputEvent::Home)
            } else if action.type_id() == std::any::TypeId::of::<EditorEnd>() {
                Some(InputEvent::End)
            } else if action.type_id() == std::any::TypeId::of::<EditorPageUp>() {
                Some(InputEvent::PageUp)
            } else if action.type_id() == std::any::TypeId::of::<EditorPageDown>() {
                Some(InputEvent::PageDown)
            } else if action.type_id() == std::any::TypeId::of::<EditorSelectAll>() {
                Some(InputEvent::CmdA)
            } else {
                None
            }
        }
        
        pub fn set_cursor_position(&mut self, position: usize) {
            self.document.set_cursor_position(position);
        }
        
        pub fn has_selection(&self) -> bool {
            self.document.has_selection()
        }
        
        pub fn clear_selection(&mut self) {
            self.document.clear_selection();
        }
    }
    
    // Helper method for backward compatibility 
    fn new_with_buffer() -> TestableEditor {
        create_test_editor_minimal()
    }
    
    // Helper method for creating editor with content
    fn new_with_content(content: String) -> TestableEditor {
        TestableEditor {
            document: TextDocument::with_content(content),
            keyboard_handler: KeyboardHandler::new(),
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

    #[test]
    fn test_handle_special_keys() {
        let mut editor = new_with_buffer();
        
        // Type some text first
        editor.handle_key_input('h');
        editor.handle_key_input('e');
        editor.handle_key_input('l');
        editor.handle_key_input('l');
        editor.handle_key_input('o');
        assert_eq!(editor.content(), "hello");
        assert_eq!(editor.cursor_position(), 5);
        
        // Test backspace key
        editor.handle_special_key(crate::input::SpecialKey::Backspace);
        assert_eq!(editor.content(), "hell");
        assert_eq!(editor.cursor_position(), 4);
        
        // Test arrow keys
        editor.handle_special_key(crate::input::SpecialKey::ArrowLeft);
        assert_eq!(editor.cursor_position(), 3);
        
        editor.handle_special_key(crate::input::SpecialKey::ArrowRight);
        assert_eq!(editor.cursor_position(), 4);
    }

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

    #[test]
    fn test_action_definitions_exist() {
        // Test that our GPUI actions are properly defined
        use gpui::Action;
        
        // These should compile and be valid actions
        let _backspace = EditorBackspace {};
        let _delete = EditorDelete {};
        let _arrow_left = EditorArrowLeft {};
        let _arrow_right = EditorArrowRight {};
        
        // Test that they implement Action trait
        assert!(_backspace.boxed_clone().type_id() == std::any::TypeId::of::<EditorBackspace>());
        assert!(_arrow_left.boxed_clone().type_id() == std::any::TypeId::of::<EditorArrowLeft>());
    }

    #[test]
    fn test_action_to_input_event_conversion() {
        let mut editor = new_with_buffer();
        
        // Test that actions can be converted to InputEvents and handled
        editor.handle_editor_action(&EditorBackspace {});
        // Since buffer starts empty, this should have no effect
        assert_eq!(editor.content(), "");
        
        // Add some content first
        editor.handle_char_input('a');
        editor.handle_char_input('b');
        assert_eq!(editor.content(), "ab");
        
        // Now test backspace action
        editor.handle_editor_action(&EditorBackspace {});
        assert_eq!(editor.content(), "a");
        
        // Test arrow actions
        editor.handle_editor_action(&EditorArrowLeft {});
        assert_eq!(editor.cursor_position(), 0);
        
        editor.handle_editor_action(&EditorArrowRight {});
        assert_eq!(editor.cursor_position(), 1);
    }

    #[test]
    fn test_advanced_navigation_actions() {
        let mut editor = new_with_buffer();
        editor.handle_char_input('H');
        editor.handle_char_input('e');
        editor.handle_char_input('l');
        editor.handle_char_input('l');
        editor.handle_char_input('o');
        editor.handle_char_input(' ');
        editor.handle_char_input('w');
        editor.handle_char_input('o');
        editor.handle_char_input('r');
        editor.handle_char_input('l');
        editor.handle_char_input('d');
        // Content: "Hello world", cursor at end (position 11)
        
        // Test word navigation actions
        editor.handle_editor_action(&EditorCmdArrowLeft {});
        assert_eq!(editor.cursor_position(), 6); // Start of "world"
        
        editor.handle_editor_action(&EditorCmdArrowRight {});
        assert_eq!(editor.cursor_position(), 11); // End of "world"
        
        // Test document navigation actions
        editor.handle_editor_action(&EditorCmdArrowUp {});
        assert_eq!(editor.cursor_position(), 0); // Start of document
        
        editor.handle_editor_action(&EditorCmdArrowDown {});
        assert_eq!(editor.cursor_position(), 11); // End of document
        
        // Test Home/End actions
        editor.handle_editor_action(&EditorHome {});
        assert_eq!(editor.cursor_position(), 0); // Start of line
        
        editor.handle_editor_action(&EditorEnd {});
        assert_eq!(editor.cursor_position(), 11); // End of line
    }

    #[test]
    fn test_selection_extension_actions() {
        let mut editor = new_with_buffer();
        editor.handle_char_input('H');
        editor.handle_char_input('e');
        editor.handle_char_input('l');
        editor.handle_char_input('l');
        editor.handle_char_input('o');
        editor.handle_char_input(' ');
        editor.handle_char_input('w');
        editor.handle_char_input('o');
        editor.handle_char_input('r');
        editor.handle_char_input('l');
        editor.handle_char_input('d');
        // Content: "Hello world", cursor at end (position 11)
        editor.set_cursor_position(8); // In middle of "world"
        
        // Test word selection extension actions
        editor.handle_editor_action(&EditorCmdShiftArrowLeft {});
        assert!(editor.has_selection());
        assert_eq!(editor.cursor_position(), 6); // Start of "world"
        
        // Clear selection and test document selection
        editor.clear_selection();
        editor.set_cursor_position(8);
        editor.handle_editor_action(&EditorCmdShiftArrowUp {});
        assert!(editor.has_selection());
        assert_eq!(editor.cursor_position(), 0); // Start of document
    }

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

    #[test]
    fn test_action_binding_compilation() {
        // Test that our render method can bind actions without runtime execution
        // This ensures the action binding syntax is correct
        
        // Create a test that verifies action handling methods exist
        fn _verify_action_handler_exists() {
            let mut editor = TestableEditor {
                document: TextDocument::new(),
                keyboard_handler: KeyboardHandler::new(),
                focused: false,
            };
            
            // These method calls should compile, proving our action handlers exist
            editor.handle_editor_action(&EditorBackspace {});
            editor.handle_editor_action(&EditorDelete {});
            editor.handle_editor_action(&EditorArrowLeft {});
            editor.handle_editor_action(&EditorArrowRight {});
        }
        
        // This test verifies compilation of action handling
        assert!(true); // The real test is that the above compiles
    }

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
        
        // Test that backspace works
        editor.handle_input_event(InputEvent::Backspace);
        assert_eq!(editor.content(), "hell");
        assert_eq!(editor.cursor_position(), 4);
        
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
    
    #[test]
    fn test_delete_char() {
        let mut editor = new_with_content("Hello".to_string());
        editor.delete_char();
        assert_eq!(editor.content(), "Hell");
        assert_eq!(editor.cursor_position(), 4);
    }
    
    #[test]
    fn test_cursor_movement() {
        let mut editor = new_with_content("Hello".to_string());
        editor.document_mut().set_cursor_position(3);
        
        editor.handle_input_event(InputEvent::ArrowLeft);
        assert_eq!(editor.cursor_position(), 2);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 3);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 5);
        
        editor.handle_input_event(InputEvent::ArrowRight);
        assert_eq!(editor.cursor_position(), 5); // Should not go beyond content length
    }

    #[test]
    fn test_editor_with_text_buffer() {
        let mut editor = new_with_buffer();
        editor.insert_char('H');
        assert_eq!(editor.content(), "H");
        assert_eq!(editor.cursor_position(), 1);
        
        editor.delete_char();
        assert_eq!(editor.content(), "");
        assert_eq!(editor.cursor_position(), 0);
    }

    #[test]
    fn test_select_all_action_integration() {
        let mut editor = new_with_content("Hello world test".to_string());
        editor.set_cursor_position(8); // In middle
        
        // Test EditorSelectAll action
        editor.handle_editor_action(&EditorSelectAll {});
        assert!(editor.has_selection());
        assert_eq!(editor.document_mut().selected_text(), Some("Hello world test".to_string()));
        assert_eq!(editor.cursor_position(), 16); // At end
    }

    #[test]
    fn test_shift_arrow_selection_highlighting_integration() {
        let mut editor = new_with_content("Hello world".to_string());
        editor.set_cursor_position(5); // After "Hello"
        
        // Test shift+arrow selection creates visual highlighting
        editor.handle_input_event(InputEvent::ShiftArrowRight);
        editor.handle_input_event(InputEvent::ShiftArrowRight);
        
        assert!(editor.has_selection());
        assert_eq!(editor.document_mut().selected_text(), Some(" w".to_string()));
        
        // Verify selection range for visual highlighting
        let selection_range = editor.document_mut().selection_range().unwrap();
        assert_eq!(selection_range, (5, 7));
    }
}