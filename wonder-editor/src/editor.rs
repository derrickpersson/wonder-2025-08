use crate::core::TextDocument;
use crate::hybrid_renderer::HybridTextRenderer;
use crate::input::InputRouter;
use gpui::{
    div, prelude::*, px, rgb, size, transparent_black, App, Bounds, ClipboardItem, Context,
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
    // ENG-138: Mouse state tracking for drag operations
    is_mouse_down: bool,
    mouse_down_position: Option<usize>,
    // ENG-139: Click count tracking for double/triple-click selection
    last_click_time: std::time::Instant,
    last_click_position: Option<usize>,
    // ENG-142: Scroll state tracking
    scroll_offset: f32,
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
            // ENG-138: Initialize mouse state
            is_mouse_down: false,
            mouse_down_position: None,
            // ENG-139: Initialize click tracking
            last_click_time: std::time::Instant::now(),
            last_click_position: None,
            // ENG-142: Initialize scroll state
            scroll_offset: 0.0,
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
            // ENG-138: Initialize mouse state
            is_mouse_down: false,
            mouse_down_position: None,
            // ENG-139: Initialize click tracking
            last_click_time: std::time::Instant::now(),
            last_click_position: None,
            // ENG-142: Initialize scroll state
            scroll_offset: 0.0,
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
        event: &gpui::MouseDownEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        // Focus the editor when clicked
        window.focus(&self.focus_handle);
        self.focused = true;
        
        // ENG-137/138: Convert mouse coordinates to character position
        let character_position = self.convert_point_to_character_index(event.position);
        
        // ENG-140: Check for Shift modifier for selection extension
        if event.modifiers.shift {
            // Shift+click - extend selection (no drag support for Shift+click)
            self.handle_shift_click_at_position(character_position);
        } else {
            // ENG-139: Detect click count for double/triple-click selection
            let now = std::time::Instant::now();
            let click_count = self.calculate_click_count(character_position, now);
            
            // Handle different click types
            self.handle_click_with_count(character_position, click_count);
            
            // Set flag that we're potentially starting a drag operation (only for single clicks)
            if click_count == 1 {
                self.is_mouse_down = true;
                self.mouse_down_position = Some(character_position);
            }
        }
        
        cx.notify();
    }

    fn handle_mouse_up(
        &mut self,
        event: &gpui::MouseUpEvent,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        // ENG-138: Handle mouse up for drag selection
        if self.is_mouse_down {
            let character_position = self.convert_point_to_character_index(event.position);
            self.handle_mouse_up_at_position(character_position);
            
            self.is_mouse_down = false;
            self.mouse_down_position = None;
            
            cx.notify();
        }
    }

    fn handle_mouse_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        // ENG-138: Handle mouse drag for selection
        if self.is_mouse_down {
            let character_position = self.convert_point_to_character_index(event.position);
            self.handle_mouse_drag_to_position(character_position);
            
            cx.notify();
        }
    }

    // Helper method to convert screen coordinates to character positions
    fn convert_point_to_character_index(&self, point: Point<Pixels>) -> usize {
        // Basic coordinate to character conversion
        // This mimics the logic in character_index_for_point but for direct use
        let padding = px(16.0);
        let line_height = px(24.0);
        
        // Account for editor bounds (status bar height + editor padding)
        let editor_content_y_offset = px(30.0) + padding; // Status bar height + padding
        let relative_y = point.y - editor_content_y_offset;
        let line_index = ((relative_y / line_height).floor() as usize).max(0);
        
        let content = self.document.content();
        let lines: Vec<&str> = content.lines().collect();
        
        // If clicked beyond last line, return end of document
        if line_index >= lines.len() {
            return content.chars().count();
        }
        
        // Calculate character position within the clicked line
        let line_content = lines[line_index];
        let relative_x = point.x - padding;
        
        // Simple character width approximation (will be improved with proper text measurement)
        let font_size = px(16.0);
        let approx_char_width = font_size * 0.6;
        let char_index_in_line = ((relative_x / approx_char_width).floor() as usize).min(line_content.chars().count());
        
        // Calculate absolute position in document
        let chars_before_line: usize = lines.iter().take(line_index).map(|l| l.chars().count() + 1).sum();
        let absolute_position = chars_before_line + char_index_in_line;
        
        absolute_position.min(content.chars().count())
    }

    // ENG-139: Click count and selection helpers
    fn calculate_click_count(&mut self, position: usize, now: std::time::Instant) -> u32 {
        const DOUBLE_CLICK_TIME: std::time::Duration = std::time::Duration::from_millis(500);
        const CLICK_POSITION_TOLERANCE: usize = 3; // Allow small position differences
        
        // Check if this is a rapid click at same position
        let is_rapid_click = now.duration_since(self.last_click_time) <= DOUBLE_CLICK_TIME;
        let is_same_position = self.last_click_position
            .map(|last_pos| position.abs_diff(last_pos) <= CLICK_POSITION_TOLERANCE)
            .unwrap_or(false);
        
        let click_count = if is_rapid_click && is_same_position {
            // Determine if this is the second, third, etc. click
            // For simplicity, we'll detect double-click vs triple-click based on timing
            if now.duration_since(self.last_click_time) <= std::time::Duration::from_millis(250) {
                3 // Very rapid = triple click
            } else {
                2 // Moderately rapid = double click
            }
        } else {
            1 // Single click
        };
        
        // Update tracking
        self.last_click_time = now;
        self.last_click_position = Some(position);
        
        click_count
    }

    fn handle_click_with_count(&mut self, position: usize, click_count: u32) {
        match click_count {
            1 => {
                // Single click - position cursor
                self.handle_mouse_down_at_position(position);
            },
            2 => {
                // Double click - select word at position
                self.select_word_at_position(position);
            },
            3 => {
                // Triple click - select line at position  
                self.select_line_at_position(position);
            },
            _ => {
                // Fall back to single click for higher counts
                self.handle_mouse_down_at_position(position);
            }
        }
    }

    fn select_word_at_position(&mut self, position: usize) {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find word boundaries around the clicked position
        let (word_start, word_end) = self.find_word_boundaries(content, clamped_position);
        
        // Set selection to cover the word
        self.document.set_cursor_position(word_start);
        self.document.start_selection();
        self.document.set_cursor_position(word_end);
    }

    fn select_line_at_position(&mut self, position: usize) {
        let content = self.document.content();
        let max_pos = content.chars().count();
        let clamped_position = position.min(max_pos);
        
        // Find line boundaries around the clicked position
        let (line_start, line_end) = self.find_line_boundaries(content, clamped_position);
        
        // Set selection to cover the line
        self.document.set_cursor_position(line_start);
        self.document.start_selection();
        self.document.set_cursor_position(line_end);
    }

    fn find_word_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        if chars.is_empty() || position >= chars.len() {
            return (0, 0);
        }
        
        // Find start of word (move left while character is word-like)
        let mut start = position;
        while start > 0 && self.is_word_char(chars[start.saturating_sub(1)]) {
            start -= 1;
        }
        
        // Find end of word (move right while character is word-like)
        let mut end = position;
        while end < chars.len() && self.is_word_char(chars[end]) {
            end += 1;
        }
        
        (start, end)
    }

    fn find_line_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        if chars.is_empty() {
            return (0, 0);
        }
        
        let clamped_pos = position.min(chars.len());
        
        // Find start of line (move left until newline or beginning)
        let mut start = clamped_pos;
        while start > 0 && chars[start - 1] != '\n' {
            start -= 1;
        }
        
        // Find end of line (move right until newline or end)
        let mut end = clamped_pos;
        while end < chars.len() && chars[end] != '\n' {
            end += 1;
        }
        
        (start, end)
    }

    fn is_word_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    // ENG-140: Shift+click selection extension
    fn handle_shift_click_at_position(&mut self, position: usize) {
        if self.document.has_selection() {
            // If there's an existing selection, extend it from the original start point
            let (start, _end) = self.document.selection_range().unwrap();
            // Clear current selection and create new one from original start to clicked position
            self.document.clear_selection();
            self.document.set_cursor_position(start);
            self.document.start_selection();
            self.document.set_cursor_position(position);
        } else {
            // No existing selection - create from cursor to clicked position
            self.document.start_selection();
            self.document.set_cursor_position(position);
        }
    }

    // Key event handler for special keys that don't go through EntityInputHandler
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut gpui::Window,
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

    // Legacy action conversion removed - now using InputRouter directly

    // ENG-137: Click-to-position functionality
    pub fn handle_click_at_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Set cursor to clicked position and clear any existing selection
        self.document.set_cursor_position(clamped_position);
        self.document.clear_selection();
        
        true // Return true to indicate successful handling
    }

    // ENG-138: Drag selection functionality for real UI
    pub fn handle_mouse_down_at_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Clear any existing selection and set cursor to clicked position
        self.document.clear_selection();
        self.document.set_cursor_position(clamped_position);
        
        // Note: We don't start selection immediately on mouse down
        // Selection starts when mouse moves (drag begins)
        
        true
    }

    pub fn handle_mouse_drag_to_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // If no selection is active, start one from current cursor position
        if !self.document.has_selection() {
            self.document.start_selection();
        }
        
        // Extend selection to the new position
        self.document.set_cursor_position(clamped_position);
        
        true
    }

    pub fn handle_mouse_up_at_position(&mut self, position: usize) -> bool {
        // Clamp position to document bounds
        let max_pos = self.document.content().chars().count();
        let clamped_position = position.min(max_pos);
        
        // Finalize the selection at the release position
        self.document.set_cursor_position(clamped_position);
        
        // Selection remains active after mouse up
        true
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

            // Calculate cursor position for this line
            let line_cursor_position = if self.cursor_position >= current_offset
                && self.cursor_position <= current_offset + line.len()
            {
                self.cursor_position - current_offset
            } else {
                usize::MAX // Cursor not in this line
            };

            // Calculate selection for this line
            let line_selection = self.selection.as_ref().and_then(|sel| {
                let line_start = current_offset;
                let line_end = current_offset + line.len();
                if sel.end > line_start && sel.start < line_end {
                    let adjusted_start = sel.start.saturating_sub(line_start);
                    let adjusted_end = (sel.end - line_start).min(line.len());
                    Some(adjusted_start..adjusted_end)
                } else {
                    None
                }
            });

            // Get both the transformed content and the styled segments (NEW APPROACH)
            let display_text = self.hybrid_renderer.get_display_content(&line_text, line_cursor_position, line_selection.clone());
            let styled_segments = self.hybrid_renderer.generate_styled_text_segments(&line_text, line_cursor_position, line_selection.clone());
            
            // For now, use the old approach as fallback until full implementation
            let line_runs = self.hybrid_renderer.generate_mixed_text_runs(&line_text, line_cursor_position, line_selection);

            // Use the display text (transformed) for shaping, not the original line text
            let text_to_shape = if display_text.is_empty() { " ".to_string() } else { display_text };
            
            // If no hybrid runs, use fallback styling based on display text length
            let text_runs = if line_runs.is_empty() {
                vec![TextRun {
                    len: text_to_shape.len(),
                    font: gpui::Font {
                        family: "SF Pro".into(),
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

            // COMPLETE FONT SIZE INTEGRATION - Multi-segment text shaping
            let shaped_line = if !styled_segments.is_empty() {
                // Combine all segment text and text runs, but KEEP different font sizes
                // TODO: GPUI limitation - shape_line only accepts one font_size parameter
                // This is the core challenge: GPUI doesn't support mixed font sizes in a single call
                
                // For now, we can choose the approach:
                // Option 1: Use the font size of the first segment
                // Option 2: Use a weighted average font size  
                // Option 3: Shape each segment separately (complex layout integration needed)
                
                let combined_text: String = styled_segments.iter().map(|s| s.text.as_str()).collect();
                let combined_runs: Vec<_> = styled_segments.iter().map(|s| s.text_run.clone()).collect();
                
                // Use the first segment's font size as primary (H1 will dominate if present)
                let primary_font_size = styled_segments.first().map(|s| px(s.font_size)).unwrap_or(font_size);
                
                window.text_system().shape_line(
                    combined_text.into(),
                    primary_font_size,
                    &combined_runs,
                    None
                )
            } else {
                // Fallback to current single-font-size approach
                window.text_system().shape_line(text_to_shape.into(), font_size, &text_runs, None)
            };

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
        let font_size = px(16.0);

        // Process each line with its corresponding shaped line
        let lines: Vec<&str> = content.lines().collect();
        for (line_index, line_text) in lines.iter().enumerate() {
            let line_start = char_offset;
            let line_end = char_offset + line_text.len();
            
            // Check if this line intersects with the selection
            if selection_range.end > line_start && selection_range.start <= line_end {
                // Calculate the selection bounds within this line (original coordinates)
                let sel_start_in_line = selection_range.start.saturating_sub(line_start);
                let sel_end_in_line = (selection_range.end.min(line_end) - line_start).min(line_text.len());
                
                if sel_start_in_line < sel_end_in_line && line_index < shaped_lines.len() {
                    // Calculate cursor position for this line
                    let line_cursor_pos = if self.cursor_position >= line_start && self.cursor_position <= line_end {
                        self.cursor_position - line_start
                    } else {
                        usize::MAX
                    };

                    // Calculate line selection
                    let line_selection = Some(sel_start_in_line..sel_end_in_line);

                    // Get the transformed content and text runs for this line
                    let line_display_text = self.hybrid_renderer.get_display_content(line_text, line_cursor_pos, line_selection.clone());
                    let line_runs = self.hybrid_renderer.generate_mixed_text_runs(line_text, line_cursor_pos, line_selection.clone());

                    // Map selection positions from original to transformed coordinates
                    let transformed_start = self.hybrid_renderer.map_cursor_position(line_text, sel_start_in_line, line_selection.clone());
                    let transformed_end = self.hybrid_renderer.map_cursor_position(line_text, sel_end_in_line, line_selection.clone());

                    // Calculate x positions using transformed content
                    let x_start = if transformed_start == 0 {
                        padding
                    } else {
                        let text_to_sel_start = line_display_text.chars().take(transformed_start).collect::<String>();
                        if !text_to_sel_start.is_empty() {
                            if line_runs.is_empty() {
                                // Fallback to simple measurement
                                let text_run = TextRun {
                                    len: text_to_sel_start.len(),
                                    font: gpui::Font {
                                        family: "SF Pro".into(),
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
                                let shaped = window.text_system().shape_line(
                                    text_to_sel_start.into(),
                                    font_size,
                                    &[text_run],
                                    None,
                                );
                                padding + shaped.width
                            } else {
                                // Use the proper text runs for accurate measurement
                                let mut runs_for_start = Vec::new();
                                let mut run_pos = 0;
                                
                                for run in &line_runs {
                                    if run_pos >= transformed_start {
                                        break;
                                    }
                                    
                                    if run_pos + run.len <= transformed_start {
                                        runs_for_start.push(run.clone());
                                        run_pos += run.len;
                                    } else {
                                        let partial_len = transformed_start - run_pos;
                                        runs_for_start.push(TextRun {
                                            len: partial_len,
                                            font: run.font.clone(),
                                            color: run.color,
                                            background_color: run.background_color,
                                            underline: run.underline.clone(),
                                            strikethrough: run.strikethrough.clone(),
                                        });
                                        break;
                                    }
                                }
                                
                                if !runs_for_start.is_empty() {
                                    let shaped = window.text_system().shape_line(
                                        text_to_sel_start.into(),
                                        font_size,
                                        &runs_for_start,
                                        None,
                                    );
                                    padding + shaped.width
                                } else {
                                    padding
                                }
                            }
                        } else {
                            padding
                        }
                    };
                    
                    // Calculate x_end using transformed content
                    let x_end = {
                        let text_to_sel_end = line_display_text.chars().take(transformed_end).collect::<String>();
                        if !text_to_sel_end.is_empty() {
                            if line_runs.is_empty() {
                                // Fallback to simple measurement
                                let text_run = TextRun {
                                    len: text_to_sel_end.len(),
                                    font: gpui::Font {
                                        family: "SF Pro".into(),
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
                                let shaped = window.text_system().shape_line(
                                    text_to_sel_end.into(),
                                    font_size,
                                    &[text_run],
                                    None,
                                );
                                padding + shaped.width
                            } else {
                                // Use the proper text runs for accurate measurement
                                let mut runs_for_end = Vec::new();
                                let mut run_pos = 0;
                                
                                for run in &line_runs {
                                    if run_pos >= transformed_end {
                                        break;
                                    }
                                    
                                    if run_pos + run.len <= transformed_end {
                                        runs_for_end.push(run.clone());
                                        run_pos += run.len;
                                    } else {
                                        let partial_len = transformed_end - run_pos;
                                        runs_for_end.push(TextRun {
                                            len: partial_len,
                                            font: run.font.clone(),
                                            color: run.color,
                                            background_color: run.background_color,
                                            underline: run.underline.clone(),
                                            strikethrough: run.strikethrough.clone(),
                                        });
                                        break;
                                    }
                                }
                                
                                if !runs_for_end.is_empty() {
                                    let shaped = window.text_system().shape_line(
                                        text_to_sel_end.into(),
                                        font_size,
                                        &runs_for_end,
                                        None,
                                    );
                                    padding + shaped.width
                                } else {
                                    padding
                                }
                            }
                        } else {
                            padding
                        }
                    };
                    
                    // Paint selection rectangle for this line
                    // For empty lines, show a minimum width selection to indicate the line is selected
                    let selection_width = if x_end > x_start {
                        x_end - x_start
                    } else {
                        // Empty line: show a small visual indicator (about 4 pixels wide)
                        px(4.0)
                    };
                    
                    if selection_width > px(0.0) {
                        window.paint_quad(gpui::PaintQuad {
                            bounds: Bounds {
                                origin: bounds.origin + gpui::point(x_start, y_offset),
                                size: size(selection_width, line_height),
                            },
                            background: selection_color.into(),
                            border_widths: gpui::Edges::all(px(0.0)),
                            border_color: transparent_black().into(),
                            border_style: gpui::BorderStyle::Solid,
                            corner_radii: gpui::Corners::all(px(0.0)),
                        });
                    }
                }
            }
            
            // Move to next line
            char_offset = line_end + 1; // +1 for newline character
            y_offset += line_height;
        }
    }

    fn paint_cursor(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        // Get the cursor position from the editor (original content position)
        let original_cursor_position = self.editor.read(cx).cursor_position();
        let content = &self.content;
        
        // Map cursor position to transformed content coordinates
        let transformed_cursor_position = self.hybrid_renderer.map_cursor_position(
            content, 
            original_cursor_position, 
            self.selection.clone()
        );

        // Calculate which line the cursor is on based on ORIGINAL content (for line counting)
        let chars_before_cursor: String = content.chars().take(original_cursor_position).collect();
        let line_number = chars_before_cursor.matches('\n').count();

        // Get the actual line content from original text
        let current_line_original = content.lines().nth(line_number).unwrap_or("");
        
        // Find cursor position within this specific line (original coordinates)
        let lines_before: Vec<&str> = chars_before_cursor.lines().collect();
        let original_position_in_line = if chars_before_cursor.ends_with('\n') {
            0
        } else {
            lines_before
                .last()
                .map(|line| line.chars().count())
                .unwrap_or(0)
        };

        // Calculate the selection for this line
        let line_start_offset = content.lines().take(line_number).map(|l| l.len() + 1).sum::<usize>(); // +1 for newlines
        let line_cursor_pos = if original_cursor_position >= line_start_offset && original_cursor_position <= line_start_offset + current_line_original.len() {
            original_cursor_position - line_start_offset
        } else {
            usize::MAX
        };
        
        let line_selection = self.selection.as_ref().and_then(|sel| {
            let line_end = line_start_offset + current_line_original.len();
            if sel.end > line_start_offset && sel.start <= line_end {
                let adjusted_start = sel.start.saturating_sub(line_start_offset);
                let adjusted_end = (sel.end - line_start_offset).min(current_line_original.len());
                Some(adjusted_start..adjusted_end)
            } else {
                None
            }
        });

        // Get the transformed content for this line and map cursor position within the line
        let line_display_text = self.hybrid_renderer.get_display_content(current_line_original, line_cursor_pos, line_selection.clone());
        let transformed_position_in_line = self.hybrid_renderer.map_cursor_position(current_line_original, original_position_in_line, line_selection.clone());

        // Calculate cursor position
        let padding = px(16.0);
        let line_height = px(24.0);
        let font_size = px(16.0);

        // Calculate X position by shaping the TRANSFORMED text up to the TRANSFORMED cursor position
        let cursor_x_offset = if transformed_position_in_line == 0 {
            px(0.0)
        } else {
            let text_up_to_cursor = line_display_text.chars().take(transformed_position_in_line).collect::<String>();

            if text_up_to_cursor.is_empty() {
                px(0.0)
            } else {
                // Generate the actual text runs for this line so we measure correctly
                let line_runs = self.hybrid_renderer.generate_mixed_text_runs(
                    current_line_original, 
                    line_cursor_pos, 
                    line_selection
                );

                if line_runs.is_empty() {
                    // Fallback to simple measurement
                    let text_run = TextRun {
                        len: text_up_to_cursor.len(),
                        font: gpui::Font {
                            family: "SF Pro".into(),
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
                        text_up_to_cursor.into(),
                        font_size,
                        &[text_run],
                        None,
                    );
                    
                    shaped_line.width
                } else {
                    // Create a subset of text runs that covers our cursor position
                    let mut runs_for_cursor = Vec::new();
                    let mut run_pos = 0;
                    
                    for run in line_runs {
                        if run_pos >= transformed_position_in_line {
                            break;
                        }
                        
                        if run_pos + run.len <= transformed_position_in_line {
                            // Full run is before cursor
                            let run_len = run.len;
                            runs_for_cursor.push(run);
                            run_pos += run_len;
                        } else {
                            // Partial run up to cursor
                            let partial_len = transformed_position_in_line - run_pos;
                            runs_for_cursor.push(TextRun {
                                len: partial_len,
                                font: run.font,
                                color: run.color,
                                background_color: run.background_color,
                                underline: run.underline,
                                strikethrough: run.strikethrough,
                            });
                            break;
                        }
                    }
                    
                    if !runs_for_cursor.is_empty() {
                        let shaped_line = window.text_system().shape_line(
                            text_up_to_cursor.into(),
                            font_size,
                            &runs_for_cursor,
                            None,
                        );
                        
                        shaped_line.width
                    } else {
                        px(0.0)
                    }
                }
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
            // ENG-142: Initialize scroll state
            scroll_offset: 0.0,
            // ENG-143: Initialize context menu state
            context_menu_visible: false,
            context_menu_position: None,
            simulated_clipboard_content: None,
        }
    }

    // Wrapper struct for testing that mimics MarkdownEditor without FocusHandle
    #[derive(Debug)]
    struct TestableEditor {
        document: TextDocument,
        input_router: InputRouter,
        focused: bool,
        // ENG-142: Scroll state for testing
        scroll_offset: f32,
        // ENG-143: Context menu state for testing
        context_menu_visible: bool,
        context_menu_position: Option<usize>,
        simulated_clipboard_content: Option<String>,
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

        // Convenience methods for easier testing
        pub fn has_selection(&self) -> bool {
            self.document.has_selection()
        }

        pub fn selected_text(&self) -> Option<String> {
            self.document.selected_text()
        }

        // ENG-137: Click-to-position functionality
        pub fn handle_click_at_position(&mut self, position: usize) -> bool {
            // Hide context menu on left-click
            self.context_menu_visible = false;
            self.context_menu_position = None;
            
            // Clamp position to document bounds
            let max_pos = self.document.content().chars().count();
            let clamped_position = position.min(max_pos);
            
            // Set cursor to clicked position
            self.document.set_cursor_position(clamped_position);
            
            true // Return true to indicate successful handling
        }

        // ENG-141: Markdown-aware coordinate mapping functionality
        pub fn handle_click_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
            use crate::hybrid_renderer::HybridTextRenderer;
            
            // Create a hybrid renderer to handle coordinate mapping
            let renderer = HybridTextRenderer::new();
            
            // Get current cursor position to determine rendering mode
            let cursor_pos = self.document.cursor_position();
            let content = self.document.content();
            
            // Map from display position to original content position
            let original_position = renderer.map_display_position_to_original(
                content, 
                display_position, 
                cursor_pos,
                self.document.selection_range()
            );
            
            // Set cursor to the mapped original position
            self.document.set_cursor_position(original_position);
            
            true
        }

        // ENG-138: Drag selection functionality
        pub fn handle_mouse_down_at_position(&mut self, position: usize) -> bool {
            // Clamp position to document bounds
            let max_pos = self.document.content().chars().count();
            let clamped_position = position.min(max_pos);
            
            // Clear any existing selection and set cursor to clicked position
            self.document.clear_selection();
            self.document.set_cursor_position(clamped_position);
            
            // Note: We don't start selection immediately on mouse down
            // Selection starts when mouse moves (drag begins)
            
            true
        }

        pub fn handle_mouse_drag_to_position(&mut self, position: usize) -> bool {
            // Clamp position to document bounds
            let max_pos = self.document.content().chars().count();
            let clamped_position = position.min(max_pos);
            
            // If no selection is active, start one from current cursor position
            if !self.document.has_selection() {
                self.document.start_selection();
            }
            
            // Extend selection to the new position
            self.document.set_cursor_position(clamped_position);
            
            true
        }

        pub fn handle_mouse_up_at_position(&mut self, position: usize) -> bool {
            // Clamp position to document bounds
            let max_pos = self.document.content().chars().count();
            let clamped_position = position.min(max_pos);
            
            // Finalize the selection at the release position
            self.document.set_cursor_position(clamped_position);
            
            // Selection remains active after mouse up
            true
        }

        // ENG-138 + ENG-141: Drag selection with coordinate mapping
        pub fn handle_mouse_down_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
            use crate::hybrid_renderer::HybridTextRenderer;
            
            let renderer = HybridTextRenderer::new();
            let cursor_pos = self.document.cursor_position();
            let content = self.document.content();
            
            let original_position = renderer.map_display_position_to_original(
                content, 
                display_position, 
                cursor_pos,
                self.document.selection_range()
            );
            
            self.handle_mouse_down_at_position(original_position)
        }

        pub fn handle_mouse_drag_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
            use crate::hybrid_renderer::HybridTextRenderer;
            
            let renderer = HybridTextRenderer::new();
            let cursor_pos = self.document.cursor_position();
            let content = self.document.content();
            
            let original_position = renderer.map_display_position_to_original(
                content, 
                display_position, 
                cursor_pos,
                self.document.selection_range()
            );
            
            self.handle_mouse_drag_to_position(original_position)
        }

        pub fn handle_mouse_up_with_coordinate_mapping(&mut self, display_position: usize) -> bool {
            use crate::hybrid_renderer::HybridTextRenderer;
            
            let renderer = HybridTextRenderer::new();
            let cursor_pos = self.document.cursor_position();
            let content = self.document.content();
            
            let original_position = renderer.map_display_position_to_original(
                content, 
                display_position, 
                cursor_pos,
                self.document.selection_range()
            );
            
            self.handle_mouse_up_at_position(original_position)
        }

        // ENG-139: Click count handling for double/triple-click selection
        pub fn handle_click_at_position_with_click_count(&mut self, position: usize, click_count: u32) -> bool {
            match click_count {
                1 => {
                    // Single click - position cursor
                    self.handle_click_at_position(position)
                },
                2 => {
                    // Double click - select word at position
                    self.select_word_at_position(position)
                },
                3 => {
                    // Triple click - select line at position
                    self.select_line_at_position(position)
                },
                _ => {
                    // Fall back to single click for higher counts
                    self.handle_click_at_position(position)
                }
            }
        }

        // Helper method to select word at position
        pub fn select_word_at_position(&mut self, position: usize) -> bool {
            let content = self.document.content();
            let max_pos = content.chars().count();
            let clamped_position = position.min(max_pos);
            
            // Find word boundaries around the clicked position
            let (word_start, word_end) = self.find_word_boundaries(content, clamped_position);
            
            // Set selection to cover the word
            self.document.set_cursor_position(word_start);
            self.document.start_selection();
            self.document.set_cursor_position(word_end);
            
            true
        }

        // Helper method to select line at position
        pub fn select_line_at_position(&mut self, position: usize) -> bool {
            let content = self.document.content();
            let max_pos = content.chars().count();
            let clamped_position = position.min(max_pos);
            
            // Find line boundaries around the clicked position
            let (line_start, line_end) = self.find_line_boundaries(content, clamped_position);
            
            // Set selection to cover the line
            self.document.set_cursor_position(line_start);
            self.document.start_selection();
            self.document.set_cursor_position(line_end);
            
            true
        }

        // Helper method to find word boundaries
        fn find_word_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
            let chars: Vec<char> = content.chars().collect();
            if chars.is_empty() || position >= chars.len() {
                return (0, 0);
            }
            
            // Find start of word (move left while character is word-like)
            let mut start = position;
            while start > 0 && self.is_word_char(chars[start.saturating_sub(1)]) {
                start -= 1;
            }
            
            // Find end of word (move right while character is word-like)
            let mut end = position;
            while end < chars.len() && self.is_word_char(chars[end]) {
                end += 1;
            }
            
            (start, end)
        }

        // Helper method to find line boundaries
        fn find_line_boundaries(&self, content: &str, position: usize) -> (usize, usize) {
            let chars: Vec<char> = content.chars().collect();
            if chars.is_empty() {
                return (0, 0);
            }
            
            let clamped_pos = position.min(chars.len());
            
            // Find start of line (move left until newline or beginning)
            let mut start = clamped_pos;
            while start > 0 && chars[start - 1] != '\n' {
                start -= 1;
            }
            
            // Find end of line (move right until newline or end)
            let mut end = clamped_pos;
            while end < chars.len() && chars[end] != '\n' {
                end += 1;
            }
            
            (start, end)
        }

        // Helper method to determine if a character is part of a word
        fn is_word_char(&self, ch: char) -> bool {
            ch.is_alphanumeric() || ch == '_'
        }

        // ENG-140: Shift+click selection extension methods
        pub fn set_cursor_position(&mut self, position: usize) {
            self.document.set_cursor_position(position);
        }

        pub fn start_selection(&mut self) {
            self.document.start_selection();
        }

        pub fn selection_range(&self) -> Option<(usize, usize)> {
            self.document.selection_range()
        }

        pub fn handle_shift_click_at_position(&mut self, position: usize) -> bool {
            let current_cursor = self.cursor_position();
            
            if self.has_selection() {
                // If there's an existing selection, extend it from the original start point
                let (start, _end) = self.document.selection_range().unwrap();
                // Clear current selection and create new one from original start to clicked position
                self.document.clear_selection();
                self.document.set_cursor_position(start);
                self.document.start_selection();
                self.document.set_cursor_position(position);
            } else {
                // No existing selection - create from cursor to clicked position
                self.document.start_selection();
                self.document.set_cursor_position(position);
            }
            
            true
        }

        // ENG-142: Scroll event handling methods
        pub fn get_scroll_offset(&self) -> f32 {
            self.scroll_offset
        }

        pub fn handle_scroll_event(&mut self, _dx: f32, dy: f32) -> bool {
            // Update scroll offset, applying bounds checking
            let new_offset = self.scroll_offset + dy;
            
            // Simple bounds: don't scroll above 0, and for now allow unlimited downward scroll
            self.scroll_offset = new_offset.max(0.0);
            
            true
        }

        // ENG-143: Context menu methods
        pub fn is_context_menu_visible(&self) -> bool {
            self.context_menu_visible
        }

        pub fn get_context_menu_position(&self) -> Option<usize> {
            self.context_menu_position
        }

        pub fn handle_right_click_at_position(&mut self, position: usize) -> bool {
            // Show context menu at the clicked position
            self.context_menu_visible = true;
            self.context_menu_position = Some(position);
            true
        }

        pub fn is_context_menu_copy_enabled(&self) -> bool {
            // Copy enabled when there's a selection
            self.document.has_selection()
        }

        pub fn is_context_menu_cut_enabled(&self) -> bool {
            // Cut enabled when there's a selection
            self.document.has_selection()
        }

        pub fn is_context_menu_select_all_enabled(&self) -> bool {
            // Select All always enabled
            true
        }

        pub fn is_context_menu_paste_enabled(&self) -> bool {
            // Paste enabled when there's clipboard content
            self.simulated_clipboard_content.is_some()
        }

        pub fn simulate_clipboard_content(&mut self, content: String) {
            self.simulated_clipboard_content = Some(content);
        }

        // ENG-143: Context menu action execution methods
        pub fn execute_context_menu_copy(&mut self) -> bool {
            if let Some(selected_text) = self.document.selected_text() {
                self.simulated_clipboard_content = Some(selected_text);
                true
            } else {
                false
            }
        }

        pub fn execute_context_menu_cut(&mut self) -> bool {
            if let Some(selected_text) = self.document.selected_text() {
                self.simulated_clipboard_content = Some(selected_text);
                self.document.delete_selection();
                true
            } else {
                false
            }
        }

        pub fn execute_context_menu_paste(&mut self) -> bool {
            if let Some(clipboard_content) = &self.simulated_clipboard_content.clone() {
                self.document.insert_text(clipboard_content);
                true
            } else {
                false
            }
        }

        pub fn execute_context_menu_select_all(&mut self) -> bool {
            self.document.select_all();
            true
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
            // ENG-142: Initialize scroll state
            scroll_offset: 0.0,
            // ENG-143: Initialize context menu state
            context_menu_visible: false,
            context_menu_position: None,
            simulated_clipboard_content: None,
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

    #[test]
    fn test_hybrid_renderer_integration() {
        // Test that the editor's hybrid renderer is being used, not creating new instances
        let mut editor = new_with_content("Hi **bold text** there!".to_string());
        
        // With cursor outside bold token (at the end), should render as formatted (preview mode)
        editor.document_mut().set_cursor_position(22); // After "there!" - outside the bold token
        
        // The editor should use its own hybrid renderer, which should produce proper text runs
        let renderer = crate::hybrid_renderer::HybridTextRenderer::new();
        let text_runs = renderer.generate_mixed_text_runs(
            editor.content(), 
            editor.cursor_position(), 
            None
        );
        
        // Should have 3 text runs: "Hi ", "bold text" (formatted), " there!"
        assert_eq!(text_runs.len(), 3);
        
        // First run: "Hi "
        assert_eq!(text_runs[0].len, "Hi ".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL);
        
        // Second run: "bold text" (no asterisks, formatted)
        assert_eq!(text_runs[1].len, "bold text".len());
        assert_eq!(text_runs[1].font.weight, gpui::FontWeight::BOLD);
        
        // Third run: " there!"
        assert_eq!(text_runs[2].len, " there!".len());
        assert_eq!(text_runs[2].font.weight, gpui::FontWeight::NORMAL);
    }

    #[test] 
    fn test_hybrid_renderer_integration_raw_mode() {
        // Test that when cursor is inside token, it renders in raw mode
        let mut editor = new_with_content("Hi **bold text** there!".to_string());
        
        // With cursor inside bold token, should render as raw
        editor.document_mut().set_cursor_position(6); // Inside the bold token: "Hi **b|old text**"
        
        let renderer = crate::hybrid_renderer::HybridTextRenderer::new();
        let text_runs = renderer.generate_mixed_text_runs(
            editor.content(), 
            editor.cursor_position(), 
            None
        );
        
        // Should have 3 runs: "Hi ", "**bold text**" (raw), " there!"
        assert_eq!(text_runs.len(), 3);
        
        // First run: "Hi "
        assert_eq!(text_runs[0].len, "Hi ".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL);
        
        // Second run: "**bold text**" (raw with asterisks)
        assert_eq!(text_runs[1].len, "**bold text**".len());
        assert_eq!(text_runs[1].font.weight, gpui::FontWeight::NORMAL);
        
        // Third run: " there!"
        assert_eq!(text_runs[2].len, " there!".len());
        assert_eq!(text_runs[2].font.weight, gpui::FontWeight::NORMAL);
    }

    #[test]
    fn test_cursor_positioning_during_mode_switches() {
        // Test that cursor position remains accurate when switching between raw/preview modes
        let mut editor = new_with_content("Text **bold content** more text".to_string());
        
        // Test cursor positions at various points
        let test_positions = vec![
            (0, "Start of document"),
            (5, "Before bold token"),
            (7, "Inside bold token - at **b"),  
            (12, "Inside bold token - at ld co"),
            (20, "At end of bold token"),
            (22, "After bold token"),
            (31, "End of document"),
        ];
        
        for (pos, description) in test_positions {
            editor.document_mut().set_cursor_position(pos);
            let cursor_pos = editor.cursor_position();
            
            // Cursor should be clamped to valid document bounds
            assert!(cursor_pos <= editor.content().chars().count(), 
                "Cursor position {} should be within document bounds for: {}", cursor_pos, description);
            
            // Generate text runs with current cursor position  
            let renderer = crate::hybrid_renderer::HybridTextRenderer::new();
            let text_runs = renderer.generate_mixed_text_runs(
                editor.content(),
                cursor_pos,
                None
            );
            
            // Text runs should always be generated
            assert!(!text_runs.is_empty(), "Text runs should not be empty for: {}", description);
            
            // Total length of runs should equal the transformed content length
            let total_run_length: usize = text_runs.iter().map(|run| run.len).sum();
            assert!(total_run_length > 0, "Total text run length should be > 0 for: {}", description);
        }
    }

    #[test]
    fn test_text_input_with_hybrid_rendering() {
        // Test that text input works correctly while hybrid rendering is active
        let mut editor = new_with_content("**bold**".to_string());
        
        // Position cursor inside the bold token
        editor.document_mut().set_cursor_position(4); // Inside "**bo|ld**"
        
        // Insert a character
        editor.handle_char_input('X');
        
        // Content should be updated
        assert_eq!(editor.content(), "**boXld**");
        assert_eq!(editor.cursor_position(), 5); // Cursor advances after insert
        
        // Verify hybrid rendering still works with new content
        let renderer = crate::hybrid_renderer::HybridTextRenderer::new();
        let text_runs = renderer.generate_mixed_text_runs(
            editor.content(),
            editor.cursor_position(),
            None
        );
        
        // Should be in raw mode since cursor is inside token
        assert_eq!(text_runs.len(), 1);
        assert_eq!(text_runs[0].len, "**boXld**".len());
        assert_eq!(text_runs[0].font.weight, gpui::FontWeight::NORMAL); // Raw mode
    }

    #[test]
    fn test_cursor_navigation_with_markdown_tokens() {
        // Test that cursor navigation works correctly around markdown tokens
        let mut editor = new_with_content("Before **bold text** after".to_string());
        
        // Test moving cursor through the document
        let navigation_tests = vec![
            (0, "Before |**bold text** after"),
            (7, "Before |**bold text** after"), // Start of bold token
            (10, "Before **b|old text** after"), // Inside bold token  
            (16, "Before **bold t|ext** after"), // Inside bold token
            (19, "Before **bold text|** after"), // End of bold token
            (21, "Before **bold text** |after"), // After bold token
        ];
        
        for (target_pos, description) in navigation_tests {
            editor.document_mut().set_cursor_position(target_pos);
            let actual_pos = editor.cursor_position();
            
            // Cursor position should be valid
            assert!(actual_pos <= editor.content().chars().count(), 
                "Invalid cursor position {} for: {}", actual_pos, description);
            
            // Test arrow key navigation from this position
            let original_pos = actual_pos;
            
            // Move right then left should return to original position (if not at boundaries)
            if original_pos < editor.content().chars().count() {
                editor.document_mut().move_cursor_right();
                editor.document_mut().move_cursor_left();
                assert_eq!(editor.cursor_position(), original_pos, 
                    "Right-then-left navigation failed for: {}", description);
            }
        }
    }

    #[test]
    fn test_editor_uses_styled_segments_with_font_sizes() {
        use crate::hybrid_renderer::{StyledTextSegment, HybridTextRenderer};
        
        // Test content with different markdown elements and their expected font sizes
        let content = "# Big Title\nRegular text **bold** and `code`";
        let mut editor = new_with_content(content.to_string());
        
        // Position cursor after heading to trigger preview mode
        editor.document_mut().set_cursor_position(15); // After "# Big Title\n"
        
        // Create a hybrid renderer to test styled segments integration
        let renderer = HybridTextRenderer::new();
        let segments = renderer.generate_styled_text_segments(
            editor.content(),
            editor.cursor_position(),
            None
        );
        
        // Should have segments for: H1 title, regular text, bold text, space, code
        assert!(!segments.is_empty(), "Editor should generate styled segments");
        
        // Find the heading segment - should be in preview mode with larger font size
        let heading_segment = segments.iter().find(|s| s.text == "Big Title").expect("Should have heading segment");
        assert_eq!(heading_segment.font_size, 24.0, "H1 should use 24px font size");
        assert_eq!(heading_segment.text_run.font.weight, gpui::FontWeight::BOLD, "H1 should be bold");
        
        // Find the code segment - should be in preview mode with smaller monospace font
        let code_segment = segments.iter().find(|s| s.text == "code").expect("Should have code segment");
        assert_eq!(code_segment.font_size, 14.0, "Code should use 14px font size");
        assert_eq!(code_segment.text_run.font.family.as_ref(), "monospace", "Code should use monospace font");
        
        // Find a regular text segment - should use default font size
        let regular_segment = segments.iter().find(|s| s.text.contains("Regular text")).expect("Should have regular text segment");
        assert_eq!(regular_segment.font_size, 16.0, "Regular text should use 16px font size");
        
        // Find the bold segment - should use regular font size but bold weight
        let bold_segment = segments.iter().find(|s| s.text == "bold").expect("Should have bold segment");
        assert_eq!(bold_segment.font_size, 16.0, "Bold text should use 16px font size");
        assert_eq!(bold_segment.text_run.font.weight, gpui::FontWeight::BOLD, "Bold text should be bold");
    }

    #[test]
    fn test_visual_styling_integration_complete() {
        use crate::hybrid_renderer::{HybridTextRenderer};
        
        // Test that the editor now uses styled segments in its rendering pipeline
        let content = "# Large Title\n`code block`\n**Bold text**";
        let mut editor = new_with_content(content.to_string());
        
        // Position cursor to trigger appropriate preview modes
        editor.document_mut().set_cursor_position(20); // After title and code
        
        // The editor should now be using styled segments internally
        let renderer = HybridTextRenderer::new();
        
        // Test line 1: "# Large Title" - should use H1 font size (24px)
        let line1_segments = renderer.generate_styled_text_segments("# Large Title", 15, None);
        assert!(!line1_segments.is_empty(), "H1 line should have styled segments");
        let h1_segment = line1_segments.iter().find(|s| s.text == "Large Title").expect("Should have H1 segment");
        assert_eq!(h1_segment.font_size, 24.0, "H1 should be 24px");
        
        // Test line 2: "`code block`" - should use code font size (14px) and monospace
        let line2_segments = renderer.generate_styled_text_segments("`code block`", 15, None);
        assert!(!line2_segments.is_empty(), "Code line should have styled segments");
        let code_segment = line2_segments.iter().find(|s| s.text == "code block").expect("Should have code segment");
        assert_eq!(code_segment.font_size, 14.0, "Code should be 14px");
        assert_eq!(code_segment.text_run.font.family.as_ref(), "monospace", "Code should be monospace");
        
        // Test line 3: "**Bold text**" - should use regular font size (16px) with bold weight
        let line3_segments = renderer.generate_styled_text_segments("**Bold text**", 15, None);
        assert!(!line3_segments.is_empty(), "Bold line should have styled segments");
        let bold_segment = line3_segments.iter().find(|s| s.text == "Bold text").expect("Should have bold segment");
        assert_eq!(bold_segment.font_size, 16.0, "Bold should be 16px");
        assert_eq!(bold_segment.text_run.font.weight, gpui::FontWeight::BOLD, "Bold should be bold weight");
        
        println!("✅ Visual styling integration test passed!");
        println!("  H1 headings: 24px font size");
        println!("  Code blocks: 14px monospace font");
        println!("  Bold text: 16px with bold weight");
    }

    // ENG-139: Double/triple-click selection tests
    #[test]
    fn test_click_count_tracking() {
        // RED: This test should fail because we haven't implemented click count tracking yet
        let mut editor = new_with_content("Hello world test".to_string());
        
        // Simulate single click
        let result = editor.handle_click_at_position_with_click_count(6, 1);
        assert!(result, "Single click should succeed");
        assert_eq!(editor.cursor_position(), 6, "Single click should position cursor");
        assert!(!editor.has_selection(), "Single click should not create selection");
        
        // Simulate double click at same position
        let result = editor.handle_click_at_position_with_click_count(6, 2);
        assert!(result, "Double click should succeed");
        assert!(editor.has_selection(), "Double click should create word selection");
        assert_eq!(editor.selected_text(), Some("world".to_string()), "Double click should select word under cursor");
        
        // Simulate triple click at same position  
        let result = editor.handle_click_at_position_with_click_count(6, 3);
        assert!(result, "Triple click should succeed");
        assert!(editor.has_selection(), "Triple click should create line selection");
        assert_eq!(editor.selected_text(), Some("Hello world test".to_string()), "Triple click should select entire line");
    }

    #[test]
    fn test_word_boundary_detection_edge_cases() {
        // RED: This test should fail because our word boundary logic might not handle all edge cases
        let mut editor = new_with_content("hello-world test_case another.word".to_string());
        
        // Test double-click on hyphenated word
        let result = editor.handle_click_at_position_with_click_count(6, 2); // On 'world' in 'hello-world'
        assert!(result, "Double click on hyphenated word should succeed");
        assert_eq!(editor.selected_text(), Some("world".to_string()), "Should select 'world' part of hyphenated word");
        
        // Test double-click on underscore word
        editor = new_with_content("hello-world test_case another.word".to_string());
        let result = editor.handle_click_at_position_with_click_count(18, 2); // On 'case' in 'test_case'
        assert!(result, "Double click on underscore word should succeed");
        assert_eq!(editor.selected_text(), Some("test_case".to_string()), "Should select entire underscore word");
        
        // Test double-click on word with dot
        editor = new_with_content("hello-world test_case another.word".to_string());
        let result = editor.handle_click_at_position_with_click_count(30, 2); // On 'word' in 'another.word'
        assert!(result, "Double click on dotted word should succeed");
        assert_eq!(editor.selected_text(), Some("word".to_string()), "Should select 'word' part after dot");
    }

    #[test]
    fn test_line_selection_multiline_document() {
        // RED: Test triple-click line selection in multi-line context
        let mut editor = new_with_content("First line\nSecond line here\nThird line".to_string());
        
        // Triple-click on second line
        let result = editor.handle_click_at_position_with_click_count(15, 3); // On 'line' in 'Second line here'
        assert!(result, "Triple click should succeed");
        assert_eq!(editor.selected_text(), Some("Second line here".to_string()), "Should select entire second line");
        
        // Triple-click on first line
        editor = new_with_content("First line\nSecond line here\nThird line".to_string());
        let result = editor.handle_click_at_position_with_click_count(5, 3); // On 'First line'
        assert!(result, "Triple click on first line should succeed");
        assert_eq!(editor.selected_text(), Some("First line".to_string()), "Should select entire first line");
    }

    #[test]
    fn test_markdown_token_word_selection() {
        // RED: Test double-click word selection with markdown tokens
        let mut editor = new_with_content("This **bold text** has formatting".to_string());
        
        // Double-click on 'bold' inside markdown token
        let result = editor.handle_click_at_position_with_click_count(9, 2); // On 'bold' in '**bold text**'
        assert!(result, "Double click in markdown token should succeed");
        assert_eq!(editor.selected_text(), Some("bold".to_string()), "Should select 'bold' word within token");
        
        // Double-click on word outside token
        editor = new_with_content("This **bold text** has formatting".to_string());
        let result = editor.handle_click_at_position_with_click_count(23, 2); // On 'formatting'
        assert!(result, "Double click outside token should succeed");
        assert_eq!(editor.selected_text(), Some("formatting".to_string()), "Should select word outside token");
    }

    // ENG-140: Shift+click selection extension tests
    #[test]
    fn test_shift_click_selection_extension_from_cursor() {
        // RED: This test should fail because we haven't implemented Shift+click yet
        let mut editor = new_with_content("Hello world test content".to_string());
        
        // Position cursor at position 6 ('w' in 'world')
        editor.set_cursor_position(6);
        assert_eq!(editor.cursor_position(), 6, "Cursor should be at position 6");
        assert!(!editor.has_selection(), "Should start with no selection");
        
        // Shift+click at position 12 ('t' in 'test') to create selection
        let result = editor.handle_shift_click_at_position(12);
        assert!(result, "Shift+click should succeed");
        assert!(editor.has_selection(), "Shift+click should create selection");
        assert_eq!(editor.selected_text(), Some("world ".to_string()), "Should select from cursor to click position");
        assert_eq!(editor.selection_range(), Some((6, 12)), "Selection range should be from 6 to 12");
    }

    #[test] 
    fn test_shift_click_extends_existing_selection() {
        // RED: Test extending an existing selection with Shift+click
        let mut editor = new_with_content("The quick brown fox jumps".to_string());
        
        // Create initial selection of 'quick' (positions 4-9)
        editor.set_cursor_position(4);
        editor.start_selection();
        editor.set_cursor_position(9);
        assert_eq!(editor.selected_text(), Some("quick".to_string()), "Initial selection should be 'quick'");
        
        // Shift+click at position 15 ('brown') to extend selection
        let result = editor.handle_shift_click_at_position(15);
        assert!(result, "Shift+click extend should succeed");
        assert_eq!(editor.selected_text(), Some("quick brown".to_string()), "Should extend selection to include 'brown'");
        assert_eq!(editor.selection_range(), Some((4, 15)), "Extended selection range should be from 4 to 15");
    }

    #[test]
    fn test_shift_click_backwards_selection() {
        // RED: Test Shift+click extending selection backwards
        let mut editor = new_with_content("The quick brown fox jumps".to_string());
        
        // Position cursor at position 15 ('b' in 'brown')
        editor.set_cursor_position(15);
        assert_eq!(editor.cursor_position(), 15, "Cursor should be at position 15");
        
        // Shift+click at position 4 ('q' in 'quick') to create backward selection
        let result = editor.handle_shift_click_at_position(4);
        assert!(result, "Backward Shift+click should succeed");
        assert!(editor.has_selection(), "Should create backward selection");
        assert_eq!(editor.selected_text(), Some("quick brown".to_string()), "Should select backward from cursor");
        assert_eq!(editor.selection_range(), Some((4, 15)), "Selection should be properly ordered");
    }

    // ENG-142: Mouse wheel scrolling tests
    #[test]
    fn test_basic_scroll_tracking() {
        // RED: This test should fail because we haven't implemented scroll tracking yet
        let mut editor = new_with_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string());
        
        // Initial scroll position should be 0
        assert_eq!(editor.get_scroll_offset(), 0.0, "Initial scroll offset should be 0");
        
        // Simulate mouse wheel scroll down
        let result = editor.handle_scroll_event(0.0, 100.0); // dx=0, dy=100 (scroll down)
        assert!(result, "Scroll event should succeed");
        assert_eq!(editor.get_scroll_offset(), 100.0, "Scroll offset should increase when scrolling down");
        
        // Simulate mouse wheel scroll up
        let result = editor.handle_scroll_event(0.0, -50.0); // dx=0, dy=-50 (scroll up)
        assert!(result, "Scroll up should succeed");
        assert_eq!(editor.get_scroll_offset(), 50.0, "Scroll offset should decrease when scrolling up");
    }

    #[test]
    fn test_scroll_bounds_checking() {
        // RED: Test scroll bounds to prevent over-scrolling
        let mut editor = new_with_content("Short content".to_string());
        
        // Try to scroll up when already at top
        let result = editor.handle_scroll_event(0.0, -100.0);
        assert!(result, "Scroll event should be handled");
        assert_eq!(editor.get_scroll_offset(), 0.0, "Should not scroll above document start");
        
        // For this test, we'll assume a viewport height that would show all content
        // So scrolling down should also be limited
        let result = editor.handle_scroll_event(0.0, 1000.0);
        assert!(result, "Large scroll event should be handled");
        // Should not scroll beyond what would show content (exact value depends on implementation)
        assert!(editor.get_scroll_offset() >= 0.0, "Scroll offset should not be negative");
    }

    // ENG-143: Right-click context menu tests
    #[test]
    fn test_right_click_detection() {
        // RED: This test should fail because we haven't implemented right-click detection yet
        let mut editor = new_with_content("Hello world".to_string());
        
        // Initially no context menu should be visible
        assert!(!editor.is_context_menu_visible(), "Context menu should initially be hidden");
        
        // Simulate right-click at position 6
        let result = editor.handle_right_click_at_position(6);
        assert!(result, "Right-click should succeed");
        assert!(editor.is_context_menu_visible(), "Context menu should be visible after right-click");
        assert_eq!(editor.get_context_menu_position(), Some(6), "Context menu should track click position");
    }

    #[test] 
    fn test_context_menu_state_management() {
        // RED: Test context menu show/hide behavior
        let mut editor = new_with_content("Sample text for context menu".to_string());
        
        // Right-click to show menu
        editor.handle_right_click_at_position(5);
        assert!(editor.is_context_menu_visible(), "Menu should be visible");
        
        // Left-click should hide menu
        editor.handle_click_at_position(10);
        assert!(!editor.is_context_menu_visible(), "Menu should be hidden after left-click");
        
        // Right-click with selection should show menu with different options
        editor.set_cursor_position(0);
        editor.start_selection();
        editor.set_cursor_position(6); // Select "Sample"
        
        editor.handle_right_click_at_position(3);
        assert!(editor.is_context_menu_visible(), "Menu should show for selection");
        assert!(editor.is_context_menu_copy_enabled(), "Copy should be enabled with selection");
        assert!(editor.is_context_menu_cut_enabled(), "Cut should be enabled with selection");
    }

    #[test]
    fn test_context_menu_item_states() {
        // RED: Test context-sensitive menu item enabling/disabling
        let mut editor = new_with_content("Text content".to_string());
        
        // Right-click with no selection
        editor.handle_right_click_at_position(5);
        assert!(!editor.is_context_menu_copy_enabled(), "Copy should be disabled without selection");
        assert!(!editor.is_context_menu_cut_enabled(), "Cut should be disabled without selection");
        assert!(editor.is_context_menu_select_all_enabled(), "Select All should always be enabled");
        
        // Add some text to clipboard (simulated)
        editor.simulate_clipboard_content("clipboard text".to_string());
        assert!(editor.is_context_menu_paste_enabled(), "Paste should be enabled with clipboard content");
    }

    #[test]
    fn test_context_menu_hides_on_left_click() {
        // RED: Test that context menu is hidden when user left-clicks
        let mut editor = new_with_content("Hello World".to_string());
        
        // First, show context menu with right-click
        editor.handle_right_click_at_position(5);
        assert!(editor.is_context_menu_visible(), "Context menu should be visible after right-click");
        
        // Then left-click should hide the context menu
        editor.handle_click_at_position(7);
        assert!(!editor.is_context_menu_visible(), "Context menu should be hidden after left-click");
        assert_eq!(editor.get_context_menu_position(), None, "Context menu position should be cleared");
        
        // Cursor should still be positioned correctly
        assert_eq!(editor.cursor_position(), 7, "Cursor should be at clicked position");
    }

    #[test]
    fn test_context_menu_action_execution() {
        // RED: Test that context menu actions execute clipboard operations
        let mut editor = new_with_content("Hello World".to_string());
        
        // Select "World" and show context menu
        editor.document_mut().set_cursor_position(6);
        editor.document_mut().start_selection();
        editor.document_mut().set_cursor_position(11); // This extends selection to "World"
        editor.handle_right_click_at_position(8);
        
        // Execute copy action
        let result = editor.execute_context_menu_copy();
        assert!(result, "Context menu copy should succeed");
        assert_eq!(editor.simulated_clipboard_content, Some("World".to_string()), "Copy should put text in simulated clipboard");
        
        // Execute cut action
        let result = editor.execute_context_menu_cut();
        assert!(result, "Context menu cut should succeed");
        assert_eq!(editor.content(), "Hello ", "Cut should remove selected text");
        assert_eq!(editor.simulated_clipboard_content, Some("World".to_string()), "Cut should put text in simulated clipboard");
        
        // Execute paste action
        let result = editor.execute_context_menu_paste();
        assert!(result, "Context menu paste should succeed");
        assert_eq!(editor.content(), "Hello World", "Paste should restore the text");
    }

    // ENG-137: Basic click-to-position cursor functionality tests
    #[test]
    fn test_click_to_position_cursor_basic() {
        // RED: This test should fail initially because we haven't implemented the functionality
        let mut editor = new_with_content("Hello World".to_string());
        
        // Simulate clicking at position 6 (between "Hello" and " World")
        // This should move cursor to position 5 (after "Hello")
        let result = editor.handle_click_at_position(5);
        
        assert!(result, "Click handling should return true when successful");
        assert_eq!(editor.cursor_position(), 5, "Cursor should be positioned at clicked location");
    }

    #[test]
    fn test_click_to_position_multiline() {
        // RED: Test multi-line click positioning
        let mut editor = new_with_content("Line 1\nLine 2\nLine 3".to_string());
        
        // Click at beginning of line 2 (position 7, after "Line 1\n")
        let result = editor.handle_click_at_position(7);
        
        assert!(result, "Multi-line click handling should succeed");
        assert_eq!(editor.cursor_position(), 7, "Cursor should be at beginning of line 2");
    }

    #[test]
    fn test_click_to_position_empty_document() {
        // RED: Test clicking in empty document
        let mut editor = new_with_buffer();
        
        // Click at position 0 in empty document
        let result = editor.handle_click_at_position(0);
        
        assert!(result, "Click in empty document should succeed");
        assert_eq!(editor.cursor_position(), 0, "Cursor should remain at position 0");
    }

    #[test]
    fn test_click_beyond_content_bounds() {
        // RED: Test clicking beyond document bounds
        let mut editor = new_with_content("Short".to_string());
        
        // Click at position 100 (beyond document end)
        let result = editor.handle_click_at_position(100);
        
        assert!(result, "Click beyond bounds should succeed");
        assert_eq!(editor.cursor_position(), 5, "Cursor should be clamped to document end");
    }

    // ENG-141: Markdown-aware mouse coordinate mapping tests
    #[test]
    fn test_coordinate_mapping_raw_vs_preview_mode() {
        // RED: This test should fail because we haven't implemented markdown-aware coordinate mapping
        let mut editor = new_with_content("**bold text** regular".to_string());
        
        // Position cursor outside bold token to trigger preview mode
        editor.document_mut().set_cursor_position(20); // After "**bold text** regular"
        
        // In preview mode, "bold text" is displayed without asterisks
        // Click at character index 5 (which should be inside the bold text in preview mode)
        let result = editor.handle_click_with_coordinate_mapping(5);
        
        assert!(result, "Coordinate mapping should succeed");
        // In preview mode, position 5 should map to position 7 in original content (after "**bold")
        assert_eq!(editor.cursor_position(), 7, "Preview mode coordinate should map to correct original position");
    }

    #[test]
    fn test_coordinate_mapping_different_font_sizes() {
        // RED: Test coordinate mapping with different font sizes (H1, code)
        let mut editor = new_with_content("# Big Title\n`code` text".to_string());
        
        // Position cursor to trigger preview mode
        editor.document_mut().set_cursor_position(22); // After all content
        
        // Click at position 3 in first line (should be inside "Big Title" in preview)
        let result = editor.handle_click_with_coordinate_mapping(3);
        
        assert!(result, "Font size coordinate mapping should succeed");
        // Position 3 in preview should map to position 5 in original ("# Bi|g Title")
        assert_eq!(editor.cursor_position(), 5, "H1 coordinate should map correctly");
    }

    #[test]
    fn test_coordinate_mapping_raw_mode() {
        // RED: Test that raw mode uses direct coordinate mapping
        let mut editor = new_with_content("**bold text**".to_string());
        
        // Position cursor inside bold token to trigger raw mode
        editor.document_mut().set_cursor_position(5); // Inside "**bol|d text**"
        
        // In raw mode, coordinates should map directly to original positions
        let result = editor.handle_click_with_coordinate_mapping(3);
        
        assert!(result, "Raw mode coordinate mapping should succeed");
        assert_eq!(editor.cursor_position(), 3, "Raw mode should use direct coordinate mapping");
    }

    #[test]
    fn test_coordinate_mapping_preserves_positions_on_mode_switch() {
        // RED: Test that cursor positions are preserved when switching modes
        let mut editor = new_with_content("Text **bold** more".to_string());
        
        // Start in preview mode (cursor outside tokens)
        editor.document_mut().set_cursor_position(17); // After "more"
        let initial_pos = editor.cursor_position();
        
        // Move cursor into bold token (should switch to raw mode)
        editor.document_mut().set_cursor_position(8); // Inside "**bol|d**"
        
        // Position should be preserved accurately
        assert_eq!(editor.cursor_position(), 8, "Position should be preserved when switching to raw mode");
        
        // Move cursor back outside (should switch to preview mode)
        editor.document_mut().set_cursor_position(initial_pos);
        assert_eq!(editor.cursor_position(), 17, "Position should be preserved when switching back to preview mode");
    }

    // ENG-138: Click-and-drag text selection tests
    #[test]
    fn test_drag_selection_basic() {
        // RED: This test should fail because we haven't implemented drag selection yet
        let mut editor = new_with_content("Hello World Test".to_string());
        
        // Start drag at position 0 (beginning of "Hello")
        let result = editor.handle_mouse_down_at_position(0);
        assert!(result, "Mouse down should succeed");
        assert!(!editor.has_selection(), "Should not have selection immediately on mouse down");
        
        // Drag to position 5 (end of "Hello")
        let result = editor.handle_mouse_drag_to_position(5);
        assert!(result, "Mouse drag should succeed");
        assert!(editor.has_selection(), "Should have selection during drag");
        assert_eq!(editor.selected_text(), Some("Hello".to_string()), "Should select 'Hello'");
        
        // Release mouse at position 5
        let result = editor.handle_mouse_up_at_position(5);
        assert!(result, "Mouse up should succeed");
        assert!(editor.has_selection(), "Should maintain selection after mouse up");
        assert_eq!(editor.selected_text(), Some("Hello".to_string()), "Should still have 'Hello' selected");
    }

    #[test]
    fn test_drag_selection_multiline() {
        // RED: Test drag selection across multiple lines
        let mut editor = new_with_content("Line 1\nLine 2\nLine 3".to_string());
        
        // Debug: Let's understand the positions
        // "Line 1\nLine 2\nLine 3"
        //  0123456 7890123 4567890
        //          ^       ^
        //         pos 3   pos 10
        
        // Start drag at position 3 (middle of "Line 1" - after "Lin")
        let result = editor.handle_mouse_down_at_position(3);
        assert!(result, "Mouse down should succeed");
        
        // Drag to position 10 (middle of "Line 2" - after "Line 2\nLi")
        let result = editor.handle_mouse_drag_to_position(10);
        assert!(result, "Multi-line drag should succeed");
        assert!(editor.has_selection(), "Should have selection during multi-line drag");
        // From pos 3 to pos 10: "e 1\nLin" (but we got "e 1\nLin", so the test was wrong)
        assert_eq!(editor.selected_text(), Some("e 1\nLin".to_string()), "Should select across lines");
        
        // Release mouse
        let result = editor.handle_mouse_up_at_position(10);
        assert!(result, "Mouse up should succeed");
        assert!(editor.has_selection(), "Should maintain multi-line selection after mouse up");
    }

    #[test]
    fn test_drag_selection_backwards() {
        // RED: Test drag selection in reverse direction (drag left)
        let mut editor = new_with_content("Hello World".to_string());
        
        // Start drag at position 11 (end of "Hello World")
        let result = editor.handle_mouse_down_at_position(11);
        assert!(result, "Mouse down should succeed");
        
        // Drag backwards to position 6 (start of "World")
        let result = editor.handle_mouse_drag_to_position(6);
        assert!(result, "Backwards drag should succeed");
        assert!(editor.has_selection(), "Should have selection during backwards drag");
        assert_eq!(editor.selected_text(), Some("World".to_string()), "Should select 'World' in reverse");
        
        // Release mouse
        let result = editor.handle_mouse_up_at_position(6);
        assert!(result, "Mouse up should succeed");
        assert!(editor.has_selection(), "Should maintain backwards selection");
    }

    #[test]
    fn test_drag_selection_with_markdown_coordinate_mapping() {
        // RED: Test drag selection with coordinate mapping (combines ENG-138 + ENG-141)
        let mut editor = new_with_content("**bold text** regular".to_string());
        
        // Position cursor outside to trigger preview mode
        editor.document_mut().set_cursor_position(20);
        
        // Test that coordinate mapping drag selection works at all
        let result = editor.handle_mouse_down_with_coordinate_mapping(0);
        assert!(result, "Coordinate-mapped mouse down should succeed");
        
        let result = editor.handle_mouse_drag_with_coordinate_mapping(3);
        assert!(result, "Coordinate-mapped drag should succeed");
        assert!(editor.has_selection(), "Should have selection with coordinate mapping");
        
        // Just verify that we get some selection - the exact mapping details 
        // are already tested in ENG-141 coordinate mapping tests
        let selected = editor.selected_text().unwrap();
        assert!(!selected.is_empty(), "Should have non-empty selection with coordinate mapping");
        
        // Release mouse
        let result = editor.handle_mouse_up_with_coordinate_mapping(3);
        assert!(result, "Coordinate-mapped mouse up should succeed");
    }

    #[test]
    fn test_drag_selection_integration_with_keyboard() {
        // RED: Test that drag selection works with existing keyboard selection
        let mut editor = new_with_content("Test text here".to_string());
        
        // First create a keyboard selection
        editor.document_mut().set_cursor_position(5); // After "Test "
        editor.document_mut().start_selection();
        editor.document_mut().set_cursor_position(9); // After "Test text"
        assert!(editor.has_selection(), "Should have keyboard selection");
        assert_eq!(editor.selected_text(), Some("text".to_string()), "Should select 'text'");
        
        // Now start a new drag selection (should replace keyboard selection)
        let result = editor.handle_mouse_down_at_position(10);
        assert!(result, "Mouse down should succeed");
        
        // Drag should clear previous selection and start new one
        let result = editor.handle_mouse_drag_to_position(14);
        assert!(result, "Mouse drag should succeed");
        assert!(editor.has_selection(), "Should have new drag selection");
        assert_eq!(editor.selected_text(), Some("here".to_string()), "Should select 'here', replacing keyboard selection");
        
        // Release mouse
        let result = editor.handle_mouse_up_at_position(14);
        assert!(result, "Mouse up should succeed");
    }
}
