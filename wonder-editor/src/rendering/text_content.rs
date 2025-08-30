use std::ops::Range;
use ropey::RopeSlice;

/// Trait for text content that can be efficiently processed by the hybrid renderer
pub trait TextContent {
    fn text_len(&self) -> usize;
    fn text_is_empty(&self) -> bool;
    fn text_slice(&self, range: Range<usize>) -> String;
    fn text_to_string(&self) -> String;
    fn char_at(&self, index: usize) -> Option<char>;
    fn chars_count(&self) -> usize;
}

impl TextContent for &str {
    fn text_len(&self) -> usize {
        self.chars().count()
    }
    
    fn text_is_empty(&self) -> bool {
        str::is_empty(self)
    }
    
    fn text_slice(&self, range: Range<usize>) -> String {
        self.chars().skip(range.start).take(range.end - range.start).collect()
    }
    
    fn text_to_string(&self) -> String {
        (*self).to_string()
    }
    
    fn char_at(&self, index: usize) -> Option<char> {
        self.chars().nth(index)
    }
    
    fn chars_count(&self) -> usize {
        self.chars().count()
    }
}

impl TextContent for RopeSlice<'_> {
    fn text_len(&self) -> usize {
        self.len_chars()
    }
    
    fn text_is_empty(&self) -> bool {
        self.len_chars() == 0
    }
    
    fn text_slice(&self, range: Range<usize>) -> String {
        if range.end <= self.len_chars() {
            self.slice(range.start..range.end).to_string()
        } else {
            self.slice(range.start..).to_string()
        }
    }
    
    fn text_to_string(&self) -> String {
        self.to_string()
    }
    
    fn char_at(&self, index: usize) -> Option<char> {
        if index < self.len_chars() {
            Some(self.char(index))
        } else {
            None
        }
    }
    
    fn chars_count(&self) -> usize {
        self.len_chars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_text_content_str() {
        let text = "Hello, world!";
        assert_eq!(text.text_len(), 13);
        assert!(!text.text_is_empty());
        assert_eq!(text.text_slice(0..5), "Hello");
        assert_eq!(text.char_at(7), Some('w'));
        assert_eq!(text.chars_count(), 13);
    }

    #[test]
    fn test_text_content_rope_slice() {
        let rope = Rope::from("Hello, world!");
        let slice = rope.slice(..);
        assert_eq!(slice.text_len(), 13);
        assert!(!slice.text_is_empty());
        assert_eq!(slice.text_slice(0..5), "Hello");
        assert_eq!(slice.char_at(7), Some('w'));
        assert_eq!(slice.chars_count(), 13);
    }

    #[test]
    fn test_text_content_empty() {
        let text = "";
        assert_eq!(text.text_len(), 0);
        assert!(text.text_is_empty());
        assert_eq!(text.char_at(0), None);
        
        let rope = Rope::from("");
        let slice = rope.slice(..);
        assert_eq!(slice.text_len(), 0);
        assert!(slice.text_is_empty());
        assert_eq!(slice.char_at(0), None);
    }
}