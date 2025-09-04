pub(self) mod cursor_diagnostics;
pub(self) mod element;
mod gpui_traits;
mod keyboard;
mod mouse;
mod rendering;

use crate::core::TextDocument;
#[cfg(test)]
mod tests;
use crate::hybrid_renderer::HybridTextRenderer;
use crate::input::InputRouter;
use gpui::{prelude::*, Bounds, Context, FocusHandle, Pixels};

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
    // Actual element bounds from GPUI (updated during paint)
    element_bounds: Option<Bounds<Pixels>>,
    // Actual line positions from GPUI rendering (Y coordinates of each line)
    actual_line_positions: Vec<f32>,
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
            element_bounds: None,
            actual_line_positions: Vec::new(),
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
            element_bounds: None,
            actual_line_positions: Vec::new(),
        }
    }

    // Content access
    pub fn content(&self) -> String {
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


    pub fn get_content(&self) -> String {
        self.content()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.handle_char_input(ch);
    }


    // Update the element bounds when we receive them from GPUI
    pub fn update_element_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.element_bounds = Some(bounds);
    }

    // Update the actual line positions from GPUI rendering
    pub fn update_line_positions(&mut self, line_positions: Vec<f32>) {
        self.actual_line_positions = line_positions;
    }

    // Provide access to document for more complex operations
    pub fn document(&self) -> &TextDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut TextDocument {
        &mut self.document
    }
}


