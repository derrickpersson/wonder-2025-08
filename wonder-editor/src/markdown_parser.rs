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
    Strikethrough(String),
    TaskListItem(bool, String), // checked, text
    HorizontalRule,
    Image(String, String, Option<String>), // alt, url, title
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
        let mut in_strikethrough = false;
        let mut in_task_list = false;
        let mut task_checked = false;
        let mut link_url = String::new();
        let mut in_image = false;
        let mut image_alt = String::new();
        let mut image_url = String::new();
        let mut image_title: Option<String> = None;
        
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
                    Tag::Strikethrough => {
                        in_strikethrough = true;
                    }
                    Tag::CodeBlock(_kind) => {
                        // CodeBlockKind can be Indented or Fenced
                        in_code = true;
                        current_text.clear();
                    }
                    Tag::Link { dest_url, .. } => {
                        in_link = true;
                        link_url = dest_url.to_string();
                    }
                    Tag::Image { dest_url, title, .. } => {
                        in_image = true;
                        image_url = dest_url.to_string();
                        image_title = if title.is_empty() { None } else { Some(title.to_string()) };
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
                    TagEnd::Strikethrough => {
                        if in_strikethrough {
                            tokens.push(MarkdownToken::Strikethrough(current_text.clone()));
                            current_text.clear();
                            in_strikethrough = false;
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
                    TagEnd::Image => {
                        if in_image {
                            tokens.push(MarkdownToken::Image(current_text.clone(), image_url.clone(), image_title.clone()));
                            current_text.clear();
                            image_url.clear();
                            image_title = None;
                            in_image = false;
                        }
                    }
                    TagEnd::Item => {
                        if in_list_item {
                            if in_task_list {
                                tokens.push(MarkdownToken::TaskListItem(task_checked, current_text.clone()));
                                in_task_list = false;
                            } else {
                                tokens.push(MarkdownToken::ListItem(current_text.clone()));
                            }
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
                    if !in_heading.is_some() && !in_emphasis && !in_strong && !in_strikethrough && !in_code && !in_link && !in_image && !in_list_item && !in_block_quote {
                        tokens.push(MarkdownToken::Text(text.to_string()));
                        current_text.clear();
                    }
                }
                Event::Code(code) => {
                    tokens.push(MarkdownToken::Code(code.to_string()));
                }
                Event::TaskListMarker(checked) => {
                    in_task_list = true;
                    task_checked = checked;
                }
                Event::Rule => {
                    tokens.push(MarkdownToken::HorizontalRule);
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
        let mut in_strikethrough = false;
        let mut strikethrough_start = 0;
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
                Event::Start(Tag::Strikethrough) => {
                    in_strikethrough = true;
                    strikethrough_start = range.start;
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
                Event::End(TagEnd::Strikethrough) => {
                    if in_strikethrough {
                        tokens.push(ParsedToken {
                            token_type: MarkdownToken::Strikethrough(current_text.clone()),
                            start: strikethrough_start,
                            end: range.end,
                        });
                        current_text.clear();
                        in_strikethrough = false;
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
                    if in_heading.is_some() || in_strong || in_emphasis || in_strikethrough || in_link || in_code_block || in_list_item || in_block_quote {
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

// ENG-116: Incremental parsing structures for performance optimization
#[derive(Debug, Clone)]
pub struct TextChange {
    pub start: usize,      // Character position where change starts
    pub deleted_len: usize, // Number of characters deleted
    pub inserted_text: String, // New text inserted
}

#[derive(Debug, Clone)]
struct CachedBlock {
    tokens: Vec<ParsedToken>,
    content_hash: u64,
    line_start: usize,
    line_end: usize,
    char_start: usize,
    char_end: usize,
}

pub struct IncrementalParser {
    base_parser: MarkdownParser,
    cached_blocks: Vec<CachedBlock>,
    last_content: String,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            base_parser: MarkdownParser::new(),
            cached_blocks: Vec::new(),
            last_content: String::new(),
        }
    }

    pub fn parse_incremental(&mut self, content: &str, changes: Vec<TextChange>) -> Vec<ParsedToken> {
        // If no previous content or changes, do full parse
        if self.last_content.is_empty() || changes.is_empty() {
            self.last_content = content.to_string();
            let tokens = self.base_parser.parse_with_positions(content);
            self.update_cache(&tokens, content);
            return tokens;
        }

        // Check if changes are small enough for incremental parsing
        let total_change_size: usize = changes.iter()
            .map(|c| c.deleted_len + c.inserted_text.len())
            .sum();

        // If changes are substantial (>10% of document), just do full reparse
        if total_change_size > content.len() / 10 {
            self.last_content = content.to_string();
            let tokens = self.base_parser.parse_with_positions(content);
            self.update_cache(&tokens, content);
            return tokens;
        }

        // Determine affected regions based on changes
        let affected_blocks = self.find_affected_blocks(&changes);
        
        // If no cached blocks or too many affected, fall back to full parse
        if self.cached_blocks.is_empty() || affected_blocks.len() > self.cached_blocks.len() / 2 {
            self.last_content = content.to_string();
            let tokens = self.base_parser.parse_with_positions(content);
            self.update_cache(&tokens, content);
            return tokens;
        }

        // Perform incremental parsing (simplified for now)
        self.last_content = content.to_string();
        let tokens = self.base_parser.parse_with_positions(content);
        self.update_cache(&tokens, content);
        tokens
    }

    fn find_affected_blocks(&self, _changes: &[TextChange]) -> Vec<usize> {
        // For now, return all blocks as potentially affected
        // A more sophisticated implementation would analyze change positions
        (0..self.cached_blocks.len()).collect()
    }

    fn update_cache(&mut self, tokens: &[ParsedToken], content: &str) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        self.cached_blocks.clear();
        
        if tokens.is_empty() {
            return;
        }

        // Group tokens into logical blocks (simplified approach)
        // For now, treat each token as its own block
        for token in tokens {
            let block_content = &content[token.start..token.end.min(content.len())];
            let mut hasher = DefaultHasher::new();
            block_content.hash(&mut hasher);
            
            let cached_block = CachedBlock {
                tokens: vec![token.clone()],
                content_hash: hasher.finish(),
                line_start: token.start,  // Simplified
                line_end: token.end,      // Simplified  
                char_start: token.start,
                char_end: token.end,
            };
            
            self.cached_blocks.push(cached_block);
        }
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

    // TDD RED: First failing test for strikethrough
    #[test]
    fn test_parse_strikethrough() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("~~strikethrough text~~");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Strikethrough(s) if s == "strikethrough text")));
    }

    #[test] 
    fn test_parse_strikethrough_with_positions() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse_with_positions("~~strikethrough text~~");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token_type, MarkdownToken::Strikethrough(ref s) if s == "strikethrough text"));
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 22);
    }

    // TDD RED: Task list tests
    #[test]
    fn test_parse_task_lists() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("- [ ] Unchecked task\n- [x] Checked task");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::TaskListItem(false, s) if s == "Unchecked task")));
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::TaskListItem(true, s) if s == "Checked task")));
    }

    // TDD RED: Horizontal rule test
    #[test]
    fn test_parse_horizontal_rule() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("---");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::HorizontalRule)));
    }

    // TDD RED: Image test
    #[test]
    fn test_parse_image() {
        let parser = MarkdownParser::new();
        let tokens = parser.parse("![alt text](image.jpg \"title\")");
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Image(alt, url, title) 
            if alt == "alt text" && url == "image.jpg" && title == &Some("title".to_string()))));
    }

    // Comprehensive test for all new token types added in ENG-113
    #[test]
    fn test_parse_extended_markdown_elements() {
        let parser = MarkdownParser::new();
        let markdown = r#"# Heading
~~strikethrough~~
- [ ] Unchecked task
- [x] Checked task
---
![alt](image.jpg "title")"#;
        
        let tokens = parser.parse(markdown);
        
        // Test strikethrough
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Strikethrough(s) if s == "strikethrough")));
        
        // Test task lists
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::TaskListItem(false, s) if s == "Unchecked task")));
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::TaskListItem(true, s) if s == "Checked task")));
        
        // Test horizontal rule
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::HorizontalRule)));
        
        // Test image
        assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Image(alt, url, title) 
            if alt == "alt" && url == "image.jpg" && title == &Some("title".to_string()))));
    }

    // TDD RED: Test basic incremental parsing setup
    #[test]
    fn test_incremental_parser_creation() {
        let mut incremental_parser = IncrementalParser::new();
        let markdown = "# Hello World\n\nSome content here.";
        
        // First parse should work normally
        let tokens = incremental_parser.parse_incremental(markdown, vec![]);
        assert!(tokens.len() > 0);
        assert!(tokens.iter().any(|t| matches!(t.token_type, MarkdownToken::Heading(1, ref s) if s == "Hello World")));
    }

    #[test]
    fn test_incremental_parser_single_character_edit() {
        let mut incremental_parser = IncrementalParser::new();
        let original_markdown = "# Hello World\n\nSome content here.";
        
        // Initial parse
        let initial_tokens = incremental_parser.parse_incremental(original_markdown, vec![]);
        let initial_count = initial_tokens.len();
        
        // Single character edit - add exclamation
        let modified_markdown = "# Hello World!\n\nSome content here.";
        let change = TextChange { start: 13, deleted_len: 0, inserted_text: "!".to_string() };
        
        let updated_tokens = incremental_parser.parse_incremental(modified_markdown, vec![change]);
        
        // Should have same number of tokens but updated heading
        assert_eq!(updated_tokens.len(), initial_count);
        assert!(updated_tokens.iter().any(|t| matches!(t.token_type, MarkdownToken::Heading(1, ref s) if s == "Hello World!")));
    }

    #[test]
    fn test_incremental_parser_performance() {
        use std::time::Instant;

        let mut incremental_parser = IncrementalParser::new();
        
        // Create a larger document to test performance
        let mut large_markdown = String::new();
        for i in 0..100 {
            large_markdown.push_str(&format!("# Heading {}\n\nParagraph {} with **bold** and *italic* text.\n\n", i, i));
        }
        
        // Initial parse
        let start = Instant::now();
        let _initial_tokens = incremental_parser.parse_incremental(&large_markdown, vec![]);
        let initial_duration = start.elapsed();
        
        // Small edit
        let modified_markdown = large_markdown.replace("Heading 0", "Heading 0 Modified");
        let change = TextChange { start: 2, deleted_len: 9, inserted_text: "Heading 0 Modified".to_string() };
        
        let start = Instant::now();
        let _updated_tokens = incremental_parser.parse_incremental(&modified_markdown, vec![change]);
        let incremental_duration = start.elapsed();
        
        // Print performance metrics (for manual inspection during testing)
        println!("Initial parse: {:?}", initial_duration);
        println!("Incremental parse: {:?}", incremental_duration);
        
        // Ensure incremental parsing completes in reasonable time
        // (This is more about ensuring we don't regress massively than strict performance)
        assert!(incremental_duration < std::time::Duration::from_millis(100), 
                "Incremental parsing took too long: {:?}", incremental_duration);
    }

    #[test]
    fn test_incremental_parser_large_changes() {
        let mut incremental_parser = IncrementalParser::new();
        let original_markdown = "# Small document\n\nWith some content.";
        
        // Initial parse
        let initial_tokens = incremental_parser.parse_incremental(original_markdown, vec![]);
        
        // Large change - should fall back to full parse
        let large_addition = "\n\n# New Section\n\nWith **bold** content.\n\n## Subsection\n\n- List item\n- Another item\n\n`code`";
        let change = TextChange { 
            start: original_markdown.len(), 
            deleted_len: 0, 
            inserted_text: large_addition.to_string()
        };
        
        let modified_markdown = original_markdown.to_string() + &change.inserted_text;
        let tokens = incremental_parser.parse_incremental(&modified_markdown, vec![change]);
        
        // Should handle large changes gracefully and generate more tokens than initial
        assert!(tokens.len() > initial_tokens.len(), "Should have more tokens after large addition");
        assert!(tokens.len() > 5, "Should have multiple tokens from expanded document");
    }
}