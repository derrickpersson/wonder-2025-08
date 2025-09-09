pub(self) mod cursor_diagnostics;
pub(self) mod element;
#[cfg(test)]
mod element_wrapping_tests;
mod gpui_traits;
#[cfg(test)]
mod mouse_wrapping_tests;
mod keyboard;
mod mouse;
mod rendering;
#[cfg(test)]
mod scroll_integration_tests;

use crate::core::{TextDocument, CursorMovementService, ViewportManager};
#[cfg(test)]
mod tests;
use crate::hybrid_renderer::HybridTextRenderer;
use crate::input::InputRouter;
use crate::rendering::VisualLineManager;
use gpui::{Bounds, Context, FocusHandle, Pixels};

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
    // Actual element bounds from GPUI (updated during paint)
    element_bounds: Option<Bounds<Pixels>>,
    // Actual line positions from GPUI rendering (Y coordinates of each line)
    actual_line_positions: Vec<f32>,
    // Visual line manager for storing actual GPUI-rendered visual lines
    visual_line_manager: VisualLineManager,
    // Unified cursor movement service with visual-line-first approach
    cursor_movement: CursorMovementService,
    // Viewport management for scrolling support
    viewport_manager: ViewportManager,
}

impl MarkdownEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let input_router = InputRouter::new();
        
        Self {
            document: TextDocument::new(),
            input_router,
            hybrid_renderer: HybridTextRenderer::with_line_wrapping(gpui::px(600.0)),
            focused: true, // Start focused
            focus_handle,
            // ENG-138: Initialize mouse state
            is_mouse_down: false,
            mouse_down_position: None,
            // ENG-139: Initialize click tracking
            last_click_time: std::time::Instant::now(),
            last_click_position: None,
            element_bounds: None,
            actual_line_positions: Vec::new(),
            visual_line_manager: VisualLineManager::new(),
            cursor_movement: CursorMovementService::new(),
            viewport_manager: ViewportManager::new(24.0), // 24px line height
        }
    }

    pub fn new_with_content(content: String, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let input_router = InputRouter::new();
        
        Self {
            document: TextDocument::with_content(content),
            input_router,
            hybrid_renderer: HybridTextRenderer::with_line_wrapping(gpui::px(600.0)),
            focused: true, // Start focused
            focus_handle,
            // ENG-138: Initialize mouse state
            is_mouse_down: false,
            mouse_down_position: None,
            // ENG-139: Initialize click tracking
            last_click_time: std::time::Instant::now(),
            last_click_position: None,
            element_bounds: None,
            actual_line_positions: Vec::new(),
            visual_line_manager: VisualLineManager::new(),
            cursor_movement: CursorMovementService::new(),
            viewport_manager: ViewportManager::new(24.0), // 24px line height
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
        // Ensure cursor remains visible after character input
        self.ensure_cursor_visible();
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
    
    // Update the visual line manager with actual GPUI-rendered visual lines
    pub fn update_visual_line_manager(&mut self, visual_line_manager: VisualLineManager) {
        self.visual_line_manager = visual_line_manager;
    }
    
    // Provide access to visual line manager for mouse coordinate conversion
    pub fn visual_line_manager(&self) -> &VisualLineManager {
        &self.visual_line_manager
    }
    
    // Note: update_cursor_visual_line removed - now handled by VisualLineManager

    // Provide access to document for more complex operations
    pub fn document(&self) -> &TextDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut TextDocument {
        &mut self.document
    }
    
    // Viewport management methods
    pub fn viewport_manager(&self) -> &ViewportManager {
        &self.viewport_manager
    }
    
    pub fn viewport_manager_mut(&mut self) -> &mut ViewportManager {
        &mut self.viewport_manager
    }
    
    /// Update viewport height based on element bounds
    pub fn update_viewport_from_bounds(&mut self, bounds: Bounds<Pixels>) {
        let viewport_height = bounds.size.height.0;
        self.viewport_manager.scroll_state_mut().set_viewport_height(viewport_height);
    }
    
    /// Get current scroll offset
    pub fn scroll_offset(&self) -> f32 {
        self.viewport_manager.scroll_state().vertical_offset()
    }
    
    /// Scroll by a pixel amount (positive = down, negative = up)
    pub fn scroll_by(&mut self, delta: f32) {
        self.viewport_manager.scroll_state_mut().scroll_by(delta);
    }
    
    /// Scroll to a specific position
    pub fn scroll_to(&mut self, position: f32) {
        self.viewport_manager.scroll_state_mut().scroll_to(position);
    }
    
    /// Scroll by a number of lines
    pub fn scroll_by_lines(&mut self, line_delta: i32) {
        self.viewport_manager.scroll_by_lines(line_delta);
    }
    
    /// Scroll by pages (viewport height)
    pub fn scroll_by_page(&mut self, page_delta: i32) {
        self.viewport_manager.scroll_by_page(page_delta);
    }
    
    /// Scroll to top of document
    pub fn scroll_to_top(&mut self) {
        self.viewport_manager.scroll_to_top();
    }
    
    /// Scroll to bottom of document  
    pub fn scroll_to_bottom(&mut self) {
        self.viewport_manager.scroll_to_bottom();
    }
    
    /// Ensure cursor is visible by scrolling if necessary
    pub fn ensure_cursor_visible(&mut self) {
        // Calculate which line the cursor is on
        let cursor_position = self.document.cursor_position();
        let content = self.document.content();
        // CRITICAL FIX: Use rope's line calculation instead of string slicing to avoid Unicode boundary issues
        let cursor_line = self.document.rope().char_to_line(cursor_position);
        
        // Calculate total line count
        let total_lines = content.lines().count().max(1);
        
        // ENG-185: Use actual height from VisualLineManager - NO ESTIMATES  
        // Get the real document height from the visual line rendering system
        if let Some(actual_height) = self.visual_line_manager.get_total_document_height(24.0) {
            // Use the actual height calculated from rendered visual lines
            self.viewport_manager.scroll_state_mut().set_document_height(actual_height);
        }
        // NO FALLBACK - wait for actual values from rendering
        
        // Ensure cursor line is visible
        self.viewport_manager.ensure_line_visible(cursor_line, total_lines);
    }
}

// ActionHandler implementation for MarkdownEditor
// This ensures auto-scroll happens after all cursor movements and text modifications
impl crate::input::actions::ActionHandler for MarkdownEditor {
    fn handle_action(&mut self, action: crate::input::actions::EditorAction) -> bool {
        // ENG-191: Handle scroll actions directly (not text document operations)
        match &action {
            crate::input::actions::EditorAction::ScrollUp => {
                self.scroll_by_lines(-3); // Scroll up 3 lines
                return true;
            }
            crate::input::actions::EditorAction::ScrollDown => {
                self.scroll_by_lines(3); // Scroll down 3 lines
                return true;
            }
            crate::input::actions::EditorAction::ScrollPageUp => {
                self.scroll_by_page(-1); // Scroll up one page
                return true;
            }
            crate::input::actions::EditorAction::ScrollPageDown => {
                self.scroll_by_page(1); // Scroll down one page
                return true;
            }
            crate::input::actions::EditorAction::ScrollToTop => {
                self.scroll_to_top(); // Scroll to document start
                return true;
            }
            crate::input::actions::EditorAction::ScrollToBottom => {
                self.scroll_to_bottom(); // Scroll to document end
                return true;
            }
            _ => {
                // For other actions, delegate to document first
            }
        }
        
        // Delegate to document for non-scroll actions
        let handled = self.document.handle_action(action.clone());
        
        // After any action that might move the cursor, ensure it remains visible
        // This includes cursor movements, text insertion, deletion, etc.
        if handled {
            match action {
                crate::input::actions::EditorAction::MoveCursor(_) |
                crate::input::actions::EditorAction::ExtendSelection(_) |
                crate::input::actions::EditorAction::InsertChar(_) |
                crate::input::actions::EditorAction::InsertText(_) |
                crate::input::actions::EditorAction::Backspace |
                crate::input::actions::EditorAction::Delete |
                crate::input::actions::EditorAction::SelectAll |
                crate::input::actions::EditorAction::ClearSelection => {
                    // Ensure cursor visibility after actions that may change cursor position
                    self.ensure_cursor_visible();
                }
                _ => {
                    // Other actions don't affect cursor position, no auto-scroll needed
                }
            }
        }
        
        handled
    }
}


