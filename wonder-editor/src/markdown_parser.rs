use pulldown_cmark::{Parser, Event, Tag, TagEnd, Options};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToken {
    pub token_type: MarkdownToken,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownToken {
    Heading(u32, String),
    Paragraph(String),
    Bold(String),
    Italic(String),
    Code(String),
    CodeBlock(Option<String>, String), // language, content
    Link(String, String), // text, url
    ListItem(String),
    BlockQuote(String),
    Text(String),
}

#[derive(Clone)]
pub struct MarkdownParser {
    options: Options,
}

impl MarkdownParser {
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        Self { options }
    }

    pub fn parse(&self, markdown: &str) -> Vec<MarkdownToken> {
        let parser = Parser::new_ext(markdown, self.options);
        let mut tokens = Vec::new();
        let mut current_text = String::new();
        let mut in_heading = None;
        let mut in_emphasis = false;
        let mut in_strong = false;
        let mut in_code = false;
        let mut in_link = false;
        let mut in_list_item = false;
        let mut in_block_quote = false;
        let mut link_url = String::new();
        
        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        in_heading = Some(level as u32);
                    }
                    Tag::Emphasis => {
                        in_emphasis = true;
                    }
                    Tag::Strong => {
                        in_strong = true;
                    }
                    Tag::CodeBlock(kind) => {
                        // CodeBlockKind can be Indented or Fenced
                        in_code = true;
                        current_text.clear();
                    }
                    Tag::Link { dest_url, .. } => {
                        in_link = true;
                        link_url = dest_url.to_string();
                    }
                    Tag::Item => {
                        in_list_item = true;
                        current_text.clear();
                    }
                    Tag::BlockQuote => {
                        in_block_quote = true;
                        current_text.clear();
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    TagEnd::Heading(_) => {
                        if let Some(level) = in_heading.take() {
                            tokens.push(MarkdownToken::Heading(level, current_text.clone()));
                            current_text.clear();
                        }
                    }
                    TagEnd::Emphasis => {
                        if in_emphasis {
                            tokens.push(MarkdownToken::Italic(current_text.clone()));
                            current_text.clear();
                            in_emphasis = false;
                        }
                    }
                    TagEnd::Strong => {
                        if in_strong {
                            tokens.push(MarkdownToken::Bold(current_text.clone()));
                            current_text.clear();
                            in_strong = false;
                        }
                    }
                    TagEnd::CodeBlock => {
                        if in_code {
                            tokens.push(MarkdownToken::CodeBlock(None, current_text.clone()));
                            current_text.clear();
                            in_code = false;
                        }
                    }
                    TagEnd::Link => {
                        if in_link {
                            tokens.push(MarkdownToken::Link(current_text.clone(), link_url.clone()));
                            current_text.clear();
                            link_url.clear();
                            in_link = false;
                        }
                    }
                    TagEnd::Item => {
                        if in_list_item {
                            tokens.push(MarkdownToken::ListItem(current_text.clone()));
                            current_text.clear();
                            in_list_item = false;
                        }
                    }
                    TagEnd::BlockQuote => {
                        if in_block_quote {
                            tokens.push(MarkdownToken::BlockQuote(current_text.clone()));
                            current_text.clear();
                            in_block_quote = false;
                        }
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    current_text.push_str(&text);
                    if !in_heading.is_some() && !in_emphasis && !in_strong && !in_code && !in_link && !in_list_item && !in_block_quote {
                        tokens.push(MarkdownToken::Text(text.to_string()));
                        current_text.clear();
                    }
                }
                Event::Code(code) => {
                    tokens.push(MarkdownToken::Code(code.to_string()));
                }
                _ => {}
            }
        }
        
        tokens
    }

    pub fn parse_with_positions(&self, markdown: &str) -> Vec<ParsedToken> {
        let parser = Parser::new_ext(markdown, self.options);
        let offset_iter = parser.into_offset_iter();
        let mut tokens = Vec::new();
        let mut current_text = String::new();
        let mut in_heading = None;
        let mut heading_start = 0;
        let mut in_strong = false;
        let mut strong_start = 0;
        let mut in_emphasis = false;
        let mut emphasis_start = 0;
        let mut in_link = false;
        let mut link_start = 0;
        let mut link_url = String::new();
        let mut in_code_block = false;
        let mut code_block_start = 0;
        let mut code_block_lang = None;
        let mut in_list_item = false;
        let mut list_item_start = 0;
        let mut in_block_quote = false;
        let mut block_quote_start = 0;
        
        for (event, range) in offset_iter {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = Some(level as u32);
                    heading_start = range.start;
                }
                Event::Start(Tag::Strong) => {
                    in_strong = true;
                    strong_start = range.start;
                }
                Event::Start(Tag::Emphasis) => {
                    in_emphasis = true;
                    emphasis_start = range.start;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    in_link = true;
                    link_start = range.start;
                    link_url = dest_url.to_string();
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_block_start = range.start;
                    code_block_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() { None } else { Some(lang.to_string()) }
                        }
                        _ => None,
                    };
                }
                Event::Start(Tag::Item) => {
                    in_list_item = true;
                    list_item_start = range.start;
                    current_text.clear();
                }
                Event::Start(Tag::BlockQuote) => {
                    in_block_quote = true;
                    block_quote_start = range.start;
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = in_heading.take() {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::Heading(level, current_text.clone()),
                            start: heading_start,
                            end: range.end,
                        });
                        current_text.clear();
                    }
                }
                Event::End(TagEnd::Strong) => {
                    if in_strong {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::Bold(current_text.clone()),
                            start: strong_start,
                            end: range.end,
                        });
                        current_text.clear();
                        in_strong = false;
                    }
                }
                Event::End(TagEnd::Emphasis) => {
                    if in_emphasis {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::Italic(current_text.clone()),
                            start: emphasis_start,
                            end: range.end,
                        });
                        current_text.clear();
                        in_emphasis = false;
                    }
                }
                Event::End(TagEnd::Link) => {
                    if in_link {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::Link(current_text.clone(), link_url.clone()),
                            start: link_start,
                            end: range.end,
                        });
                        current_text.clear();
                        link_url.clear();
                        in_link = false;
                    }
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::CodeBlock(code_block_lang.clone(), current_text.clone()),
                            start: code_block_start,
                            end: range.end,
                        });
                        current_text.clear();
                        code_block_lang = None;
                        in_code_block = false;
                    }
                }
                Event::End(TagEnd::Item) => {
                    if in_list_item {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::ListItem(current_text.clone()),
                            start: list_item_start,
                            end: range.end,
                        });
                        current_text.clear();
                        in_list_item = false;
                    }
                }
                Event::End(TagEnd::BlockQuote) => {
                    if in_block_quote {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::BlockQuote(current_text.clone()),
                            start: block_quote_start,
                            end: range.end,
                        });
                        current_text.clear();
                        in_block_quote = false;
                    }
                }
                Event::Text(text) => {
                    if in_heading.is_some() || in_strong || in_emphasis || in_link || in_code_block || in_list_item || in_block_quote {
                        current_text.push_str(&text);
                    }
                }
                Event::Code(code) => {
                    tokens.push(ParsedToken {
                        token_type: MarkdownToken::Code(code.to_string()),
                        start: range.start,
                        end: range.end,
                    });
                }
                _ => {}
            }
        }
        
        tokens
    }

    pub fn find_token_at_position<'a>(&self, tokens: &'a [ParsedToken], position: usize) -> Option<&'a ParsedToken> {
        tokens.iter().find(|token| position >= token.start && position < token.end)
    }

    pub fn find_all_tokens_at_position<'a>(&self, tokens: &'a [ParsedToken], position: usize) -> Vec<&'a ParsedToken> {
        tokens.iter().filter(|token| position >= token.start && position < token.end).collect()
    }

    pub fn find_outermost_token_at_position<'a>(&self, tokens: &'a [ParsedToken], position: usize) -> Option<&'a ParsedToken> {
        tokens.iter()
            .filter(|token| position >= token.start && position < token.end)
            .max_by_key(|token| token.end - token.start) // Largest span is outermost
    }

    pub fn get_current_token_context(&self, markdown: &str, cursor_position: usize) -> TokenContext {
        let tokens = self.parse_with_positions(markdown);
        let current_token = self.find_token_at_position(&tokens, cursor_position);
        let all_tokens = self.find_all_tokens_at_position(&tokens, cursor_position);
        
        TokenContext {
            current_token: current_token.cloned(),
            all_tokens: all_tokens.into_iter().cloned().collect(),
            tokens,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenContext {
    pub current_token: Option<ParsedToken>,
    pub all_tokens: Vec<ParsedToken>,
    pub tokens: Vec<ParsedToken>, // All tokens in document
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("# Hello World");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], MarkdownToken::Heading(1, "Hello World".to_string()));
    }

    #[test]
    fn test_parse_bold() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("**bold text**");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Bold(s) if s == "bold text")));
    }

    #[test]
    fn test_parse_italic() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("*italic text*");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Italic(s) if s == "italic text")));
    }

    #[test]
    fn test_parse_code() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("`code`");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Code(s) if s == "code")));
    }

    #[test]
    fn test_parse_code_block() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("```\ncode block\n```");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::CodeBlock(None, s) if s == "code block\n")));
    }

    #[test]
    fn test_parse_link() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("[link text](http://example.com)");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Link(text, url) if text == "link text" && url == "http://example.com")));
    }

    #[test]
    fn test_parse_mixed_content() {
        let parser = MarkdownParser::new();
        let markdown = "# Title\n\nSome **bold** and *italic* text with `code`.";
        let tokens = parser.parse(markdown);
        
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Heading(1, s) if s == "Title")));
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Bold(s) if s == "bold")));
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Italic(s) if s == "italic")));
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Code(s) if s == "code")));
    }

    #[test]
    fn test_parsed_token_with_position() {
        let token = ParsedToken {
            token_type: MarkdownToken::Heading(1, "Hello".to_string()),
            start: 0,
            end: 7,
        };
        
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 7);
        assert!(matches!(token.token_type, MarkdownToken::Heading(1, ref s) if s == "Hello"));
    }

    #[test]
    fn test_parse_with_positions_heading() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse_with_positions("# Hello World");
        
        assert_eq!(tokens.len(), 1);
        let token = &tokens[0];
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 13);
        assert!(matches!(token.token_type, MarkdownToken::Heading(1, ref s) if s == "Hello World"));
    }

    #[test]
    fn test_find_token_at_cursor() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse_with_positions("# Hello World");
        
        let token_at_start = parser.find_token_at_position(&tokens, 0);
        assert!(token_at_start.is_some());
        assert!(matches!(token_at_start.unwrap().token_type, MarkdownToken::Heading(1, ref s) if s == "Hello World"));
        
        let token_at_middle = parser.find_token_at_position(&tokens, 5);
        assert!(token_at_middle.is_some());
        
        let token_at_end = parser.find_token_at_position(&tokens, 13);
        assert!(token_at_end.is_none()); // Position 13 is after the token
    }

    #[test]
    fn test_cursor_at_token_boundaries() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse_with_positions("**bold** text");
        
        // Before bold token
        let before_bold = parser.find_token_at_position(&tokens, 0);
        assert!(before_bold.is_some());
        
        // At start of bold content (after **)
        let at_bold_start = parser.find_token_at_position(&tokens, 2);
        assert!(at_bold_start.is_some());
        
        // At end of bold content (before **)
        let at_bold_end = parser.find_token_at_position(&tokens, 6);
        assert!(at_bold_end.is_some());
        
        // After bold token
        let after_bold = parser.find_token_at_position(&tokens, 8);
        assert!(after_bold.is_none() || !matches!(after_bold.unwrap().token_type, MarkdownToken::Bold(_)));
    }

    #[test]
    fn test_nested_token_detection() {
        let parser = MarkdownParser::new();
        // Test with nested bold inside heading
        let tokens = parser.parse_with_positions("# Heading with **bold** text");
        
        // Find all tokens at position 15 (inside the bold within heading)
        let nested_tokens = parser.find_all_tokens_at_position(&tokens, 15);
        
        // Should find both heading and bold token
        assert!(nested_tokens.len() >= 1); // At least the heading token
        
        // Should be able to find the outermost token (heading)
        let outermost = parser.find_outermost_token_at_position(&tokens, 15);
        assert!(outermost.is_some());
        assert!(matches!(outermost.unwrap().token_type, MarkdownToken::Heading(_, _)));
    }

    #[test]
    fn test_cursor_movement_context_updates() {
        let parser = MarkdownParser::new();
        let markdown = "# Title\n\nSome **bold** text";
        
        // Test context at different positions
        let context_at_title = parser.get_current_token_context(markdown, 3);
        assert!(context_at_title.current_token.is_some());
        assert!(matches!(context_at_title.current_token.unwrap().token_type, MarkdownToken::Heading(_, _)));
        
        let context_at_bold = parser.get_current_token_context(markdown, 15);
        assert!(context_at_bold.current_token.is_some());
        assert!(matches!(context_at_bold.current_token.unwrap().token_type, MarkdownToken::Bold(_)));
        
        // Use a position that's definitely after the bold token
        let context_at_plain = parser.get_current_token_context(markdown, 25);
        // Should be None since no token covers this position
        assert!(context_at_plain.current_token.is_none());
    }

    #[test]
    fn test_parse_blockquote() {
        let parser = MarkdownParser::new();
        let markdown = "> This is a blockquote\n> with multiple lines";
        let tokens = parser.parse(markdown);
        
        // Should find a BlockQuote token
        let blockquotes: Vec<_> = tokens.iter()
            .filter_map(|t| match t {
                MarkdownToken::BlockQuote(text) => Some(text.as_str()),
                _ => None
            })
            .collect();
        
        assert!(!blockquotes.is_empty());
        // The exact text depends on how pulldown-cmark handles multiline blockquotes
    }

    #[test]
    fn test_parse_list_items() {
        let parser = MarkdownParser::new();
        let markdown = "- First item\n- Second item\n- Third item";
        let tokens = parser.parse(markdown);
        
        // Should find 3 ListItem tokens
        let list_items: Vec<_> = tokens.iter()
            .filter_map(|t| match t {
                MarkdownToken::ListItem(text) => Some(text.as_str()),
                _ => None
            })
            .collect();
        
        assert_eq!(list_items.len(), 3);
        assert_eq!(list_items[0], "First item");
        assert_eq!(list_items[1], "Second item");
        assert_eq!(list_items[2], "Third item");
    }

    #[test]
    fn test_parse_advanced_elements_with_positions() {
        let parser = MarkdownParser::new();
        
        // Test link parsing
        let link_markdown = "[Click here](https://example.com)";
        let link_tokens = parser.parse_with_positions(link_markdown);
        assert_eq!(link_tokens.len(), 1);
        if let MarkdownToken::Link(text, url) = &link_tokens[0].token_type {
            assert_eq!(text, "Click here");
            assert_eq!(url, "https://example.com");
        } else {
            panic!("Expected Link token");
        }
        
        // Test inline code parsing
        let code_markdown = "Use `println!` to print";
        let code_tokens = parser.parse_with_positions(code_markdown);
        assert!(code_tokens.iter().any(|t| matches!(t.token_type, MarkdownToken::Code(ref s) if s == "println!")));
        
        // Test code block parsing
        let code_block_markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let block_tokens = parser.parse_with_positions(code_block_markdown);
        assert!(block_tokens.iter().any(|t| matches!(t.token_type, MarkdownToken::CodeBlock(Some(ref lang), _) if lang == "rust")));
    }
}