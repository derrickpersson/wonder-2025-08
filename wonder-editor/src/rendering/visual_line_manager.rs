use std::collections::{HashMap, HashSet};
use std::ops::Range;
use gpui::{px, Pixels};
use super::VisualLine;

/// Manages all visual lines for the document, providing efficient lookup and bounds calculation
#[derive(Debug, Clone)]
pub struct VisualLineManager {
    /// All visual lines for the document (flat array for efficient access)
    visual_lines: Vec<VisualLine>,
    
    /// Map from logical line index to range of visual line indices
    /// e.g., logical line 0 might map to visual lines 0..3 if it wraps into 3 visual lines
    logical_to_visual_map: HashMap<usize, Range<usize>>,
    
    /// Y positions for each visual line (from GPUI rendering)
    y_positions: Vec<f32>,
    
    /// Total number of logical lines processed
    logical_line_count: usize,
    
    /// Whether line wrapping is enabled
    wrapping_enabled: bool,
    
    /// Set of logical lines that need to be re-wrapped (dirty tracking)
    dirty_lines: HashSet<usize>,
    
    /// Document version for cache invalidation
    document_version: u64,
}

impl VisualLineManager {
    /// Create a new empty visual line manager
    pub fn new() -> Self {
        Self {
            visual_lines: Vec::new(),
            logical_to_visual_map: HashMap::new(),
            y_positions: Vec::new(),
            logical_line_count: 0,
            wrapping_enabled: true,
            dirty_lines: HashSet::new(),
            document_version: 0,
        }
    }
    
    /// Clear all visual lines and mappings
    pub fn clear(&mut self) {
        self.visual_lines.clear();
        self.logical_to_visual_map.clear();
        self.y_positions.clear();
        self.logical_line_count = 0;
        self.dirty_lines.clear();
        self.document_version = 0;
    }
    
    /// Add visual lines for a logical line
    pub fn add_visual_lines_for_logical(&mut self, logical_line: usize, visual_lines: Vec<VisualLine>) {
        let start_idx = self.visual_lines.len();
        let count = visual_lines.len();
        
        // Add to flat array
        self.visual_lines.extend(visual_lines);
        
        // Update mapping
        if count > 0 {
            self.logical_to_visual_map.insert(logical_line, start_idx..start_idx + count);
        }
        
        // Track logical line count
        self.logical_line_count = self.logical_line_count.max(logical_line + 1);
    }
    
    /// Get all visual lines
    pub fn all_visual_lines(&self) -> &[VisualLine] {
        &self.visual_lines
    }
    
    /// Get visual lines for a specific logical line
    pub fn get_visual_lines_for_logical(&self, logical_line: usize) -> Option<&[VisualLine]> {
        self.logical_to_visual_map
            .get(&logical_line)
            .and_then(|range| self.visual_lines.get(range.clone()))
    }
    
    /// Find the visual line containing a character position within a logical line
    pub fn find_visual_line_at_position(
        &self,
        logical_line: usize,
        position_in_line: usize,
    ) -> Option<(usize, &VisualLine)> {
        let range = self.logical_to_visual_map.get(&logical_line)?;
        
        for idx in range.clone() {
            if let Some(vl) = self.visual_lines.get(idx) {
                if position_in_line >= vl.start_offset && position_in_line <= vl.end_offset {
                    return Some((idx, vl));
                }
            }
        }
        
        // Check if cursor is at the end of the last visual line
        if let Some(last_idx) = range.clone().last() {
            if let Some(last_vl) = self.visual_lines.get(last_idx) {
                if position_in_line == last_vl.end_offset {
                    return Some((last_idx, last_vl));
                }
            }
        }
        
        None
    }
    
    /// Find visual lines that intersect with a selection range in a logical line
    pub fn find_visual_lines_in_selection(
        &self,
        logical_line: usize,
        sel_start: usize,
        sel_end: usize,
    ) -> Vec<(usize, &VisualLine)> {
        let mut result = Vec::new();
        
        if let Some(range) = self.logical_to_visual_map.get(&logical_line) {
            for idx in range.clone() {
                if let Some(vl) = self.visual_lines.get(idx) {
                    // Check if this visual line intersects with the selection
                    if sel_end > vl.start_offset && sel_start < vl.end_offset {
                        result.push((idx, vl));
                    }
                }
            }
        }
        
        result
    }
    
    /// Update Y positions from GPUI rendering
    pub fn update_y_positions(&mut self, positions: Vec<f32>) {
        self.y_positions = positions;
    }
    
    /// Get the Y position for a visual line
    pub fn get_y_position(&self, visual_line_index: usize) -> Option<f32> {
        self.y_positions.get(visual_line_index).copied()
    }
    
    /// Get the total document height including both rendered and non-rendered lines
    /// Due to viewport culling, we need to account for all lines, not just visible ones
    pub fn get_total_document_height(&self, line_height: f32) -> Option<f32> {
        if self.y_positions.is_empty() {
            return None;
        }
        
        // CRITICAL: Due to viewport culling, only visible lines are processed,
        // so we can only return height of rendered content, not total document.
        // The element layout phase needs to provide total logical line count.
        
        // For now, return the height of all rendered visual lines
        let total_rendered_visual_lines = self.y_positions.len();
        Some(total_rendered_visual_lines as f32 * line_height)
    }
    
    /// Calculate total document height given total logical line count
    /// This method should be used when we know the total logical lines in the document
    pub fn calculate_total_document_height(&self, total_logical_lines: usize, line_height: f32) -> f32 {
        // For documents with line wrapping, we need to estimate based on average lines per logical line
        let visual_lines_count = if self.logical_to_visual_map.is_empty() {
            // No line wrapping data available, assume 1:1 ratio
            total_logical_lines
        } else {
            // Calculate average visual lines per logical line from available data
            let total_visual_in_map: usize = self.logical_to_visual_map.values()
                .map(|range| range.end - range.start)
                .sum();
            let logical_lines_in_map = self.logical_to_visual_map.len();
            
            if logical_lines_in_map > 0 {
                let avg_visual_per_logical = total_visual_in_map as f32 / logical_lines_in_map as f32;
                (total_logical_lines as f32 * avg_visual_per_logical).round() as usize
            } else {
                total_logical_lines
            }
        };
        
        visual_lines_count as f32 * line_height
    }
    
    /// Calculate bounds for a visual line (X and Y position) with scroll offset
    pub fn get_visual_line_bounds(
        &self,
        visual_line_index: usize,
        bounds_origin: gpui::Point<Pixels>,
        padding: Pixels,
        line_height: Pixels,
        scroll_offset: f32,
    ) -> Option<gpui::Point<Pixels>> {
        // Get Y from stored positions (document-relative) and convert to screen coordinates
        let y = if let Some(y_pos) = self.get_y_position(visual_line_index) {
            // y_pos is document-relative, convert to screen coordinates with scroll
            bounds_origin.y + padding + px(y_pos - scroll_offset)
        } else {
            // Fallback calculation with scroll offset
            bounds_origin.y + padding + (line_height * visual_line_index as f32) - px(scroll_offset)
        };
        
        Some(gpui::point(bounds_origin.x + padding, y))
    }
    
    /// Get the total number of visual lines
    pub fn visual_line_count(&self) -> usize {
        self.visual_lines.len()
    }
    
    /// Get the number of visual lines for a logical line
    pub fn visual_lines_per_logical(&self, logical_line: usize) -> usize {
        self.logical_to_visual_map
            .get(&logical_line)
            .map(|range| range.len())
            .unwrap_or(1) // Default to 1 if not found
    }
    
    /// Check if a logical line has been processed
    pub fn has_logical_line(&self, logical_line: usize) -> bool {
        self.logical_to_visual_map.contains_key(&logical_line)
    }
    
    // === Dirty Line Tracking Methods ===
    
    /// Mark a logical line as dirty (needs re-wrapping)
    pub fn mark_line_dirty(&mut self, logical_line: usize) {
        self.dirty_lines.insert(logical_line);
    }
    
    /// Mark multiple logical lines as dirty
    pub fn mark_lines_dirty(&mut self, lines: &[usize]) {
        for &line in lines {
            self.dirty_lines.insert(line);
        }
    }
    
    /// Mark a range of logical lines as dirty (inclusive)
    pub fn mark_range_dirty(&mut self, start_line: usize, end_line: usize) {
        for line in start_line..=end_line {
            self.dirty_lines.insert(line);
        }
    }
    
    /// Check if a logical line is dirty
    pub fn is_line_dirty(&self, logical_line: usize) -> bool {
        self.dirty_lines.contains(&logical_line)
    }
    
    /// Get all dirty lines
    pub fn get_dirty_lines(&self) -> Vec<usize> {
        let mut dirty: Vec<usize> = self.dirty_lines.iter().copied().collect();
        dirty.sort();
        dirty
    }
    
    /// Clear dirty status for a specific line (after re-wrapping)
    pub fn clear_line_dirty(&mut self, logical_line: usize) {
        self.dirty_lines.remove(&logical_line);
    }
    
    /// Clear all dirty lines
    pub fn clear_all_dirty(&mut self) {
        self.dirty_lines.clear();
    }
    
    /// Get the count of dirty lines
    pub fn dirty_line_count(&self) -> usize {
        self.dirty_lines.len()
    }
    
    /// Update the document version and mark affected lines dirty
    pub fn update_document_version(&mut self, new_version: u64, affected_lines: &[usize]) {
        if new_version != self.document_version {
            self.document_version = new_version;
            self.mark_lines_dirty(affected_lines);
        }
    }
    
    /// Get the current document version
    pub fn document_version(&self) -> u64 {
        self.document_version
    }
    
    /// Invalidate visual lines for specific logical lines (removes them from cache)
    pub fn invalidate_lines(&mut self, logical_lines: &[usize]) {
        for &logical_line in logical_lines {
            // Mark as dirty for re-wrapping
            self.mark_line_dirty(logical_line);
            
            // Remove from visual lines if it exists
            if let Some(range) = self.logical_to_visual_map.remove(&logical_line) {
                // Note: We don't actually remove from visual_lines Vec to avoid index shifting
                // Instead, we just remove the mapping. The Vec will be rebuilt during next full update
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::StyledTextSegment;
    
    fn create_test_visual_line(logical: usize, start: usize, end: usize, index: usize) -> VisualLine {
        // Create a simple text run for testing
        let text_run = gpui::TextRun {
            len: 4, // "test".len()
            font: gpui::Font {
                family: "SF Pro".into(),
                features: gpui::FontFeatures::default(),
                weight: gpui::FontWeight::NORMAL,
                style: gpui::FontStyle::Normal,
                fallbacks: None,
            },
            color: gpui::rgb(0xffffff).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        
        VisualLine::new(
            logical,
            start,
            end,
            index,
            px(100.0),
            px(24.0),
            vec![StyledTextSegment {
                text: "test".to_string(),
                font_size: 16.0,
                text_run,
            }],
        )
    }
    
    #[test]
    fn test_visual_line_manager_basic() {
        let mut manager = VisualLineManager::new();
        
        // Add visual lines for logical line 0 (wraps into 2 visual lines)
        manager.add_visual_lines_for_logical(0, vec![
            create_test_visual_line(0, 0, 50, 0),
            create_test_visual_line(0, 50, 80, 1),
        ]);
        
        // Add single visual line for logical line 1
        manager.add_visual_lines_for_logical(1, vec![
            create_test_visual_line(1, 0, 30, 2),
        ]);
        
        assert_eq!(manager.visual_line_count(), 3);
        assert_eq!(manager.visual_lines_per_logical(0), 2);
        assert_eq!(manager.visual_lines_per_logical(1), 1);
    }
    
    #[test]
    fn test_find_visual_line_at_position() {
        let mut manager = VisualLineManager::new();
        
        manager.add_visual_lines_for_logical(0, vec![
            create_test_visual_line(0, 0, 50, 0),
            create_test_visual_line(0, 50, 80, 1),
        ]);
        
        // Position in first visual line
        let (idx, vl) = manager.find_visual_line_at_position(0, 25).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(vl.start_offset, 0);
        assert_eq!(vl.end_offset, 50);
        
        // Position in second visual line
        let (idx, vl) = manager.find_visual_line_at_position(0, 60).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(vl.start_offset, 50);
        assert_eq!(vl.end_offset, 80);
        
        // Position at end of last visual line
        let (idx, _) = manager.find_visual_line_at_position(0, 80).unwrap();
        assert_eq!(idx, 1);
    }
    
    #[test]
    fn test_find_visual_lines_in_selection() {
        let mut manager = VisualLineManager::new();
        
        manager.add_visual_lines_for_logical(0, vec![
            create_test_visual_line(0, 0, 50, 0),
            create_test_visual_line(0, 50, 100, 1),
            create_test_visual_line(0, 100, 150, 2),
        ]);
        
        // Selection spanning first two visual lines
        let lines = manager.find_visual_lines_in_selection(0, 25, 75);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, 0);
        assert_eq!(lines[1].0, 1);
        
        // Selection in single visual line
        let lines = manager.find_visual_lines_in_selection(0, 10, 40);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, 0);
        
        // Selection spanning all visual lines
        let lines = manager.find_visual_lines_in_selection(0, 0, 150);
        assert_eq!(lines.len(), 3);
    }
    
    #[test]
    fn test_dirty_line_tracking_basic() {
        let mut manager = VisualLineManager::new();
        
        // Initially no dirty lines
        assert_eq!(manager.dirty_line_count(), 0);
        assert!(!manager.is_line_dirty(0));
        assert!(manager.get_dirty_lines().is_empty());
        
        // Mark line as dirty
        manager.mark_line_dirty(0);
        assert_eq!(manager.dirty_line_count(), 1);
        assert!(manager.is_line_dirty(0));
        assert_eq!(manager.get_dirty_lines(), vec![0]);
        
        // Mark multiple lines dirty
        manager.mark_lines_dirty(&[2, 1, 3]);
        assert_eq!(manager.dirty_line_count(), 4);
        assert_eq!(manager.get_dirty_lines(), vec![0, 1, 2, 3]); // Should be sorted
        
        // Clear specific dirty line
        manager.clear_line_dirty(1);
        assert_eq!(manager.dirty_line_count(), 3);
        assert!(!manager.is_line_dirty(1));
        assert_eq!(manager.get_dirty_lines(), vec![0, 2, 3]);
        
        // Clear all dirty lines
        manager.clear_all_dirty();
        assert_eq!(manager.dirty_line_count(), 0);
        assert!(manager.get_dirty_lines().is_empty());
    }
    
    #[test]
    fn test_dirty_line_range_marking() {
        let mut manager = VisualLineManager::new();
        
        // Mark a range of lines dirty
        manager.mark_range_dirty(2, 5);
        assert_eq!(manager.dirty_line_count(), 4);
        assert_eq!(manager.get_dirty_lines(), vec![2, 3, 4, 5]);
        
        // Verify individual lines
        assert!(!manager.is_line_dirty(1));
        assert!(manager.is_line_dirty(2));
        assert!(manager.is_line_dirty(3));
        assert!(manager.is_line_dirty(4));
        assert!(manager.is_line_dirty(5));
        assert!(!manager.is_line_dirty(6));
    }
    
    #[test]
    fn test_document_version_tracking() {
        let mut manager = VisualLineManager::new();
        
        // Initial version
        assert_eq!(manager.document_version(), 0);
        
        // Update document version with affected lines
        manager.update_document_version(1, &[0, 2]);
        assert_eq!(manager.document_version(), 1);
        assert_eq!(manager.dirty_line_count(), 2);
        assert_eq!(manager.get_dirty_lines(), vec![0, 2]);
        
        // Same version should not mark lines dirty again
        manager.clear_all_dirty();
        manager.update_document_version(1, &[1, 3]);
        assert_eq!(manager.dirty_line_count(), 0); // No change since version is same
        
        // New version should mark lines dirty
        manager.update_document_version(2, &[1, 3]);
        assert_eq!(manager.document_version(), 2);
        assert_eq!(manager.dirty_line_count(), 2);
        assert_eq!(manager.get_dirty_lines(), vec![1, 3]);
    }
    
    #[test]
    fn test_invalidate_lines() {
        let mut manager = VisualLineManager::new();
        
        // Add some visual lines first
        manager.add_visual_lines_for_logical(0, vec![
            create_test_visual_line(0, 0, 50, 0),
        ]);
        manager.add_visual_lines_for_logical(1, vec![
            create_test_visual_line(1, 0, 30, 1),
        ]);
        
        assert!(manager.has_logical_line(0));
        assert!(manager.has_logical_line(1));
        assert_eq!(manager.dirty_line_count(), 0);
        
        // Invalidate specific lines
        manager.invalidate_lines(&[0]);
        assert!(!manager.has_logical_line(0)); // Mapping should be removed
        assert!(manager.has_logical_line(1)); // Other line should remain
        assert_eq!(manager.dirty_line_count(), 1);
        assert!(manager.is_line_dirty(0));
    }
    
    #[test]
    fn test_clear_preserves_dirty_tracking_state() {
        let mut manager = VisualLineManager::new();
        
        // Set up some state
        manager.mark_lines_dirty(&[1, 2, 3]);
        manager.update_document_version(5, &[]);
        
        // Clear should reset everything
        manager.clear();
        assert_eq!(manager.dirty_line_count(), 0);
        assert_eq!(manager.document_version(), 0);
        assert!(manager.get_dirty_lines().is_empty());
    }
}