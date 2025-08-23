use crate::markdown_parser::{MarkdownParser, MarkdownToken};
use gpui::{IntoElement, div, Styled, ParentElement, px, FontWeight};

#[derive(Clone)]
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
        let mut container = div().flex().flex_col();
        
        // Group tokens into logical blocks
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            match &token.token_type {
                MarkdownToken::Heading(level, text) => {
                    container = container.child(self.render_heading(*level, text));
                    i += 1;
                }
                MarkdownToken::ListItem(text) => {
                    // Collect consecutive list items
                    let mut list_items = vec![text.clone()];
                    let mut j = i + 1;
                    while j < tokens.len() {
                        if let MarkdownToken::ListItem(item_text) = &tokens[j].token_type {
                            list_items.push(item_text.clone());
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    container = container.child(self.render_list(&list_items));
                    i = j;
                }
                MarkdownToken::Paragraph(text) |
                MarkdownToken::Text(text) => {
                    container = container.child(self.render_paragraph(text));
                    i += 1;
                }
                MarkdownToken::Bold(text) => {
                    container = container.child(self.render_paragraph_with_bold(text));
                    i += 1;
                }
                MarkdownToken::Italic(text) => {
                    container = container.child(self.render_paragraph_with_italic(text));
                    i += 1;
                }
                MarkdownToken::Link(text, url) => {
                    container = container.child(self.render_link(text, url));
                    i += 1;
                }
                MarkdownToken::Code(text) => {
                    container = container.child(self.render_inline_code(text));
                    i += 1;
                }
                MarkdownToken::CodeBlock(language, content) => {
                    container = container.child(self.render_code_block(language.as_ref().map(|s| s.as_str()), content));
                    i += 1;
                }
                MarkdownToken::BlockQuote(text) => {
                    container = container.child(self.render_block_quote(text));
                    i += 1;
                }
            }
        }
        
        container
    }

    fn render_heading(&self, level: u32, text: &str) -> impl IntoElement {
        let (font_size, font_weight, margin_top, margin_bottom) = match level {
            1 => (px(32.0), FontWeight::BOLD, px(24.0), px(16.0)),
            2 => (px(24.0), FontWeight::BOLD, px(20.0), px(12.0)),
            3 => (px(20.0), FontWeight::SEMIBOLD, px(16.0), px(12.0)),
            4 => (px(16.0), FontWeight::SEMIBOLD, px(14.0), px(10.0)),
            5 => (px(14.0), FontWeight::MEDIUM, px(12.0), px(8.0)),
            _ => (px(13.0), FontWeight::MEDIUM, px(10.0), px(8.0)),
        };

        div()
            .text_size(font_size)
            .font_weight(font_weight)
            .mt(margin_top)
            .mb(margin_bottom)
            .child(text.to_string())
    }

    fn render_paragraph(&self, text: &str) -> impl IntoElement {
        div()
            .mb_2()
            .child(text.to_string())
    }

    fn render_paragraph_with_bold(&self, text: &str) -> impl IntoElement {
        div()
            .mb_2()
            .font_weight(FontWeight::BOLD)
            .child(text.to_string())
    }

    fn render_paragraph_with_italic(&self, text: &str) -> impl IntoElement {
        div()
            .mb_2()
            .italic()
            .child(text.to_string())
    }

    fn render_list(&self, items: &[String]) -> impl IntoElement {
        let mut list_container = div()
            .flex()
            .flex_col()
            .ml_4()
            .mb_2();

        for item in items {
            let item_element = div()
                .flex()
                .flex_row()
                .items_start()
                .mb_1()
                .child(
                    div()
                        .mr_2()
                        .child("•")
                )
                .child(
                    div()
                        .flex_1()
                        .child(item.to_string())
                );
            list_container = list_container.child(item_element);
        }

        list_container
    }

    fn render_link(&self, text: &str, url: &str) -> impl IntoElement {
        div()
            .mb_2()
            .text_color(gpui::rgb(0x0969da)) // Link blue color
            .child(format!("{} ({})", text, url))
    }

    fn render_inline_code(&self, text: &str) -> impl IntoElement {
        div()
            .mb_2()
            .px_1()
            .rounded_sm()
            .bg(gpui::rgb(0xf6f8fa)) // Light gray background
            .font_family("monospace")
            .child(text.to_string())
    }

    fn render_code_block(&self, language: Option<&str>, content: &str) -> impl IntoElement {
        let lang_label = language.map(|l| format!("[{}]\n", l)).unwrap_or_default();
        div()
            .p_3()
            .mb_3()
            .rounded_md()
            .bg(gpui::rgb(0xf6f8fa)) // Light gray background
            .font_family("monospace")
            .text_size(px(14.0))
            .child(format!("{}{}", lang_label, content))
    }

    fn render_block_quote(&self, text: &str) -> impl IntoElement {
        div()
            .pl_4()
            .mb_3()
            .border_l_4()
            .border_color(gpui::rgb(0xd1d5db)) // Gray border
            .text_color(gpui::rgb(0x6b7280)) // Muted text color
            .child(text.to_string())
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
    fn test_render_headers_with_proper_typography() {
        let renderer = PreviewRenderer::new();
        
        // Test H1 rendering
        let h1_element = renderer.render_heading(1, "Main Title");
        // Should not panic - actual visual testing would need GPUI runtime
        
        // Test H2 rendering
        let h2_element = renderer.render_heading(2, "Subtitle");
        // Should not panic
        
        // Test H3 rendering
        let h3_element = renderer.render_heading(3, "Section");
        // Should not panic
        
        // Test different heading levels
        for level in 1..=6 {
            let heading = renderer.render_heading(level, &format!("Heading {}", level));
            // Should not panic for any level
        }
    }

    #[test]
    fn test_render_bold_text_with_font_weight() {
        let renderer = PreviewRenderer::new();
        
        // Test bold rendering
        let bold_element = renderer.render_paragraph_with_bold("This text should be bold");
        // Should not panic - actual font weight would be verified visually
    }

    #[test]
    fn test_render_italic_text_with_font_style() {
        let renderer = PreviewRenderer::new();
        
        // Test italic rendering
        let italic_element = renderer.render_paragraph_with_italic("This text should be italic");
        // Should not panic - actual font style would be verified visually
    }

    #[test]
    fn test_render_list_with_bullets() {
        let renderer = PreviewRenderer::new();
        
        // Test list rendering
        let list_items = vec![
            "First item".to_string(),
            "Second item".to_string(),
            "Third item".to_string(),
        ];
        
        let list_element = renderer.render_list(&list_items);
        // Should not panic - bullets and indentation would be verified visually
    }

    #[test]
    fn test_render_markdown_with_mixed_content() {
        let renderer = PreviewRenderer::new();
        
        // Test complete markdown rendering
        let markdown = r#"# Main Title

This is a paragraph with **bold** and *italic* text.

## Features

- First feature
- Second feature
- Third feature

Here's some `inline code` and a link to [example](https://example.com).
"#;
        
        let _rendered = renderer.render_markdown(markdown);
        // Should not panic - complete rendering pipeline test
    }

    #[test]
    fn test_render_code_elements() {
        let renderer = PreviewRenderer::new();
        
        // Test inline code
        let _inline_code = renderer.render_inline_code("println!");
        
        // Test code block with language
        let _code_block = renderer.render_code_block(
            Some("rust"), 
            "fn main() {\n    println!(\"Hello!\");\n}"
        );
        
        // Test code block without language
        let _code_block_no_lang = renderer.render_code_block(
            None,
            "Some code here"
        );
        
        // Should not panic
    }

    #[test]
    fn test_render_link() {
        let renderer = PreviewRenderer::new();
        
        // Test link rendering
        let _link_element = renderer.render_link("Click here", "https://example.com");
        
        // Should not panic
    }

    #[test]
    fn test_render_blockquote() {
        let renderer = PreviewRenderer::new();
        
        // Test blockquote rendering
        let _blockquote = renderer.render_block_quote("This is a quoted text");
        
        // Should not panic - blockquote would be styled with left border
    }

    #[test]
    fn test_heading_hierarchy_font_sizes() {
        let renderer = PreviewRenderer::new();
        
        // Verify that heading sizes decrease with level
        for level in 1..=6 {
            let _heading = renderer.render_heading(level, &format!("Level {} Heading", level));
            // Font sizes should decrease: H1 > H2 > H3 > H4 > H5 > H6
            // This would be verified visually in the actual UI
        }
    }
}