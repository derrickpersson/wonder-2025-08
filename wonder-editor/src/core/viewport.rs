use super::ScrollState;

/// Manages viewport calculations and visible line range queries
#[derive(Debug, Clone)]
pub struct ViewportManager {
    /// Current scroll state
    scroll_state: ScrollState,
    /// Height of each line in pixels (assumed uniform for now)
    line_height: f32,
}

impl ViewportManager {
    /// Create a new viewport manager
    pub fn new(line_height: f32) -> Self {
        Self {
            scroll_state: ScrollState::new(),
            line_height,
        }
    }
    
    /// Create viewport manager with specific scroll state
    pub fn with_scroll_state(scroll_state: ScrollState, line_height: f32) -> Self {
        Self {
            scroll_state,
            line_height,
        }
    }
    
    /// Get reference to scroll state
    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }
    
    /// Get mutable reference to scroll state
    pub fn scroll_state_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }
    
    /// Get line height
    pub fn line_height(&self) -> f32 {
        self.line_height
    }
    
    /// Set line height
    pub fn set_line_height(&mut self, height: f32) {
        self.line_height = height;
    }
    
    /// Calculate which lines are visible based on current scroll position
    /// Returns (first_visible_line_index, last_visible_line_index)
    pub fn get_visible_line_range(&self, total_lines: usize) -> (usize, usize) {
        if total_lines == 0 {
            return (0, 0);
        }
        
        let (visible_top, visible_bottom) = self.scroll_state.visible_y_range();
        
        // Calculate first visible line (round down)
        let first_line = (visible_top / self.line_height).floor() as usize;
        let first_line = first_line.min(total_lines.saturating_sub(1));
        
        // Calculate last visible line (round up to ensure we include partially visible lines)
        let last_line = (visible_bottom / self.line_height).ceil() as usize;
        let last_line = last_line.min(total_lines);
        
        (first_line, last_line)
    }
    
    /// Check if a specific line index is visible
    pub fn is_line_visible(&self, line_index: usize, total_lines: usize) -> bool {
        let (first, last) = self.get_visible_line_range(total_lines);
        line_index >= first && line_index < last
    }
    
    /// Get Y position for a specific line index
    pub fn get_line_y_position(&self, line_index: usize) -> f32 {
        line_index as f32 * self.line_height
    }
    
    /// Get the line index at a specific Y position
    pub fn get_line_at_y_position(&self, y: f32) -> usize {
        (y / self.line_height).floor() as usize
    }
    
    /// Calculate total document height for a given number of lines
    pub fn calculate_document_height(&self, total_lines: usize) -> f32 {
        total_lines as f32 * self.line_height
    }
    
    /// Update document height based on line count
    pub fn update_document_height(&mut self, total_lines: usize) {
        let new_height = self.calculate_document_height(total_lines);
        self.scroll_state.set_document_height(new_height);
    }
    
    /// Ensure a specific line is visible by scrolling if necessary
    pub fn ensure_line_visible(&mut self, line_index: usize, total_lines: usize) -> bool {
        let line_y = self.get_line_y_position(line_index);
        let line_bottom = line_y + self.line_height;
        
        let (visible_top, visible_bottom) = self.scroll_state.visible_y_range();
        
        // Check if line is already fully visible
        if line_y >= visible_top && line_bottom <= visible_bottom {
            return false; // No scroll needed
        }
        
        // Scroll to make line visible
        if line_y < visible_top {
            // Line is above viewport - scroll up to show it at top
            self.scroll_state.scroll_to(line_y);
        } else if line_bottom > visible_bottom {
            // Line is below viewport - scroll down to show it at bottom
            let new_scroll = line_bottom - self.scroll_state.viewport_height();
            self.scroll_state.scroll_to(new_scroll);
        }
        
        true // Scroll was performed
    }
    
    /// Scroll by a number of lines
    pub fn scroll_by_lines(&mut self, line_delta: i32) {
        let pixel_delta = line_delta as f32 * self.line_height;
        self.scroll_state.scroll_by(pixel_delta);
    }
    
    /// Scroll by a page (viewport height)
    pub fn scroll_by_page(&mut self, page_delta: i32) {
        let pixel_delta = page_delta as f32 * self.scroll_state.viewport_height();
        self.scroll_state.scroll_by(pixel_delta);
    }
    
    /// Scroll to top of document
    pub fn scroll_to_top(&mut self) {
        self.scroll_state.scroll_to(0.0);
    }
    
    /// Scroll to bottom of document
    pub fn scroll_to_bottom(&mut self) {
        let max_scroll = self.scroll_state.max_scroll_position();
        self.scroll_state.scroll_to(max_scroll);
    }
    
    /// Get viewport information for rendering (scroll_offset, viewport_height, total_lines)
    pub fn get_viewport_info(&self) -> (f32, f32, usize) {
        let total_lines = (self.scroll_state.document_height() / self.line_height).ceil() as usize;
        (
            self.scroll_state.vertical_offset(),
            self.scroll_state.viewport_height(),
            total_lines,
        )
    }
}

impl Default for ViewportManager {
    fn default() -> Self {
        Self::new(24.0) // Default line height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_manager_creation() {
        let viewport = ViewportManager::new(24.0);
        assert_eq!(viewport.line_height(), 24.0);
        assert_eq!(viewport.scroll_state().vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_get_visible_line_range_no_scroll() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport.update_document_height(20); // 20 total lines
        
        let (first, last) = viewport.get_visible_line_range(20);
        assert_eq!(first, 0);
        assert_eq!(last, 10);
    }
    
    #[test]
    fn test_get_visible_line_range_scrolled() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport.update_document_height(20); // 20 total lines
        viewport.scroll_state_mut().scroll_to(120.0); // Scroll down 5 lines
        
        let (first, last) = viewport.get_visible_line_range(20);
        assert_eq!(first, 5); // Start from line 5
        assert_eq!(last, 15); // Show up to line 15
    }
    
    #[test]
    fn test_get_visible_line_range_at_bottom() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport.update_document_height(20); // 20 total lines
        viewport.scroll_to_bottom();
        
        let (first, last) = viewport.get_visible_line_range(20);
        assert_eq!(first, 10); // Start from line 10
        assert_eq!(last, 20); // Show up to line 20
    }
    
    #[test]
    fn test_get_visible_line_range_empty_document() {
        let viewport = ViewportManager::new(24.0);
        let (first, last) = viewport.get_visible_line_range(0);
        assert_eq!(first, 0);
        assert_eq!(last, 0);
    }
    
    #[test]
    fn test_get_visible_line_range_small_document() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        
        // Document with only 5 lines
        let (first, last) = viewport.get_visible_line_range(5);
        assert_eq!(first, 0);
        assert_eq!(last, 5);
    }
    
    #[test]
    fn test_is_line_visible() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport.update_document_height(20); // 20 total lines
        viewport.scroll_state_mut().scroll_to(120.0); // Scroll to show lines 5-14
        
        // Lines 5-14 should be visible
        assert!(viewport.is_line_visible(5, 20));
        assert!(viewport.is_line_visible(10, 20));
        assert!(viewport.is_line_visible(14, 20));
        
        // Lines outside range should not be visible
        assert!(!viewport.is_line_visible(4, 20));
        assert!(!viewport.is_line_visible(15, 20));
    }
    
    #[test]
    fn test_get_line_y_position() {
        let viewport = ViewportManager::new(24.0);
        
        assert_eq!(viewport.get_line_y_position(0), 0.0);
        assert_eq!(viewport.get_line_y_position(1), 24.0);
        assert_eq!(viewport.get_line_y_position(10), 240.0);
    }
    
    #[test]
    fn test_get_line_at_y_position() {
        let viewport = ViewportManager::new(24.0);
        
        assert_eq!(viewport.get_line_at_y_position(0.0), 0);
        assert_eq!(viewport.get_line_at_y_position(24.0), 1);
        assert_eq!(viewport.get_line_at_y_position(23.9), 0); // Just before line 1
        assert_eq!(viewport.get_line_at_y_position(240.0), 10);
    }
    
    #[test]
    fn test_calculate_document_height() {
        let viewport = ViewportManager::new(24.0);
        
        assert_eq!(viewport.calculate_document_height(0), 0.0);
        assert_eq!(viewport.calculate_document_height(10), 240.0);
        assert_eq!(viewport.calculate_document_height(100), 2400.0);
    }
    
    #[test]
    fn test_update_document_height() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        
        viewport.update_document_height(20);
        assert_eq!(viewport.scroll_state().document_height(), 480.0); // 20 * 24
    }
    
    #[test]
    fn test_ensure_line_visible_already_visible() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        
        // Line 5 should be visible (lines 0-9 are visible)
        let scrolled = viewport.ensure_line_visible(5, 20);
        assert!(!scrolled); // No scroll needed
        assert_eq!(viewport.scroll_state().vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_ensure_line_visible_scroll_up() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        viewport.scroll_state_mut().scroll_to(240.0); // Show lines 10-19
        
        // Make line 5 visible (need to scroll up)
        let scrolled = viewport.ensure_line_visible(5, 20);
        assert!(scrolled); // Scroll was needed
        assert_eq!(viewport.scroll_state().vertical_offset(), 120.0); // 5 * 24
    }
    
    #[test]
    fn test_ensure_line_visible_scroll_down() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        // Currently showing lines 0-9
        
        // Make line 15 visible (need to scroll down)
        let scrolled = viewport.ensure_line_visible(15, 20);
        assert!(scrolled); // Scroll was needed
        // Line 15 should be at bottom: (15+1)*24 - 240 = 144
        assert_eq!(viewport.scroll_state().vertical_offset(), 144.0);
    }
    
    #[test]
    fn test_scroll_by_lines() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        
        // Scroll down 3 lines
        viewport.scroll_by_lines(3);
        assert_eq!(viewport.scroll_state().vertical_offset(), 72.0); // 3 * 24
        
        // Scroll up 2 lines
        viewport.scroll_by_lines(-2);
        assert_eq!(viewport.scroll_state().vertical_offset(), 24.0); // 72 - 48
    }
    
    #[test]
    fn test_scroll_by_page() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        
        // Scroll down 1 page
        viewport.scroll_by_page(1);
        assert_eq!(viewport.scroll_state().vertical_offset(), 240.0); // 1 * 240
        
        // Scroll up 1 page
        viewport.scroll_by_page(-1);
        assert_eq!(viewport.scroll_state().vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_scroll_to_top_and_bottom() {
        let mut viewport = ViewportManager::new(24.0);
        viewport.scroll_state_mut().set_viewport_height(240.0);
        viewport.update_document_height(20);
        
        // Scroll to bottom
        viewport.scroll_to_bottom();
        assert_eq!(viewport.scroll_state().vertical_offset(), 240.0); // 480 - 240
        
        // Scroll to top
        viewport.scroll_to_top();
        assert_eq!(viewport.scroll_state().vertical_offset(), 0.0);
    }
}