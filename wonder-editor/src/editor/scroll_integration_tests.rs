#[cfg(test)]
mod tests {
    use crate::core::{ViewportManager, ScrollState};
    use gpui::{Bounds, px, size, point};

    #[test]
    fn test_viewport_manager_initialization() {
        let viewport_manager = ViewportManager::new(24.0);
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 0.0);
        assert_eq!(viewport_manager.scroll_state().viewport_height(), 0.0);
        assert_eq!(viewport_manager.line_height(), 24.0);
    }
    
    #[test]
    fn test_bounds_to_viewport_height_conversion() {
        let bounds = Bounds {
            origin: point(px(0.0), px(0.0)),
            size: size(px(800.0), px(600.0)),
        };
        
        // Test that we can extract viewport height from bounds
        let viewport_height = bounds.size.height.0;
        assert_eq!(viewport_height, 600.0);
        
        // Test with scroll state
        let mut scroll_state = ScrollState::new();
        scroll_state.set_viewport_height(viewport_height);
        assert_eq!(scroll_state.viewport_height(), 600.0);
    }
    
    #[test]
    fn test_line_count_to_document_height() {
        let mut viewport_manager = ViewportManager::new(24.0);
        
        // Test document height calculation
        viewport_manager.update_document_height(10);
        assert_eq!(viewport_manager.scroll_state().document_height(), 240.0); // 10 * 24
        
        viewport_manager.update_document_height(50);
        assert_eq!(viewport_manager.scroll_state().document_height(), 1200.0); // 50 * 24
    }
    
    #[test]
    fn test_visible_line_range_calculation() {
        let mut viewport_manager = ViewportManager::new(24.0);
        viewport_manager.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport_manager.update_document_height(30); // 30 total lines
        
        // At top of document
        let (first, last) = viewport_manager.get_visible_line_range(30);
        assert_eq!(first, 0);
        assert_eq!(last, 10);
        
        // Scrolled down 5 lines
        viewport_manager.scroll_state_mut().scroll_to(120.0); // 5 * 24
        let (first, last) = viewport_manager.get_visible_line_range(30);
        assert_eq!(first, 5);
        assert_eq!(last, 15);
    }
    
    #[test]
    fn test_ensure_line_visible_scroll_logic() {
        let mut viewport_manager = ViewportManager::new(24.0);
        viewport_manager.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        viewport_manager.update_document_height(30); // 30 total lines
        
        // Line 5 should be visible initially (lines 0-9 visible)
        let scrolled = viewport_manager.ensure_line_visible(5, 30);
        assert!(!scrolled); // No scroll needed
        
        // Line 20 needs scrolling down
        let scrolled = viewport_manager.ensure_line_visible(20, 30);
        assert!(scrolled); // Scroll was needed
        
        // Verify line 20 is now visible
        let (first, last) = viewport_manager.get_visible_line_range(30);
        assert!(20 >= first && 20 < last);
    }
    
    #[test]
    fn test_cursor_line_calculation() {
        let content = "Line 0\nLine 1\nLine 2\nLine 3\n";
        
        // Test cursor position to line conversion
        let cursor_at_line_0 = 0;
        let cursor_at_line_1 = 7; // After "Line 0\n"
        let cursor_at_line_2 = 14; // After "Line 0\nLine 1\n"
        let cursor_at_line_3 = 21; // After "Line 0\nLine 1\nLine 2\n"
        
        // Count newlines before cursor position - FIXED: Use character-based slicing to avoid Unicode boundary issues
        let line_0 = content.chars().take(cursor_at_line_0).filter(|&c| c == '\n').count();
        let line_1 = content.chars().take(cursor_at_line_1).filter(|&c| c == '\n').count();
        let line_2 = content.chars().take(cursor_at_line_2).filter(|&c| c == '\n').count();
        let line_3 = content.chars().take(cursor_at_line_3).filter(|&c| c == '\n').count();
        
        assert_eq!(line_0, 0);
        assert_eq!(line_1, 1);
        assert_eq!(line_2, 2);
        assert_eq!(line_3, 3);
    }
    
    #[test]
    fn test_scroll_by_methods() {
        let mut viewport_manager = ViewportManager::new(24.0);
        viewport_manager.scroll_state_mut().set_viewport_height(240.0);
        viewport_manager.update_document_height(50); // Large document
        
        // Test scroll by lines
        viewport_manager.scroll_by_lines(3);
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 72.0); // 3 * 24
        
        // Test scroll by pages
        viewport_manager.scroll_state_mut().scroll_to(0.0); // Reset
        viewport_manager.scroll_by_page(1);
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 240.0); // 1 * viewport_height
        
        // Test scroll to top/bottom
        viewport_manager.scroll_to_bottom();
        let max_scroll = viewport_manager.scroll_state().max_scroll_position();
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), max_scroll);
        
        viewport_manager.scroll_to_top();
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 0.0);
    }
    
    #[test]
    fn test_scroll_bounds_with_different_document_sizes() {
        let mut viewport_manager = ViewportManager::new(24.0);
        viewport_manager.scroll_state_mut().set_viewport_height(240.0); // 10 lines visible
        
        // Small document that fits in viewport
        viewport_manager.update_document_height(5); // 5 lines = 120px
        viewport_manager.scroll_state_mut().scroll_to(100.0);
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 0.0); // Clamped to 0
        
        // Large document
        viewport_manager.update_document_height(50); // 50 lines = 1200px
        viewport_manager.scroll_state_mut().scroll_to(2000.0);
        assert_eq!(viewport_manager.scroll_state().vertical_offset(), 960.0); // Clamped to 1200-240=960
    }
}