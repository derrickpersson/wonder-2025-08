use std::collections::HashMap;
use gpui::{px, Pixels, Window};
use crate::rendering::{StyledTextSegment, TextRunGenerator};

/// Represents a single visual line after wrapping a logical line
#[derive(Debug, Clone)]
pub struct VisualLine {
    /// The original logical line index this visual line comes from
    pub logical_line: usize,
    /// Start character offset within the logical line
    pub start_offset: usize,
    /// End character offset within the logical line (exclusive)
    pub end_offset: usize,
    /// Visual line index (0-based from start of document)
    pub visual_index: usize,
    /// Measured width of this visual line in pixels
    pub width: Pixels,
    /// Height of this visual line in pixels (accounts for different font sizes)
    pub height: Pixels,
    /// The styled text segments for this visual line
    pub segments: Vec<StyledTextSegment>,
}

impl VisualLine {
    pub fn new(
        logical_line: usize, 
        start_offset: usize, 
        end_offset: usize, 
        visual_index: usize,
        width: Pixels,
        height: Pixels,
        segments: Vec<StyledTextSegment>
    ) -> Self {
        Self {
            logical_line,
            start_offset,
            end_offset,
            visual_index,
            width,
            height,
            segments,
        }
    }
    
    /// Get the text content of this visual line
    pub fn text(&self) -> String {
        self.segments.iter().map(|s| s.text.as_str()).collect::<String>()
    }
    
    /// Get the length of this visual line in characters
    pub fn len(&self) -> usize {
        self.end_offset - self.start_offset
    }
}

/// Information about where a line was wrapped
#[derive(Debug, Clone)]
pub struct WrapPoint {
    /// Character position within the logical line where wrap occurs
    pub position: usize,
    /// Whether this is a word boundary wrap (preferred) or character wrap (forced)
    pub is_word_boundary: bool,
}

/// Maps between logical line positions and visual line positions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualPosition {
    /// Visual line index (0-based from start of document)  
    pub visual_line: usize,
    /// Column position within the visual line
    pub column: usize,
}

impl VisualPosition {
    pub fn new(visual_line: usize, column: usize) -> Self {
        Self { visual_line, column }
    }
}

/// Hybrid line wrapper that handles wrapping with markdown styling
#[derive(Clone)]
pub struct HybridLineWrapper {
    /// Width at which to wrap lines (in pixels)
    wrap_width: Pixels,
    /// Cache of wrapped lines by logical line index with version tracking
    /// Map: logical_line_index -> (document_version, visual_lines)
    line_cache: HashMap<usize, (u64, Vec<VisualLine>)>,
    /// Text run generator for styled segments
    text_run_generator: TextRunGenerator,
    /// Whether line wrapping is enabled
    enabled: bool,
}

impl HybridLineWrapper {
    pub fn new(wrap_width: Pixels) -> Self {
        Self {
            wrap_width,
            line_cache: HashMap::new(),
            text_run_generator: TextRunGenerator::new(),
            enabled: true,
        }
    }
    
    /// Create a line wrapper with wrapping disabled
    pub fn disabled() -> Self {
        Self {
            wrap_width: px(0.0),
            line_cache: HashMap::new(),
            text_run_generator: TextRunGenerator::new(),
            enabled: false,
        }
    }
    
    /// Enable or disable line wrapping
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            self.enabled = enabled;
            self.invalidate_cache();
        }
    }
    
    /// Update the wrap width and invalidate cache if changed
    pub fn set_wrap_width(&mut self, wrap_width: Pixels) {
        if (self.wrap_width.0 - wrap_width.0).abs() > 0.1 {
            self.wrap_width = wrap_width;
            self.invalidate_cache();
        }
    }
    
    /// Clear the cache (call when text changes)
    pub fn invalidate_cache(&mut self) {
        self.line_cache.clear();
    }
    
    /// Check if line wrapping is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get the current wrap width
    pub fn wrap_width(&self) -> Pixels {
        self.wrap_width
    }
    
    /// Wrap a logical line into visual lines
    pub fn wrap_line(
        &mut self,
        logical_line_index: usize,
        line_content: &str,
        cursor_position: usize,
        selection: Option<std::ops::Range<usize>>,
        document_version: u64,
        window: &mut Window,
    ) -> Vec<VisualLine> {
        // Debug: Basic wrapping info (can be disabled later)
        
        // If wrapping is disabled, return single visual line
        if !self.enabled || self.wrap_width.0 <= 0.0 {
                return vec![self.create_single_visual_line(
                logical_line_index, 
                line_content, 
                cursor_position, 
                selection, 
                window
            )];
        }
        
        // Check cache first - only use if version matches
        if let Some((cached_version, cached_lines)) = self.line_cache.get(&logical_line_index) {
            if *cached_version == document_version {
                return cached_lines.clone();
            }
            // else: cache is stale, will recalculate
        }
        
        // Generate styled segments for this line using hybrid renderer
        let segments = self.text_run_generator.generate_styled_text_segments(
            line_content,
            cursor_position,
            selection,
        );
        
        
        // If no segments, return empty visual line
        if segments.is_empty() {
            let visual_line = VisualLine::new(
                logical_line_index, 0, 0, 0, px(0.0), px(24.0), vec![]
            );
            return vec![visual_line];
        }
        
        // Wrap the segments into visual lines
        let visual_lines = self.wrap_segments_to_visual_lines(
            logical_line_index,
            line_content,
            segments,
            window,
        );
        
        
        // Cache the result with version tracking
        self.line_cache.insert(logical_line_index, (document_version, visual_lines.clone()));
        
        visual_lines
    }
    
    /// Create a single visual line without wrapping (for disabled mode)
    fn create_single_visual_line(
        &self,
        logical_line_index: usize,
        line_content: &str,
        cursor_position: usize,
        selection: Option<std::ops::Range<usize>>,
        window: &mut Window,
    ) -> VisualLine {
        let segments = self.text_run_generator.generate_styled_text_segments(
            line_content,
            cursor_position, 
            selection,
        );
        
        let width = self.measure_segments_width(&segments, window);
        let height = self.calculate_line_height(&segments);
        
        VisualLine::new(
            logical_line_index,
            0,
            line_content.chars().count(),
            0,
            width,
            height,
            segments,
        )
    }
    
    /// Wrap styled segments into visual lines at word boundaries when possible
    fn wrap_segments_to_visual_lines(
        &self,
        logical_line_index: usize,
        _line_content: &str,
        segments: Vec<StyledTextSegment>,
        window: &mut Window,
    ) -> Vec<VisualLine> {
        
        let mut visual_lines = Vec::new();
        let mut current_line_segments = Vec::new();
        let mut current_line_width = px(0.0);
        let mut logical_char_position = 0;
        let mut visual_line_start_pos = 0;
        let mut visual_index = 0;
        
        for (_seg_idx, segment) in segments.into_iter().enumerate() {
            let segment_width = self.measure_segment_width(&segment, window);
            
            
            // Check if this segment is too wide to fit even on an empty line
            if segment_width.0 > self.wrap_width.0 {
                
                // First, finish current line if it has content
                if !current_line_segments.is_empty() {
                    let line_height = self.calculate_segments_height(&current_line_segments);
                    let visual_line = VisualLine::new(
                        logical_line_index,
                        visual_line_start_pos,
                        logical_char_position,
                        visual_index,
                        current_line_width,
                        line_height,
                        current_line_segments.clone(),
                    );
                    visual_lines.push(visual_line);
                    
                    // Start new line
                    current_line_segments.clear();
                    current_line_width = px(0.0);
                    visual_line_start_pos = logical_char_position;
                    visual_index += 1;
                }
                
                // Split the segment at word boundaries
                let wrapped_segments = self.wrap_segment_at_words(&segment, window);
                
                for wrapped_segment in wrapped_segments {
                    let wrapped_width = self.measure_segment_width(&wrapped_segment, window);
                    
                    // Check if we need to wrap before adding this part
                    if current_line_width.0 + wrapped_width.0 > self.wrap_width.0 && !current_line_segments.is_empty() {
                        // Finish current line
                        let line_height = self.calculate_segments_height(&current_line_segments);
                        let visual_line = VisualLine::new(
                            logical_line_index,
                            visual_line_start_pos,
                            logical_char_position,
                            visual_index,
                            current_line_width,
                            line_height,
                            current_line_segments.clone(),
                        );
                        visual_lines.push(visual_line);
                        
                        // Start new line
                        current_line_segments.clear();
                        current_line_width = px(0.0);
                        visual_line_start_pos = logical_char_position;
                        visual_index += 1;
                    }
                    
                    // Add wrapped segment to current line
                    current_line_segments.push(wrapped_segment.clone());
                    current_line_width = px(current_line_width.0 + wrapped_width.0);
                    logical_char_position += wrapped_segment.text.chars().count();
                }
            } else {
                // Check if adding this segment would exceed wrap width
                if current_line_width.0 + segment_width.0 > self.wrap_width.0 && !current_line_segments.is_empty() {
                    // Wrap here - finish current line
                    let line_height = self.calculate_segments_height(&current_line_segments);
                    let visual_line = VisualLine::new(
                        logical_line_index,
                        visual_line_start_pos,
                        logical_char_position,
                        visual_index,
                        current_line_width,
                        line_height,
                        current_line_segments.clone(),
                    );
                    visual_lines.push(visual_line);
                    
                    // Start new line
                    current_line_segments.clear();
                    current_line_width = px(0.0);
                    visual_line_start_pos = logical_char_position;
                    visual_index += 1;
                }
                
                // Add segment to current line
                current_line_segments.push(segment.clone());
                current_line_width = px(current_line_width.0 + segment_width.0);
                logical_char_position += segment.text.chars().count();
            }
            
        }
        
        // Add final line if there are remaining segments
        if !current_line_segments.is_empty() {
            let line_height = self.calculate_segments_height(&current_line_segments);
            let visual_line = VisualLine::new(
                logical_line_index,
                visual_line_start_pos,
                logical_char_position,
                visual_index,
                current_line_width,
                line_height,
                current_line_segments,
            );
            visual_lines.push(visual_line);
        }
        
        // If no visual lines were created, create one empty line
        if visual_lines.is_empty() {
            visual_lines.push(VisualLine::new(
                logical_line_index,
                0,
                0,
                0,
                px(0.0),
                px(24.0),
                vec![],
            ));
        }
        
        visual_lines
    }
    
    /// Split a segment at word boundaries to fit within the wrap width
    fn wrap_segment_at_words(
        &self,
        segment: &StyledTextSegment,
        window: &mut Window,
    ) -> Vec<StyledTextSegment> {
        let text = &segment.text;
        let mut result = Vec::new();
        
        // Split text into words (preserve whitespace)
        let words: Vec<&str> = text.split_inclusive(' ').collect();
        let mut current_line = String::new();
        
        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                current_line.clone() + word
            };
            
            // Create a test segment to measure width
            let test_segment = StyledTextSegment {
                text: test_line.clone(),
                text_run: segment.text_run.clone(),
                font_size: segment.font_size,
            };
            
            let test_width = self.measure_segment_width(&test_segment, window);
            
            if test_width.0 <= self.wrap_width.0 {
                // This word fits, add it to current line
                current_line = test_line;
            } else {
                // This word doesn't fit
                if !current_line.is_empty() {
                    // Save current line and start new one with this word
                    let line_segment = StyledTextSegment {
                        text: current_line,
                        text_run: segment.text_run.clone(),
                        font_size: segment.font_size,
                    };
                    result.push(line_segment);
                    current_line = word.to_string();
                } else {
                    // Even a single word doesn't fit - just use it anyway (avoid infinite loop)
                    current_line = word.to_string();
                }
            }
        }
        
        // Add any remaining text
        if !current_line.is_empty() {
            let final_segment = StyledTextSegment {
                text: current_line,
                text_run: segment.text_run.clone(),
                font_size: segment.font_size,
            };
            result.push(final_segment);
        }
        
        // If no segments created, return original segment (avoid infinite loop)
        if result.is_empty() {
            result.push(segment.clone());
        }
        
        result
    }
    
    /// Measure the width of a styled text segment using GPUI
    fn measure_segment_width(&self, segment: &StyledTextSegment, window: &mut Window) -> Pixels {
        if segment.text.is_empty() {
            return px(0.0);
        }
        
        let shaped_line = window.text_system().shape_line(
            segment.text.clone().into(),
            px(segment.font_size),
            &[segment.text_run.clone()],
            None,
        );
        
        shaped_line.width
    }
    
    /// Measure the total width of multiple segments
    fn measure_segments_width(&self, segments: &[StyledTextSegment], window: &mut Window) -> Pixels {
        let mut total_width = 0.0;
        for segment in segments {
            total_width += self.measure_segment_width(segment, window).0;
        }
        px(total_width)
    }
    
    /// Calculate the height of a line based on the tallest segment
    fn calculate_line_height(&self, segments: &[StyledTextSegment]) -> Pixels {
        if segments.is_empty() {
            return px(24.0); // Default line height
        }
        
        let max_font_size = segments
            .iter()
            .map(|s| s.font_size)
            .fold(16.0, f32::max);
            
        // Line height is typically 1.5x font size
        px(max_font_size * 1.5)
    }
    
    /// Calculate the height of multiple segments (same as single line for now)
    fn calculate_segments_height(&self, segments: &[StyledTextSegment]) -> Pixels {
        self.calculate_line_height(segments)
    }
    
    /// Convert logical position to visual position
    pub fn logical_to_visual_position(
        &self,
        logical_line: usize, 
        logical_column: usize
    ) -> Option<VisualPosition> {
        // If wrapping is disabled, it's a 1:1 mapping
        if !self.enabled {
            return Some(VisualPosition::new(logical_line, logical_column));
        }
        
        if let Some((_version, visual_lines)) = self.line_cache.get(&logical_line) {
            for visual_line in visual_lines {
                if logical_column >= visual_line.start_offset && logical_column <= visual_line.end_offset {
                    let column_in_visual = logical_column - visual_line.start_offset;
                    return Some(VisualPosition::new(visual_line.visual_index, column_in_visual));
                }
            }
        }
        
        None
    }
    
    /// Convert visual position to logical position  
    pub fn visual_to_logical_position(
        &self,
        visual_line: usize,
        visual_column: usize
    ) -> Option<(usize, usize)> {
        // If wrapping is disabled, it's a 1:1 mapping
        if !self.enabled {
            return Some((visual_line, visual_column));
        }
        
        // Find the visual line in our cache
        for (logical_line_idx, (_version, visual_lines)) in &self.line_cache {
            for vline in visual_lines {
                if vline.visual_index == visual_line {
                    let logical_column = vline.start_offset + visual_column.min(vline.len());
                    return Some((*logical_line_idx, logical_column));
                }
            }
        }
        
        None
    }
    
    /// Get all visual lines for the document (requires all lines to be processed first)
    pub fn get_all_visual_lines(&self) -> Vec<VisualLine> {
        let mut all_visual_lines = Vec::new();
        let mut sorted_logical_lines: Vec<_> = self.line_cache.iter().collect();
        sorted_logical_lines.sort_by_key(|(logical_idx, _)| **logical_idx);
        
        for (_, (_, visual_lines)) in sorted_logical_lines {
            all_visual_lines.extend_from_slice(visual_lines);
        }
        
        all_visual_lines
    }
    
    /// Get cached visual lines for a specific logical line
    pub fn get_cached_visual_lines(&self, logical_line: usize) -> Option<Vec<VisualLine>> {
        self.line_cache.get(&logical_line).map(|(_, visual_lines)| visual_lines.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn test_hybrid_line_wrapper_creation() {
        let wrapper = HybridLineWrapper::new(px(400.0));
        assert!(wrapper.enabled);
        assert_eq!(wrapper.wrap_width.0, 400.0);
    }
    
    #[test] 
    fn test_hybrid_line_wrapper_disabled() {
        let wrapper = HybridLineWrapper::disabled();
        assert!(!wrapper.enabled);
        assert_eq!(wrapper.wrap_width.0, 0.0);
    }
    
    #[test]
    fn test_visual_line_creation() {
        let segments = vec![];
        let visual_line = VisualLine::new(0, 0, 10, 0, px(100.0), px(24.0), segments);
        
        assert_eq!(visual_line.logical_line, 0);
        assert_eq!(visual_line.start_offset, 0);
        assert_eq!(visual_line.end_offset, 10);
        assert_eq!(visual_line.len(), 10);
        assert_eq!(visual_line.width.0, 100.0);
    }
    
    #[test]
    fn test_wrap_width_update_invalidates_cache() {
        let mut wrapper = HybridLineWrapper::new(px(400.0));
        
        // Add something to cache (simulate)
        wrapper.line_cache.insert(0, (1, vec![]));
        assert!(!wrapper.line_cache.is_empty());
        
        // Update wrap width should clear cache
        wrapper.set_wrap_width(px(500.0));
        assert!(wrapper.line_cache.is_empty());
    }
    
    #[test] 
    fn test_enable_disable_invalidates_cache() {
        let mut wrapper = HybridLineWrapper::new(px(400.0));
        
        // Add something to cache (simulate)
        wrapper.line_cache.insert(0, (1, vec![]));
        assert!(!wrapper.line_cache.is_empty());
        
        // Disable should clear cache
        wrapper.set_enabled(false);
        assert!(wrapper.line_cache.is_empty());
        assert!(!wrapper.enabled);
    }

    #[test]
    fn test_logical_to_visual_position_disabled() {
        let wrapper = HybridLineWrapper::disabled();
        
        let visual_pos = wrapper.logical_to_visual_position(2, 5);
        assert_eq!(visual_pos, Some(VisualPosition::new(2, 5)));
    }
    
    #[test]
    fn test_visual_to_logical_position_disabled() {
        let wrapper = HybridLineWrapper::disabled();
        
        let logical_pos = wrapper.visual_to_logical_position(3, 7);
        assert_eq!(logical_pos, Some((3, 7)));
    }

    // This test will fail initially (RED phase) - we need to implement position mapping
    #[test]
    fn test_logical_to_visual_with_cached_wrapping() {
        let mut wrapper = HybridLineWrapper::new(px(100.0));
        
        // Simulate cached wrapped lines for logical line 0
        let visual_lines = vec![
            VisualLine::new(0, 0, 10, 0, px(90.0), px(24.0), vec![]),
            VisualLine::new(0, 10, 20, 1, px(80.0), px(24.0), vec![]),
        ];
        wrapper.line_cache.insert(0, (1, visual_lines));
        
        // Test position mapping
        let visual_pos = wrapper.logical_to_visual_position(0, 5);
        assert_eq!(visual_pos, Some(VisualPosition::new(0, 5))); // First visual line, column 5
        
        let visual_pos = wrapper.logical_to_visual_position(0, 15);
        assert_eq!(visual_pos, Some(VisualPosition::new(1, 5))); // Second visual line, column 5
    }
    
    // This test will fail initially (RED phase) - we need to implement reverse mapping
    #[test]
    fn test_visual_to_logical_with_cached_wrapping() {
        let mut wrapper = HybridLineWrapper::new(px(100.0));
        
        // Simulate cached wrapped lines for logical line 0
        let visual_lines = vec![
            VisualLine::new(0, 0, 10, 0, px(90.0), px(24.0), vec![]),
            VisualLine::new(0, 10, 20, 1, px(80.0), px(24.0), vec![]),
        ];
        wrapper.line_cache.insert(0, (1, visual_lines));
        
        // Test reverse mapping
        let logical_pos = wrapper.visual_to_logical_position(0, 5);
        assert_eq!(logical_pos, Some((0, 5))); // Logical line 0, column 5
        
        let logical_pos = wrapper.visual_to_logical_position(1, 5);
        assert_eq!(logical_pos, Some((0, 15))); // Logical line 0, column 15
    }
}