use crate::markdown_parser::{MarkdownParser, ParsedToken, MarkdownToken};
use gpui::{IntoElement, div, Styled, ParentElement, px, FontWeight};

pub struct PreviewRenderer {
    parser: MarkdownParser,
}

impl PreviewRenderer {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }

    pub fn render_markdown(&self, markdown: &str) -> impl IntoElement {
        let tokens = self.parser.parse_with_positions(markdown);
        let mut container = div();
        
        for token in tokens {
            match token.token_type {
                MarkdownToken::Heading(level, text) => {
                    container = container.child(self.render_heading(level, &text));
                }
                MarkdownToken::Bold(text) => {
                    container = container.child(self.render_bold(&text));
                }
                MarkdownToken::Italic(text) => {
                    container = container.child(self.render_italic(&text));
                }
                MarkdownToken::Link(text, url) => {
                    container = container.child(self.render_link(&text, &url));
                }
                MarkdownToken::Code(text) => {
                    container = container.child(self.render_inline_code(&text));
                }
                MarkdownToken::CodeBlock(language, content) => {
                    container = container.child(self.render_code_block(language.as_ref().map(|s| s.as_str()), &content));
                }
                _ => {
                    // For now, render other tokens as plain text
                }
            }
        }
        
        container
    }

    fn render_heading(&self, level: u32, text: &str) -> impl IntoElement {
        match level {
            1 => div()
                .font_weight(FontWeight::BOLD)
                .p_1()
                .child(text.to_string()),
            2 => div()
                .font_weight(FontWeight::BOLD)
                .p_1()
                .child(text.to_string()),
            3 => div()
                .font_weight(FontWeight::BOLD)
                .p_1()
                .child(text.to_string()),
            _ => div()
                .font_weight(FontWeight::BOLD)
                .p_1()
                .child(text.to_string()),
        }
    }

    fn render_bold(&self, text: &str) -> impl IntoElement {
        div()
            .font_weight(FontWeight::BOLD)
            .child(text.to_string())
    }

    fn render_italic(&self, text: &str) -> impl IntoElement {
        div()
            .child(text.to_string()) // Basic italic styling will be added later
    }

    fn render_link(&self, text: &str, url: &str) -> impl IntoElement {
        div()
            .child(format!("{} ({})", text, url)) // For now, show as text with URL
    }

    fn render_inline_code(&self, text: &str) -> impl IntoElement {
        div()
            .child(text.to_string()) // Will add monospace styling
    }

    fn render_code_block(&self, language: Option<&str>, content: &str) -> impl IntoElement {
        let lang_prefix = language.map(|l| format!("[{}]\n", l)).unwrap_or_default();
        div()
            .p_2()
            .child(format!("{}{}", lang_prefix, content))
    }
}

impl Default for PreviewRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_renderer_creation() {
        let renderer = PreviewRenderer::new();
        // Basic creation test - should not panic
        assert!(!std::ptr::eq(&renderer.parser, std::ptr::null()));
    }

    #[test]
    fn test_render_header() {
        let renderer = PreviewRenderer::new();
        let markdown = "# Hello World";
        
        // This should not panic and should process the heading
        let _element = renderer.render_markdown(markdown);
        
        // Test specific heading levels
        let heading1 = renderer.render_heading(1, "Title 1");
        let heading2 = renderer.render_heading(2, "Title 2");
        let heading3 = renderer.render_heading(3, "Title 3");
        
        // These should all succeed without panicking
        // The actual styling will be tested when we integrate with GPUI styling
    }

    #[test]
    fn test_render_bold_italic() {
        let renderer = PreviewRenderer::new();
        
        // Test bold rendering
        let _bold_element = renderer.render_bold("bold text");
        
        // Test italic rendering  
        let _italic_element = renderer.render_italic("italic text");
        
        // These should succeed without panicking
    }

    #[test]
    fn test_render_link() {
        let renderer = PreviewRenderer::new();
        
        // Test link rendering
        let _link_element = renderer.render_link("Click here", "https://example.com");
        
        // Should not panic
    }

    #[test]
    fn test_render_code_elements() {
        let renderer = PreviewRenderer::new();
        
        // Test inline code
        let _inline_code = renderer.render_inline_code("println!");
        
        // Test code block
        let _code_block = renderer.render_code_block(Some("rust"), "fn main() {\n    println!(\"Hello!\");\n}");
        
        // Should not panic
    }
}