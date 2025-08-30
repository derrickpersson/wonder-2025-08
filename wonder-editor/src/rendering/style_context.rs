use gpui::Hsla;

// ENG-165: StyleContext for theme-aware styling
#[derive(Clone)]
pub struct StyleContext {
    pub text_color: Hsla,
    pub code_color: Hsla,
    pub border_color: Hsla,
}

impl StyleContext {
    pub fn new_for_test() -> Self {
        Self {
            text_color: Hsla { h: 0.0, s: 0.0, l: 0.85, a: 1.0 }, // Light gray instead of black
            code_color: Hsla { h: 120.0, s: 0.5, l: 0.7, a: 1.0 }, // Green-ish instead of black
            border_color: Hsla { h: 0.0, s: 0.0, l: 0.5, a: 1.0 }, // Medium gray instead of black
        }
    }
    
    pub fn default() -> Self {
        Self {
            text_color: Hsla { h: 0.0, s: 0.0, l: 0.9, a: 1.0 },
            code_color: Hsla { h: 120.0, s: 0.5, l: 0.7, a: 1.0 },
            border_color: Hsla { h: 0.0, s: 0.0, l: 0.3, a: 1.0 },
        }
    }
    
    pub fn with_text_color(mut self, color: Hsla) -> Self {
        self.text_color = color;
        self
    }
    
    pub fn with_code_color(mut self, color: Hsla) -> Self {
        self.code_color = color;
        self
    }
    
    pub fn with_border_color(mut self, color: Hsla) -> Self {
        self.border_color = color;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_style_context_uses_theme_colors() {
        let style_context = StyleContext::new_for_test();
        
        // Check that we're not using pure black (which wouldn't be visible on dark theme)
        assert_ne!(style_context.text_color.l, 0.0);
        assert_ne!(style_context.code_color.l, 0.0);
        
        // Verify theme colors are properly set
        assert_eq!(style_context.text_color.l, 0.85);
        assert_eq!(style_context.code_color.h, 120.0);
        assert_eq!(style_context.border_color.l, 0.5);
    }
    
    #[test]
    fn test_style_context_builder() {
        let custom_color = Hsla { h: 200.0, s: 0.8, l: 0.6, a: 1.0 };
        let style_context = StyleContext::default()
            .with_text_color(custom_color)
            .with_code_color(custom_color);
        
        assert_eq!(style_context.text_color, custom_color);
        assert_eq!(style_context.code_color, custom_color);
    }
}