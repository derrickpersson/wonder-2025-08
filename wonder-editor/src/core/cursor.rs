#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    position: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { position: 0 }
    }

    pub fn at_position(position: usize) -> Self {
        Self { position }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn move_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }

    pub fn move_right(&mut self, max_position: usize) {
        if self.position < max_position {
            self.position += 1;
        }
    }

    pub fn clamp_to_bounds(&mut self, max_position: usize) {
        if self.position > max_position {
            self.position = max_position;
        }
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

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new();
        assert_eq!(cursor.position(), 0);
    }

    #[test]
    fn test_cursor_at_position() {
        let cursor = Cursor::at_position(5);
        assert_eq!(cursor.position(), 5);
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
}