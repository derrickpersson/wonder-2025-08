//! Point-based coordinate system for accurate cursor positioning
//! 
//! This module provides 2D coordinate representation for text positions,
//! following Zed's proven approach to cursor positioning accuracy.

use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Range, Sub},
};

/// A zero-indexed point in a text buffer consisting of a row and column.
/// 
/// This provides more accurate cursor positioning than offset-based systems,
/// especially for variable-width fonts and mixed content rendering.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub struct Point {
    pub row: u32,
    pub column: u32,
}

impl Point {
    /// Maximum point value for boundary conditions
    pub const MAX: Self = Self {
        row: u32::MAX,
        column: u32::MAX,
    };

    /// Creates a new Point with the given row and column
    pub fn new(row: u32, column: u32) -> Self {
        Point { row, column }
    }

    /// Creates a Point range covering the specified rows (from start to end, both inclusive)
    pub fn row_range(range: Range<u32>) -> Range<Self> {
        Point {
            row: range.start,
            column: 0,
        }..Point {
            row: range.end,
            column: 0,
        }
    }

    /// Creates a Point at origin (0, 0)
    pub fn zero() -> Self {
        Point::new(0, 0)
    }

    /// Parse a string to determine the Point at its end
    /// Used for calculating document dimensions
    pub fn parse_str(s: &str) -> Self {
        let mut point = Self::zero();
        for (row, line) in s.split('\n').enumerate() {
            point.row = row as u32;
            point.column = line.chars().count() as u32;
        }
        point
    }

    /// Returns true if this point is at the origin
    pub fn is_zero(&self) -> bool {
        self.row == 0 && self.column == 0
    }

    /// Subtracts another point, returning zero if the result would be negative
    pub fn saturating_sub(self, other: Self) -> Self {
        if self < other {
            Self::zero()
        } else {
            self - other
        }
    }
}

impl<'a> Add<&'a Self> for Point {
    type Output = Point;

    fn add(self, other: &'a Self) -> Self::Output {
        self + *other
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Self) -> Self::Output {
        if other.row == 0 {
            // Adding within same row - just add columns
            Point::new(self.row, self.column + other.column)
        } else {
            // Adding across rows - take other's column as absolute column
            Point::new(self.row + other.row, other.column)
        }
    }
}

impl<'a> Sub<&'a Self> for Point {
    type Output = Point;

    fn sub(self, other: &'a Self) -> Self::Output {
        self - *other
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Self) -> Self::Output {
        debug_assert!(other <= self, "Cannot subtract larger point from smaller point");

        if self.row == other.row {
            // Same row - subtract columns
            Point::new(0, self.column - other.column)
        } else {
            // Different rows - return row difference and current column
            Point::new(self.row - other.row, self.column)
        }
    }
}

impl<'a> AddAssign<&'a Self> for Point {
    fn add_assign(&mut self, other: &'a Self) {
        *self += *other;
    }
}

impl AddAssign<Self> for Point {
    fn add_assign(&mut self, other: Self) {
        if other.row == 0 {
            self.column += other.column;
        } else {
            self.row += other.row;
            self.column = other.column;
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    #[cfg(target_pointer_width = "64")]
    fn cmp(&self, other: &Point) -> Ordering {
        // Efficient comparison for 64-bit systems using bit packing
        let a = ((self.row as usize) << 32) | self.column as usize;
        let b = ((other.row as usize) << 32) | other.column as usize;
        a.cmp(&b)
    }

    #[cfg(target_pointer_width = "32")]
    fn cmp(&self, other: &Point) -> Ordering {
        // Standard comparison for 32-bit systems
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.column.cmp(&other.column),
            comparison @ _ => comparison,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(5, 10);
        assert_eq!(point.row, 5);
        assert_eq!(point.column, 10);
    }

    #[test]
    fn test_point_zero() {
        let zero = Point::zero();
        assert_eq!(zero.row, 0);
        assert_eq!(zero.column, 0);
        assert!(zero.is_zero());

        let non_zero = Point::new(1, 0);
        assert!(!non_zero.is_zero());
    }

    #[test]
    fn test_point_max() {
        let max = Point::MAX;
        assert_eq!(max.row, u32::MAX);
        assert_eq!(max.column, u32::MAX);
    }

    #[test]
    fn test_point_parse_str() {
        let single_line = Point::parse_str("hello");
        assert_eq!(single_line, Point::new(0, 5));

        let multi_line = Point::parse_str("hello\nworld\ntest");
        assert_eq!(multi_line, Point::new(2, 4));

        let empty = Point::parse_str("");
        assert_eq!(empty, Point::new(0, 0));

        let with_empty_line = Point::parse_str("hello\n\nworld");
        assert_eq!(with_empty_line, Point::new(2, 5));
    }

    #[test]
    fn test_point_row_range() {
        let range = Point::row_range(2..5);
        assert_eq!(range.start, Point::new(2, 0));
        assert_eq!(range.end, Point::new(5, 0));
    }

    #[test]
    fn test_point_addition() {
        let p1 = Point::new(2, 5);
        let p2 = Point::new(0, 3);
        let result = p1 + p2;
        assert_eq!(result, Point::new(2, 8)); // Same row, add columns

        let p3 = Point::new(1, 10);
        let result2 = p1 + p3;
        assert_eq!(result2, Point::new(3, 10)); // Different rows, use p3's column
    }

    #[test]
    fn test_point_addition_with_reference() {
        let p1 = Point::new(2, 5);
        let p2 = Point::new(0, 3);
        let result = p1 + &p2;
        assert_eq!(result, Point::new(2, 8));
    }

    #[test]
    fn test_point_subtraction() {
        let p1 = Point::new(3, 8);
        let p2 = Point::new(0, 3);
        let result = p1 - p2;
        assert_eq!(result, Point::new(3, 8)); // Different rows, return p1

        let p3 = Point::new(3, 5);
        let result2 = p1 - p3;
        assert_eq!(result2, Point::new(0, 3)); // Same row, subtract columns
    }

    #[test]
    fn test_point_subtraction_with_reference() {
        let p1 = Point::new(3, 8);
        let p2 = Point::new(0, 3);
        let result = p1 - &p2;
        assert_eq!(result, Point::new(3, 8));
    }

    #[test]
    fn test_point_saturating_sub() {
        let p1 = Point::new(5, 10);
        let p2 = Point::new(2, 3);
        let result = p1.saturating_sub(p2);
        assert_eq!(result, Point::new(3, 10));

        let p3 = Point::new(10, 20);
        let result2 = p1.saturating_sub(p3);
        assert_eq!(result2, Point::zero()); // p3 > p1, so return zero
    }

    #[test]
    fn test_point_add_assign() {
        let mut p1 = Point::new(2, 5);
        let p2 = Point::new(0, 3);
        p1 += p2;
        assert_eq!(p1, Point::new(2, 8));

        let mut p3 = Point::new(2, 5);
        let p4 = Point::new(1, 10);
        p3 += p4;
        assert_eq!(p3, Point::new(3, 10));
    }

    #[test]
    fn test_point_add_assign_with_reference() {
        let mut p1 = Point::new(2, 5);
        let p2 = Point::new(0, 3);
        p1 += &p2;
        assert_eq!(p1, Point::new(2, 8));
    }

    #[test]
    fn test_point_comparison() {
        let p1 = Point::new(2, 5);
        let p2 = Point::new(2, 5);
        let p3 = Point::new(2, 6);
        let p4 = Point::new(3, 0);

        assert_eq!(p1, p2);
        assert!(p1 < p3);
        assert!(p1 < p4);
        assert!(p3 > p1);
        assert!(p4 > p1);
        assert!(p4 > p3);
    }

    #[test]
    fn test_point_ordering() {
        let mut points = vec![
            Point::new(3, 5),
            Point::new(1, 10),
            Point::new(2, 0),
            Point::new(1, 5),
        ];
        
        points.sort();
        
        let expected = vec![
            Point::new(1, 5),
            Point::new(1, 10),
            Point::new(2, 0),
            Point::new(3, 5),
        ];
        
        assert_eq!(points, expected);
    }

    #[test]
    fn test_point_debug_output() {
        let point = Point::new(3, 7);
        let debug_str = format!("{:?}", point);
        assert!(debug_str.contains("Point"));
        assert!(debug_str.contains("row: 3"));
        assert!(debug_str.contains("column: 7"));
    }

    #[test]
    fn test_point_clone_and_copy() {
        let p1 = Point::new(5, 10);
        let p2 = p1; // Copy
        let p3 = p1.clone(); // Clone
        
        assert_eq!(p1, p2);
        assert_eq!(p1, p3);
        assert_eq!(p2, p3);
    }

    #[test]
    fn test_point_default() {
        let default_point: Point = Default::default();
        assert_eq!(default_point, Point::zero());
    }

    #[test]
    fn test_point_hash() {
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        let p1 = Point::new(2, 5);
        let p2 = Point::new(2, 5);
        let p3 = Point::new(3, 5);
        
        map.insert(p1, "first");
        map.insert(p3, "second");
        
        assert_eq!(map.get(&p2), Some(&"first")); // p1 and p2 are equal
        assert_eq!(map.len(), 2);
    }
}