#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    anchor: Option<usize>,
}

impl Selection {
    pub fn new() -> Self {
        Self { anchor: None }
    }

    pub fn start(&mut self, position: usize) {
        self.anchor = Some(position);
    }

    pub fn clear(&mut self) {
        self.anchor = None;
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn range(&self, cursor_position: usize) -> Option<(usize, usize)> {
        self.anchor.map(|anchor| {
            if anchor <= cursor_position {
                (anchor, cursor_position)
            } else {
                (cursor_position, anchor)
            }
        })
    }

    pub fn start_position(&self) -> Option<usize> {
        self.anchor
    }

    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    pub fn length(&self, cursor_position: usize) -> usize {
        self.range(cursor_position)
            .map(|(start, end)| end - start)
            .unwrap_or(0)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_creation() {
        let selection = Selection::new();
        assert!(!selection.is_active());
        assert_eq!(selection.range(5), None);
    }

    #[test]
    fn test_selection_start_and_clear() {
        let mut selection = Selection::new();
        
        selection.start(3);
        assert!(selection.is_active());
        assert_eq!(selection.start_position(), Some(3));
        
        selection.clear();
        assert!(!selection.is_active());
        assert_eq!(selection.start_position(), None);
    }

    #[test]
    fn test_selection_range() {
        let mut selection = Selection::new();
        selection.start(3);
        
        // Cursor after anchor
        assert_eq!(selection.range(7), Some((3, 7)));
        
        // Cursor before anchor
        assert_eq!(selection.range(1), Some((1, 3)));
        
        // Cursor at anchor
        assert_eq!(selection.range(3), Some((3, 3)));
    }

    #[test]
    fn test_selection_length() {
        let mut selection = Selection::new();
        
        // No selection
        assert_eq!(selection.length(5), 0);
        
        selection.start(3);
        assert_eq!(selection.length(7), 4);
        assert_eq!(selection.length(1), 2);
        assert_eq!(selection.length(3), 0);
    }
}