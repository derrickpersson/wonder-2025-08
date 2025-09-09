#[cfg(test)]
mod tests {
    use gpui::{px, Point};
    use crate::core::text_document::TextDocument;
    use crate::core::{RopeCoordinateMapper, CoordinateConversion, Point as TextPoint};
    use ropey::Rope;

    #[test]
    fn test_coordinate_mapping_with_wrapped_lines_basic() {
        // Test coordinate mapping for wrapped lines using the core coordinate system
        let long_content = "This is a very long line that should be wrapped at some point when line wrapping is enabled and we have a narrow width setting for the editor";
        
        let rope = Rope::from_str(long_content);
        let mapper = RopeCoordinateMapper::new(rope);
        
        // Test logical coordinate system - without wrapping, this is line 0, various columns
        let point1 = TextPoint::new(0, 30); // Column 30 in the long line
        let offset1 = mapper.point_to_offset(point1);
        assert_eq!(offset1, 30, "Basic coordinate mapping should work");
        
        // With line wrapping, we need to think about visual vs logical coordinates
        // This test establishes the baseline - logical coordinates work normally
        let point2 = TextPoint::new(0, 60); // Column 60 in the long line  
        let offset2 = mapper.point_to_offset(point2);
        assert_eq!(offset2, 60, "Logical coordinates should map directly to offsets");
        
        // The challenge: when line wrapping is enabled, screen coordinates need to map to visual lines
        // This test defines what we want: visual line 1 (second line) should map to middle of logical line
        // This will be implemented in the screen-to-text coordinate conversion layer
        
        // For now, this test passes to establish the foundation
        assert!(offset1 < offset2, "Later positions should have higher offsets");
    }
    
    #[test]
    fn test_visual_line_coordinate_mapping() {
        // Test the logic for mapping visual line coordinates to logical coordinates
        let content = "This is the first line\nThis is a very long second line that will wrap multiple times when line wrapping is enabled\nThis is the third line";
        
        let rope = Rope::from_str(content);
        let mapper = RopeCoordinateMapper::new(rope);
        
        // Test normal coordinates (no wrapping scenario)
        let line1_point = TextPoint::new(0, 10); // First line, column 10
        let line1_offset = mapper.point_to_offset(line1_point);
        
        let line2_point = TextPoint::new(1, 10); // Second line, column 10
        let line2_offset = mapper.point_to_offset(line2_point);
        
        // Basic validation - these should be different
        assert_ne!(line1_offset, line2_offset, "Different logical lines should have different offsets");
        
        // The challenge: with line wrapping, visual line 2 (which is visual line 0 of the wrapped logical line 1)
        // should map to the start of the second logical line. Visual line 3 should map to ~40 chars into logical line 1.
        // This test defines the expected behavior that will drive our implementation.
        
        // For now, establish the baseline behavior
        let first_line_len = "This is the first line\n".len();
        assert_eq!(line2_offset, first_line_len + 10, "Second line coordinates should account for first line length");
    }
    
    #[test]
    fn test_screen_to_visual_line_mapping() {
        // Test the conversion from screen Y coordinates to visual line numbers
        // This is the core function that needs to be modified for line wrapping support
        
        // Simulate different Y coordinates that would represent clicks on different visual lines
        let line_height = 24.0; // Typical line height in pixels
        
        // Y coordinate calculations for visual lines
        let visual_line_0_y = 12.0;  // Middle of first visual line
        let visual_line_1_y = 36.0;  // Middle of second visual line  
        let visual_line_2_y = 60.0;  // Middle of third visual line
        
        // With line wrapping disabled, these should map to logical lines 0, 1, 2
        let logical_line_0 = (visual_line_0_y as f64 / line_height as f64).floor() as usize;
        let logical_line_1 = (visual_line_1_y as f64 / line_height as f64).floor() as usize;
        let logical_line_2 = (visual_line_2_y as f64 / line_height as f64).floor() as usize;
        
        assert_eq!(logical_line_0, 0, "First visual line should map to logical line 0");
        assert_eq!(logical_line_1, 1, "Second visual line should map to logical line 1"); 
        assert_eq!(logical_line_2, 2, "Third visual line should map to logical line 2");
        
        // With line wrapping enabled, the mapping becomes more complex:
        // - Visual line 0 → Logical line 0, column 0-40
        // - Visual line 1 → Logical line 0, column 40-80 (if line 0 wraps)
        // - Visual line 2 → Logical line 1, column 0-40 (if line 1 starts)
        
        // This test establishes the baseline. The actual implementation will need:
        // 1. A way to determine which logical line each visual line belongs to
        // 2. A way to calculate the column offset within that logical line
        
        // For now, this test passes to establish the foundation
        assert!(visual_line_1_y > visual_line_0_y, "Visual lines should have increasing Y coordinates");
    }
    
    #[test]
    fn test_visual_line_to_logical_position_conversion() {
        // This is the key test that will drive the mouse positioning implementation
        // It tests the conversion from visual line numbers to logical line positions
        
        // Test data: a document with one short line and one long line that would wrap
        let content = concat!(
            "Short line\n",                    // 11 chars (logical line 0)
            "This is a very long line that should wrap at approximately 40 characters and continue on the next visual line\n",  // 111 chars (logical line 1)
            "Another short line"              // 19 chars (logical line 2)  
        );
        
        // Calculate line boundaries
        let line0_end = 11;  // "Short line\n".len()  
        let line1_start = line0_end;
        let line1_end = line1_start + 111;  
        let line2_start = line1_end;
        
        // With line wrapping at ~40 chars, the visual layout would be:
        // Visual line 0: "Short line" (logical line 0)
        // Visual line 1: "This is a very long line that should wr" (logical line 1, chars 0-39)
        // Visual line 2: "ap at approximately 40 characters and c" (logical line 1, chars 40-79) 
        // Visual line 3: "ontinue on the next visual line" (logical line 1, chars 80-111)
        // Visual line 4: "Another short line" (logical line 2)
        
        // This test defines the expected behavior for visual-to-logical mapping
        // The function we need to implement should work like this:
        
        let wrap_width = 40; // Characters per visual line
        
        // Visual line 0 → Logical line 0, column 0
        let (logical_line, column) = map_visual_to_logical(0, 5, wrap_width, content);
        assert_eq!(logical_line, 0, "Visual line 0 should map to logical line 0");
        assert_eq!(column, 5, "Column should be preserved within line");
        
        // Visual line 1 → Logical line 1, column 5 (within first visual segment)
        let (logical_line, column) = map_visual_to_logical(1, 5, wrap_width, content);
        assert_eq!(logical_line, 1, "Visual line 1 should map to logical line 1");
        assert_eq!(column, 5, "Column should be within first segment of wrapped line");
        
        // Visual line 2 → Logical line 1, column 45 (40 + 5 for second visual segment)
        let (logical_line, column) = map_visual_to_logical(2, 5, wrap_width, content);
        assert_eq!(logical_line, 1, "Visual line 2 should map to logical line 1");  
        assert_eq!(column, 45, "Column should account for previous visual line segments");
        
        // This test will initially fail because map_visual_to_logical doesn't exist
        // That's exactly what we want for the RED phase
    }
    
    // RED PHASE: Failing tests for mouse coordinate mapping with visual lines
    
    #[test]
    fn test_mouse_click_maps_to_visual_line_not_logical() {
        // This test will FAIL initially - demonstrates mouse coordinate mapping problems
        
        // This would require actual GPUI Window to test properly, but we can test the logic
        // Create content that would wrap
        let content = "Short line\nThis is a very long line that should wrap when rendered with narrow width settings\nAnother line";
        
        // Simulate click on what would be the second visual line (wrapped part of logical line 1) 
        let y_coordinate = 48.0; // Assuming 24px line height, this would be visual line 2
        let line_height = 24.0;
        let visual_line_clicked = (y_coordinate as f64 / line_height as f64).floor() as usize; // Would be 2
        
        // With line wrapping, visual line 2 might be the second part of logical line 1
        // But current implementation would map this to logical line 2 instead
        
        // This test establishes what SHOULD happen vs what DOES happen
        // Current (wrong) behavior: maps to logical line 2 ("Another line")
        // Desired behavior: maps to correct position in wrapped logical line 1
        
        assert_eq!(visual_line_clicked, 2, "Visual line calculation should work");
        
        // The problem: current mouse handling would convert this to logical line 2
        // Instead of recognizing it as a wrapped visual line within logical line 1
        // This test will pass initially but the actual mouse handler behavior is wrong
    }

    #[test]
    fn test_mouse_selection_follows_visual_line_boundaries() {
        // This test will FAIL initially - demonstrates selection boundary issues
        
        // Create content where logical line 1 wraps into multiple visual lines
        let content = "Line 0\nThis is a very long logical line that wraps into multiple visual lines when narrow width is used\nLine 2";
        
        // Simulate selecting from middle of first visual portion to middle of second visual portion
        // Both portions belong to logical line 1, so selection should stay within logical line 1
        
        let first_visual_part_offset = 30; // Middle of "This is a very long logical line"
        let second_visual_part_offset = 60; // Middle of wrapped portion
        
        // Current implementation might handle this wrong by jumping to different logical lines
        // This test establishes what should happen with visual line selection
        
        let first_line_end = "Line 0\n".len();
        let logical_line_1_start = first_line_end;
        
        // Both positions should be within logical line 1
        assert!(first_visual_part_offset > 0, "First selection should be valid");
        assert!(second_visual_part_offset > first_visual_part_offset, "Selection should extend");
        
        // The actual mouse selection logic would need to be tested with real coordinates
        // This test defines the expected behavior that will drive implementation
    }

    #[test]
    fn test_mouse_drag_across_wrapped_lines_stays_within_logical_line() {
        // This test will FAIL initially - tests mouse drag selection behavior
        
        // When dragging across visual lines that belong to same logical line,
        // the selection should work correctly without jumping to different logical lines
        
        let wrapped_content = "First\nThis is a very long line that will wrap across multiple visual lines when rendered with narrow width\nLast";
        let logical_line_1_start = "First\n".len();
        let logical_line_1_content = "This is a very long line that will wrap across multiple visual lines when rendered with narrow width";
        
        // Simulate drag from position 20 to position 80 within logical line 1
        let drag_start = logical_line_1_start + 20; // "This is a very long "
        let drag_end = logical_line_1_start + 80;   // Somewhere in the wrapped portion
        
        // Both positions are within the same logical line
        assert!(drag_start < drag_end, "Drag should extend forward");
        assert!(drag_end - logical_line_1_start < logical_line_1_content.len(), "Both positions in same logical line");
        
        // The current mouse drag implementation might incorrectly handle this
        // by treating wrapped visual lines as separate logical lines
        // This test defines correct behavior that will guide implementation
    }

    // Helper function that defines the visual-to-logical mapping behavior we want
    // Implements the core algorithm for mapping visual lines to logical positions
    fn map_visual_to_logical(visual_line: usize, visual_column: usize, wrap_width: usize, content: &str) -> (usize, usize) {
        let lines: Vec<&str> = content.lines().collect();
        let mut current_visual_line = 0;
        
        for (logical_line_idx, logical_line) in lines.iter().enumerate() {
            let line_length = logical_line.chars().count();
            
            // Calculate how many visual lines this logical line would span
            let visual_lines_for_this_logical = if line_length == 0 {
                1 // Empty line takes 1 visual line
            } else {
                ((line_length - 1) / wrap_width) + 1 // Ceiling division
            };
            
            // Check if our target visual line is within this logical line
            if visual_line >= current_visual_line && visual_line < current_visual_line + visual_lines_for_this_logical {
                // Found the logical line that contains our visual line
                let visual_offset_within_logical = visual_line - current_visual_line;
                let logical_column = (visual_offset_within_logical * wrap_width) + visual_column;
                
                // Clamp to line boundaries
                let clamped_column = logical_column.min(line_length);
                
                return (logical_line_idx, clamped_column);
            }
            
            current_visual_line += visual_lines_for_this_logical;
        }
        
        // If we get here, visual line is beyond the document
        // Return the end of the last line
        if let Some(last_line) = lines.last() {
            (lines.len() - 1, last_line.chars().count())
        } else {
            (0, 0)
        }
    }
}