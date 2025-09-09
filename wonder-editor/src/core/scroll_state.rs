
/// Manages scrolling state and viewport calculations for the editor
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollState {
    /// Current vertical scroll offset in pixels (0 = top of document)
    vertical_offset: f32,
    /// Total document height in pixels
    document_height: f32,
    /// Viewport height in pixels (visible area)
    viewport_height: f32,
}

impl ScrollState {
    /// Create a new scroll state
    pub fn new() -> Self {
        Self {
            vertical_offset: 0.0,
            document_height: 0.0,
            viewport_height: 0.0,
        }
    }
    
    /// Create scroll state with specified viewport height
    pub fn with_viewport_height(viewport_height: f32) -> Self {
        Self {
            vertical_offset: 0.0,
            document_height: 0.0,
            viewport_height,
        }
    }
    
    /// Get current vertical scroll offset
    pub fn vertical_offset(&self) -> f32 {
        self.vertical_offset
    }
    
    /// Get viewport height
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }
    
    /// Get total document height
    pub fn document_height(&self) -> f32 {
        self.document_height
    }
    
    /// Set viewport dimensions
    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
        // Ensure scroll position is still valid after viewport change
        self.clamp_scroll_position();
    }
    
    /// Set total document height
    pub fn set_document_height(&mut self, height: f32) {
        self.document_height = height;
        // Ensure scroll position is still valid after document size change
        self.clamp_scroll_position();
    }
    
    /// Scroll by a relative amount (positive = down, negative = up)
    pub fn scroll_by(&mut self, delta: f32) {
        self.vertical_offset += delta;
        self.clamp_scroll_position();
    }
    
    /// Scroll to an absolute position
    pub fn scroll_to(&mut self, position: f32) {
        self.vertical_offset = position;
        self.clamp_scroll_position();
    }
    
    /// Get the maximum valid scroll position
    pub fn max_scroll_position(&self) -> f32 {
        (self.document_height - self.viewport_height).max(0.0)
    }
    
    /// Check if scrolling is possible (document larger than viewport)
    pub fn is_scrollable(&self) -> bool {
        self.document_height > self.viewport_height
    }
    
    /// Get the range of Y coordinates currently visible in the viewport
    pub fn visible_y_range(&self) -> (f32, f32) {
        let top = self.vertical_offset;
        let bottom = self.vertical_offset + self.viewport_height;
        (top, bottom)
    }
    
    /// Check if a Y coordinate is currently visible
    pub fn is_y_visible(&self, y: f32) -> bool {
        let (top, bottom) = self.visible_y_range();
        y >= top && y < bottom
    }
    
    /// Ensure scroll position is within valid bounds
    fn clamp_scroll_position(&mut self) {
        let max_scroll = self.max_scroll_position();
        self.vertical_offset = self.vertical_offset.max(0.0).min(max_scroll);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state_initialization() {
        let scroll_state = ScrollState::new();
        
        assert_eq!(scroll_state.vertical_offset(), 0.0);
        assert_eq!(scroll_state.viewport_height(), 0.0);
        assert_eq!(scroll_state.document_height(), 0.0);
        assert!(!scroll_state.is_scrollable());
    }
    
    #[test]
    fn test_scroll_state_with_viewport_height() {
        let scroll_state = ScrollState::with_viewport_height(600.0);
        
        assert_eq!(scroll_state.vertical_offset(), 0.0);
        assert_eq!(scroll_state.viewport_height(), 600.0);
        assert_eq!(scroll_state.document_height(), 0.0);
        assert!(!scroll_state.is_scrollable());
    }
    
    #[test]
    fn test_scroll_by_positive() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        
        scroll_state.scroll_by(100.0);
        assert_eq!(scroll_state.vertical_offset(), 100.0);
        
        scroll_state.scroll_by(50.0);
        assert_eq!(scroll_state.vertical_offset(), 150.0);
    }
    
    #[test]
    fn test_scroll_by_negative() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        scroll_state.scroll_to(300.0);
        
        scroll_state.scroll_by(-100.0);
        assert_eq!(scroll_state.vertical_offset(), 200.0);
        
        scroll_state.scroll_by(-50.0);
        assert_eq!(scroll_state.vertical_offset(), 150.0);
    }
    
    #[test]
    fn test_scroll_to_absolute_position() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        
        scroll_state.scroll_to(400.0);
        assert_eq!(scroll_state.vertical_offset(), 400.0);
        
        scroll_state.scroll_to(0.0);
        assert_eq!(scroll_state.vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_scroll_bounds_clamping() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        
        // Test scrolling beyond bottom
        scroll_state.scroll_to(800.0); // max should be 1200-600=600
        assert_eq!(scroll_state.vertical_offset(), 600.0);
        
        // Test scrolling beyond top
        scroll_state.scroll_to(-100.0);
        assert_eq!(scroll_state.vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_scroll_bounds_with_small_document() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(400.0); // Document smaller than viewport
        
        // Should not be able to scroll at all
        scroll_state.scroll_to(100.0);
        assert_eq!(scroll_state.vertical_offset(), 0.0);
        assert!(!scroll_state.is_scrollable());
    }
    
    #[test]
    fn test_max_scroll_position() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        
        // Document larger than viewport
        scroll_state.set_document_height(1200.0);
        assert_eq!(scroll_state.max_scroll_position(), 600.0);
        
        // Document smaller than viewport
        scroll_state.set_document_height(400.0);
        assert_eq!(scroll_state.max_scroll_position(), 0.0);
    }
    
    #[test]
    fn test_is_scrollable() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        
        // Document smaller than viewport
        scroll_state.set_document_height(400.0);
        assert!(!scroll_state.is_scrollable());
        
        // Document equal to viewport
        scroll_state.set_document_height(600.0);
        assert!(!scroll_state.is_scrollable());
        
        // Document larger than viewport
        scroll_state.set_document_height(800.0);
        assert!(scroll_state.is_scrollable());
    }
    
    #[test]
    fn test_visible_y_range() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        
        // At top
        assert_eq!(scroll_state.visible_y_range(), (0.0, 600.0));
        
        // Scrolled down
        scroll_state.scroll_to(200.0);
        assert_eq!(scroll_state.visible_y_range(), (200.0, 800.0));
    }
    
    #[test]
    fn test_is_y_visible() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        scroll_state.scroll_to(200.0); // Viewing 200-800
        
        // Should be visible
        assert!(scroll_state.is_y_visible(300.0));
        assert!(scroll_state.is_y_visible(200.0)); // Top edge
        assert!(scroll_state.is_y_visible(799.0)); // Just before bottom edge
        
        // Should not be visible
        assert!(!scroll_state.is_y_visible(100.0)); // Above viewport
        assert!(!scroll_state.is_y_visible(800.0)); // At/beyond bottom edge
        assert!(!scroll_state.is_y_visible(900.0)); // Below viewport
    }
    
    #[test]
    fn test_viewport_height_change_clamps_position() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        scroll_state.scroll_to(600.0); // At max scroll position
        
        // Increase viewport height - should reduce max scroll and clamp position
        scroll_state.set_viewport_height(800.0);
        assert_eq!(scroll_state.max_scroll_position(), 400.0);
        assert_eq!(scroll_state.vertical_offset(), 400.0); // Should be clamped
    }
    
    #[test]
    fn test_document_height_change_clamps_position() {
        let mut scroll_state = ScrollState::with_viewport_height(600.0);
        scroll_state.set_document_height(1200.0);
        scroll_state.scroll_to(600.0); // At max scroll position
        
        // Reduce document height - should clamp position
        scroll_state.set_document_height(800.0);
        assert_eq!(scroll_state.max_scroll_position(), 200.0);
        assert_eq!(scroll_state.vertical_offset(), 200.0); // Should be clamped
    }
}