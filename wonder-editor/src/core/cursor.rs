use crate::core::{Point, CoordinateConversion};

/// Enhanced cursor with Point-based positioning and offset caching for performance
/// 
/// This cursor uses 2D coordinates internally for better accuracy, especially with
/// variable-width fonts and complex layouts, while maintaining backward compatibility
/// with offset-based APIs through caching.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    /// Primary position as 2D coordinates
    point: Point,
    /// Cached offset position for performance - updated when point changes
    cached_offset: usize,
    /// Flag to track if cached offset is valid
    offset_cache_valid: bool,
}

impl Cursor {
    /// Create a new cursor at the origin
    pub fn new() -> Self {
        Self {
            point: Point::zero(),
            cached_offset: 0,
            offset_cache_valid: true,
        }
    }

    /// Create a cursor at the specified offset position
    /// The Point will be calculated when needed using coordinate conversion
    pub fn at_position(position: usize) -> Self {
        Self {
            point: Point::zero(), // Will be calculated when needed
            cached_offset: position,
            offset_cache_valid: true,
        }
    }

    /// Create a cursor at the specified Point
    pub fn at_point(point: Point) -> Self {
        Self {
            point,
            cached_offset: 0, // Will be calculated when needed
            offset_cache_valid: false,
        }
    }

    /// Get the cursor position as an offset (backward compatibility)
    /// This uses the cached offset for performance when available
    pub fn position(&self) -> usize {
        self.cached_offset
    }

    /// Get the cursor position as a Point
    pub fn point(&self) -> Point {
        self.point
    }

    /// Set cursor position using an offset (backward compatibility)
    /// This invalidates the Point and will be recalculated when needed
    pub fn set_position(&mut self, position: usize) {
        self.cached_offset = position;
        self.offset_cache_valid = true;
        // Point will be recalculated when needed
        self.point = Point::zero();
    }

    /// Set cursor position using a Point
    /// This invalidates the offset cache
    pub fn set_point(&mut self, point: Point) {
        self.point = point;
        self.offset_cache_valid = false;
        self.cached_offset = 0; // Will be recalculated when needed
    }

    /// Update both point and offset simultaneously (for performance)
    /// This is useful when you have both values available
    pub fn set_position_with_point(&mut self, offset: usize, point: Point) {
        self.cached_offset = offset;
        self.point = point;
        self.offset_cache_valid = true;
    }

    /// Move cursor left by one position
    /// Uses Point-based movement when coordinate converter is available,
    /// otherwise falls back to offset-based movement
    pub fn move_left(&mut self) {
        if self.cached_offset > 0 {
            self.cached_offset -= 1;
            if self.offset_cache_valid {
                // Update point based on new offset - in a real implementation,
                // this would use coordinate conversion
                if self.point.column > 0 {
                    self.point.column -= 1;
                } else if self.point.row > 0 {
                    self.point.row -= 1;
                    // Column would be set to end of previous line - simplified for now
                    self.point.column = 0;
                }
            }
        }
    }

    /// Move cursor right by one position
    pub fn move_right(&mut self, max_position: usize) {
        if self.cached_offset < max_position {
            self.cached_offset += 1;
            if self.offset_cache_valid {
                // Simplified point update - in real implementation would use coordinate conversion
                self.point.column += 1;
            }
        }
    }

    /// Move cursor up one line (Point-based operation)
    pub fn move_up(&mut self) {
        if self.point.row > 0 {
            self.point.row -= 1;
            self.offset_cache_valid = false;
        }
    }

    /// Move cursor down one line (Point-based operation)
    pub fn move_down(&mut self) {
        self.point.row += 1;
        self.offset_cache_valid = false;
    }

    /// Move cursor to start of line
    pub fn move_to_line_start(&mut self) {
        self.point.column = 0;
        self.offset_cache_valid = false;
    }

    /// Move cursor to end of line (requires line length)
    pub fn move_to_line_end(&mut self, line_length: u32) {
        self.point.column = line_length;
        self.offset_cache_valid = false;
    }

    /// Clamp cursor to valid bounds using offset
    pub fn clamp_to_bounds(&mut self, max_position: usize) {
        if self.cached_offset > max_position {
            self.cached_offset = max_position;
            self.offset_cache_valid = true;
            self.point = Point::zero(); // Will be recalculated when needed
        }
    }

    /// Clamp cursor to valid bounds using Point
    pub fn clamp_to_point_bounds(&mut self, mapper: &impl CoordinateConversion) {
        let clamped_point = mapper.clamp_point(self.point);
        if clamped_point != self.point {
            self.point = clamped_point;
            self.offset_cache_valid = false;
        }
    }

    /// Ensure both point and offset are synchronized
    /// This should be called when you need both values to be accurate
    pub fn synchronize(&mut self, mapper: &impl CoordinateConversion) {
        if !self.offset_cache_valid {
            self.cached_offset = mapper.point_to_offset(self.point);
            self.offset_cache_valid = true;
        } else {
            // If offset is valid but point might not be, update point
            let calculated_point = mapper.offset_to_point(self.cached_offset);
            if calculated_point != self.point {
                self.point = calculated_point;
            }
        }
    }

    /// Check if the cursor is at the origin
    pub fn is_at_origin(&self) -> bool {
        self.point.is_zero() && self.cached_offset == 0
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{RopeCoordinateMapper};
    use ropey::Rope;

    fn create_test_mapper(content: &str) -> RopeCoordinateMapper {
        RopeCoordinateMapper::new(Rope::from_str(content))
    }

    // Backward compatibility tests (existing API)
    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new();
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.point(), Point::zero());
        assert!(cursor.is_at_origin());
    }

    #[test]
    fn test_cursor_at_position() {
        let cursor = Cursor::at_position(5);
        assert_eq!(cursor.position(), 5);
        // Point is not calculated until needed, but cached offset should be correct
        assert_eq!(cursor.point(), Point::zero()); // Will be calculated when synchronized
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::at_position(3);
        
        cursor.move_left();
        assert_eq!(cursor.position(), 2);
        
        cursor.move_right(10);
        assert_eq!(cursor.position(), 3);
        
        // Test bounds
        cursor.set_position(0);
        cursor.move_left();
        assert_eq!(cursor.position(), 0);
        
        cursor.set_position(10);
        cursor.move_right(10);
        assert_eq!(cursor.position(), 10);
    }

    #[test]
    fn test_cursor_clamp_to_bounds() {
        let mut cursor = Cursor::at_position(15);
        cursor.clamp_to_bounds(10);
        assert_eq!(cursor.position(), 10);
    }

    // New Point-based tests
    #[test]
    fn test_cursor_at_point() {
        let point = Point::new(2, 5);
        let cursor = Cursor::at_point(point);
        assert_eq!(cursor.point(), point);
        // Cached offset is not calculated until needed
        assert_eq!(cursor.position(), 0); // Will be calculated when synchronized
    }

    #[test]
    fn test_cursor_set_point() {
        let mut cursor = Cursor::new();
        let point = Point::new(1, 3);
        
        cursor.set_point(point);
        assert_eq!(cursor.point(), point);
        // Offset cache is invalidated
        assert_eq!(cursor.position(), 0); // Will be calculated when synchronized
    }

    #[test]
    fn test_cursor_set_position_with_point() {
        let mut cursor = Cursor::new();
        let point = Point::new(2, 4);
        let offset = 15;
        
        cursor.set_position_with_point(offset, point);
        assert_eq!(cursor.position(), offset);
        assert_eq!(cursor.point(), point);
    }

    #[test]
    fn test_cursor_point_based_movement() {
        let mut cursor = Cursor::at_point(Point::new(2, 5));
        
        cursor.move_up();
        assert_eq!(cursor.point(), Point::new(1, 5));
        
        cursor.move_down();
        assert_eq!(cursor.point(), Point::new(2, 5));
        
        cursor.move_to_line_start();
        assert_eq!(cursor.point(), Point::new(2, 0));
        
        cursor.move_to_line_end(10);
        assert_eq!(cursor.point(), Point::new(2, 10));
    }

    #[test]
    fn test_cursor_synchronization() {
        let mapper = create_test_mapper("hello\nworld\ntest");
        let mut cursor = Cursor::at_position(8); // Start of "rld"
        
        // Synchronize to get correct point
        cursor.synchronize(&mapper);
        
        // Should be at row 1, column 2 (in "world")
        assert_eq!(cursor.point(), Point::new(1, 2));
        assert_eq!(cursor.position(), 8);
    }

    #[test]
    fn test_cursor_synchronization_from_point() {
        let mapper = create_test_mapper("hello\nworld\ntest");
        let mut cursor = Cursor::at_point(Point::new(1, 2)); // "r" in "world"
        
        // Synchronize to get correct offset
        cursor.synchronize(&mapper);
        
        assert_eq!(cursor.point(), Point::new(1, 2));
        assert_eq!(cursor.position(), 8); // Should be at "rld" in "world"
    }

    #[test]
    fn test_cursor_clamp_to_point_bounds() {
        let mapper = create_test_mapper("hello\nworld");
        let mut cursor = Cursor::at_point(Point::new(0, 10)); // Beyond "hello"
        
        cursor.clamp_to_point_bounds(&mapper);
        
        // Should be clamped to end of first line
        assert_eq!(cursor.point(), Point::new(0, 5));
    }

    #[test]
    fn test_cursor_move_up_at_first_line() {
        let mut cursor = Cursor::at_point(Point::new(0, 5));
        
        cursor.move_up();
        
        // Should stay at row 0 (can't go up from first line)
        assert_eq!(cursor.point(), Point::new(0, 5));
    }

    #[test]
    fn test_cursor_combined_offset_and_point_operations() {
        let mut cursor = Cursor::at_position(5);
        
        // Move using offset-based API
        cursor.move_left();
        assert_eq!(cursor.position(), 4);
        
        // Move using Point-based API
        cursor.move_down();
        let point = cursor.point();
        assert_eq!(point.row, 1); // Should be one row down
        
        // Offset cache should be invalidated after point-based movement
        // In a real scenario, synchronization would be needed to get accurate offset
    }

    #[test]
    fn test_cursor_default() {
        let cursor: Cursor = Default::default();
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.point(), Point::zero());
        assert!(cursor.is_at_origin());
    }

    #[test]
    fn test_cursor_round_trip_conversion() {
        let mapper = create_test_mapper("hello\nworld\ntest line");
        
        // Test various positions (document has 21 characters total: "hello\nworld\ntest line")
        let test_positions = vec![0, 5, 6, 11, 12, 21];
        
        for pos in test_positions {
            let mut cursor = Cursor::at_position(pos);
            cursor.synchronize(&mapper);
            
            // Position should remain the same after synchronization
            assert_eq!(cursor.position(), pos, "Position mismatch for offset {}", pos);
            
            // Convert to point and back should give same result
            let point = cursor.point();
            let converted_offset = mapper.point_to_offset(point);
            assert_eq!(converted_offset, pos, "Round-trip conversion failed for offset {}", pos);
        }
    }

    #[test]
    fn test_cursor_equality_after_synchronization() {
        let mapper = create_test_mapper("hello\nworld");
        
        let mut cursor1 = Cursor::at_position(8);
        let mut cursor2 = Cursor::at_point(Point::new(1, 2));
        
        cursor1.synchronize(&mapper);
        cursor2.synchronize(&mapper);
        
        // Both cursors should now be at the same logical position
        assert_eq!(cursor1.position(), cursor2.position());
        assert_eq!(cursor1.point(), cursor2.point());
    }
}