//! Coordinate mapping utilities for converting between different position representations
//! 
//! This module provides utilities for converting between:
//! - Point (row, column) coordinates
//! - Offset (absolute character position) coordinates  
//! - Screen (pixel) coordinates
//! 
//! This is essential for accurate cursor positioning and mouse interaction.

use crate::core::Point;
use ropey::Rope;
use std::ops::Range;

/// Screen position in pixels
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPosition {
    pub x: f32,
    pub y: f32,
}

impl ScreenPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Trait for converting between coordinate systems
pub trait CoordinateConversion {
    /// Convert a Point to an absolute offset in the document
    fn point_to_offset(&self, point: Point) -> usize;
    
    /// Convert an absolute offset to a Point
    fn offset_to_point(&self, offset: usize) -> Point;
    
    /// Convert screen coordinates to a Point (requires layout information)
    fn screen_to_point(&self, screen_pos: ScreenPosition) -> Point;
    
    /// Convert a Point to screen coordinates (requires layout information)
    fn point_to_screen(&self, point: Point) -> ScreenPosition;
    
    /// Get the maximum valid point in the document
    fn max_point(&self) -> Point;
    
    /// Clamp a point to valid bounds
    fn clamp_point(&self, point: Point) -> Point;
}

/// Rope-based coordinate conversion implementation
/// 
/// This provides efficient coordinate conversions using the rope data structure,
/// leveraging its O(log n) performance characteristics.
pub struct RopeCoordinateMapper {
    rope: Rope,
}

impl RopeCoordinateMapper {
    /// Create a new coordinate mapper for the given rope
    pub fn new(rope: Rope) -> Self {
        Self { rope }
    }
    
    /// Update the rope for coordinate mapping
    pub fn update_rope(&mut self, rope: Rope) {
        self.rope = rope;
    }
    
    /// Get the rope reference
    pub fn rope(&self) -> &Rope {
        &self.rope
    }
}

impl CoordinateConversion for RopeCoordinateMapper {
    fn point_to_offset(&self, point: Point) -> usize {
        // Use rope's efficient line-to-char conversion
        if point.row == 0 {
            // Fast path for first line
            (point.column as usize).min(self.rope.len_chars())
        } else {
            // Find the line start offset
            let line_start_offset = if (point.row as usize) < self.rope.len_lines() {
                self.rope.line_to_char(point.row as usize)
            } else {
                // Point is beyond document end
                return self.rope.len_chars();
            };
            
            // Add column offset, clamped to line length
            let line_end_offset = if (point.row as usize + 1) < self.rope.len_lines() {
                self.rope.line_to_char(point.row as usize + 1)
            } else {
                self.rope.len_chars()
            };
            
            let max_column = line_end_offset.saturating_sub(line_start_offset);
            let column_offset = (point.column as usize).min(max_column);
            
            line_start_offset + column_offset
        }
    }
    
    fn offset_to_point(&self, offset: usize) -> Point {
        let clamped_offset = offset.min(self.rope.len_chars());
        
        if clamped_offset == 0 {
            return Point::zero();
        }
        
        // Use rope's efficient char-to-line conversion
        let line = self.rope.char_to_line(clamped_offset);
        let line_start_offset = self.rope.line_to_char(line);
        let column = clamped_offset - line_start_offset;
        
        Point::new(line as u32, column as u32)
    }
    
    fn screen_to_point(&self, _screen_pos: ScreenPosition) -> Point {
        // This requires font metrics and layout information
        // For now, return a placeholder - this will be implemented
        // when we integrate with the hybrid renderer
        Point::zero()
    }
    
    fn point_to_screen(&self, _point: Point) -> ScreenPosition {
        // This requires font metrics and layout information  
        // For now, return a placeholder - this will be implemented
        // when we integrate with the hybrid renderer
        ScreenPosition::zero()
    }
    
    fn max_point(&self) -> Point {
        if self.rope.len_chars() == 0 {
            return Point::zero();
        }
        
        let last_line = self.rope.len_lines().saturating_sub(1);
        let last_line_start = self.rope.line_to_char(last_line);
        let last_line_len = self.rope.len_chars() - last_line_start;
        
        Point::new(last_line as u32, last_line_len as u32)
    }
    
    fn clamp_point(&self, point: Point) -> Point {
        let max_point = self.max_point();
        
        if point.row > max_point.row {
            // Point is beyond last line
            max_point
        } else if point.row == max_point.row {
            // Same line - clamp column
            Point::new(point.row, point.column.min(max_point.column))
        } else {
            // Point is on a valid line - clamp column to line length
            let line_start_offset = self.rope.line_to_char(point.row as usize);
            let line_end_offset = if (point.row as usize + 1) < self.rope.len_lines() {
                self.rope.line_to_char(point.row as usize + 1)
            } else {
                self.rope.len_chars()
            };
            
            // Calculate line length excluding the newline character
            let raw_line_length = line_end_offset.saturating_sub(line_start_offset);
            let max_column = if (point.row as usize + 1) < self.rope.len_lines() {
                // Not the last line, so exclude newline character
                raw_line_length.saturating_sub(1) as u32
            } else {
                // Last line, no newline to exclude
                raw_line_length as u32
            };
            
            Point::new(point.row, point.column.min(max_column))
        }
    }
}

/// Utility functions for range conversions
pub trait PointRangeExt {
    /// Convert a Point range to an offset range
    fn to_offset_range(&self, mapper: &impl CoordinateConversion) -> Range<usize>;
}

impl PointRangeExt for Range<Point> {
    fn to_offset_range(&self, mapper: &impl CoordinateConversion) -> Range<usize> {
        let start_offset = mapper.point_to_offset(self.start);
        let end_offset = mapper.point_to_offset(self.end);
        start_offset..end_offset
    }
}

/// Utility functions for offset range conversions
pub trait OffsetRangeExt {
    /// Convert an offset range to a Point range
    fn to_point_range(&self, mapper: &impl CoordinateConversion) -> Range<Point>;
}

impl OffsetRangeExt for Range<usize> {
    fn to_point_range(&self, mapper: &impl CoordinateConversion) -> Range<Point> {
        let start_point = mapper.offset_to_point(self.start);
        let end_point = mapper.offset_to_point(self.end);
        start_point..end_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    fn create_test_rope(content: &str) -> Rope {
        Rope::from_str(content)
    }

    #[test]
    fn test_screen_position_creation() {
        let pos = ScreenPosition::new(10.5, 20.3);
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, 20.3);
        
        let zero = ScreenPosition::zero();
        assert_eq!(zero.x, 0.0);
        assert_eq!(zero.y, 0.0);
    }

    #[test]
    fn test_rope_coordinate_mapper_creation() {
        let rope = create_test_rope("hello world");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.rope().to_string(), "hello world");
    }

    #[test]
    fn test_rope_coordinate_mapper_update() {
        let rope1 = create_test_rope("hello");
        let mut mapper = RopeCoordinateMapper::new(rope1);
        
        let rope2 = create_test_rope("world");
        mapper.update_rope(rope2);
        
        assert_eq!(mapper.rope().to_string(), "world");
    }

    #[test]
    fn test_point_to_offset_single_line() {
        let rope = create_test_rope("hello world");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.point_to_offset(Point::new(0, 0)), 0);
        assert_eq!(mapper.point_to_offset(Point::new(0, 5)), 5);
        assert_eq!(mapper.point_to_offset(Point::new(0, 11)), 11);
        
        // Beyond line end should clamp
        assert_eq!(mapper.point_to_offset(Point::new(0, 20)), 11);
    }

    #[test]
    fn test_point_to_offset_multi_line() {
        let rope = create_test_rope("hello\nworld\ntest");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.point_to_offset(Point::new(0, 0)), 0);
        assert_eq!(mapper.point_to_offset(Point::new(0, 5)), 5);
        assert_eq!(mapper.point_to_offset(Point::new(1, 0)), 6); // Start of "world"
        assert_eq!(mapper.point_to_offset(Point::new(1, 5)), 11); // End of "world"
        assert_eq!(mapper.point_to_offset(Point::new(2, 0)), 12); // Start of "test"
        assert_eq!(mapper.point_to_offset(Point::new(2, 4)), 16); // End of "test"
    }

    #[test]
    fn test_offset_to_point_single_line() {
        let rope = create_test_rope("hello world");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.offset_to_point(0), Point::new(0, 0));
        assert_eq!(mapper.offset_to_point(5), Point::new(0, 5));
        assert_eq!(mapper.offset_to_point(11), Point::new(0, 11));
        
        // Beyond document end should clamp
        assert_eq!(mapper.offset_to_point(20), Point::new(0, 11));
    }

    #[test]
    fn test_offset_to_point_multi_line() {
        let rope = create_test_rope("hello\nworld\ntest");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.offset_to_point(0), Point::new(0, 0));
        assert_eq!(mapper.offset_to_point(5), Point::new(0, 5)); // End of "hello"
        assert_eq!(mapper.offset_to_point(6), Point::new(1, 0)); // Start of "world"
        assert_eq!(mapper.offset_to_point(11), Point::new(1, 5)); // End of "world"  
        assert_eq!(mapper.offset_to_point(12), Point::new(2, 0)); // Start of "test"
        assert_eq!(mapper.offset_to_point(16), Point::new(2, 4)); // End of "test"
    }

    #[test]
    fn test_round_trip_conversion() {
        let rope = create_test_rope("hello\nworld\ntest\nmore lines");
        let mapper = RopeCoordinateMapper::new(rope);
        
        // Test round trip for various points
        let test_points = vec![
            Point::new(0, 0),
            Point::new(0, 3),
            Point::new(1, 2),
            Point::new(2, 4),
            Point::new(3, 10),
        ];
        
        for point in test_points {
            let offset = mapper.point_to_offset(point);
            let converted_back = mapper.offset_to_point(offset);
            assert_eq!(point, converted_back, "Round trip failed for {:?}", point);
        }
    }

    #[test]
    fn test_max_point_single_line() {
        let rope = create_test_rope("hello");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.max_point(), Point::new(0, 5));
    }

    #[test]
    fn test_max_point_multi_line() {
        let rope = create_test_rope("hello\nworld\ntest");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.max_point(), Point::new(2, 4));
    }

    #[test]
    fn test_max_point_empty_document() {
        let rope = create_test_rope("");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.max_point(), Point::new(0, 0));
    }

    #[test]
    fn test_clamp_point_within_bounds() {
        let rope = create_test_rope("hello\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        let point = Point::new(0, 3);
        assert_eq!(mapper.clamp_point(point), point); // Should be unchanged
    }

    #[test]
    fn test_clamp_point_beyond_line_end() {
        let rope = create_test_rope("hello\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        let point = Point::new(0, 10); // Beyond "hello"
        assert_eq!(mapper.clamp_point(point), Point::new(0, 5));
    }

    #[test]
    fn test_clamp_point_beyond_document_end() {
        let rope = create_test_rope("hello\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        let point = Point::new(5, 10); // Beyond last line
        assert_eq!(mapper.clamp_point(point), Point::new(1, 5)); // End of "world"
    }

    #[test]
    fn test_point_range_to_offset_range() {
        let rope = create_test_rope("hello\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        let point_range = Point::new(0, 1)..Point::new(1, 3);
        let offset_range = point_range.to_offset_range(&mapper);
        
        assert_eq!(offset_range, 1..9); // "ello\nwor"
    }

    #[test]
    fn test_offset_range_to_point_range() {
        let rope = create_test_rope("hello\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        let offset_range = 1..9; // "ello\nwor"
        let point_range = offset_range.to_point_range(&mapper);
        
        assert_eq!(point_range, Point::new(0, 1)..Point::new(1, 3));
    }

    #[test]
    fn test_empty_document_conversions() {
        let rope = create_test_rope("");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.point_to_offset(Point::zero()), 0);
        assert_eq!(mapper.offset_to_point(0), Point::zero());
        assert_eq!(mapper.max_point(), Point::zero());
        assert_eq!(mapper.clamp_point(Point::new(10, 10)), Point::zero());
    }

    #[test]
    fn test_single_character_document() {
        let rope = create_test_rope("x");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.point_to_offset(Point::new(0, 0)), 0);
        assert_eq!(mapper.point_to_offset(Point::new(0, 1)), 1);
        assert_eq!(mapper.offset_to_point(0), Point::new(0, 0));
        assert_eq!(mapper.offset_to_point(1), Point::new(0, 1));
        assert_eq!(mapper.max_point(), Point::new(0, 1));
    }

    #[test]
    fn test_document_with_empty_lines() {
        let rope = create_test_rope("hello\n\nworld");
        let mapper = RopeCoordinateMapper::new(rope);
        
        assert_eq!(mapper.point_to_offset(Point::new(1, 0)), 6); // Start of empty line
        assert_eq!(mapper.offset_to_point(6), Point::new(1, 0)); 
        assert_eq!(mapper.offset_to_point(7), Point::new(2, 0)); // Start of "world"
    }
}