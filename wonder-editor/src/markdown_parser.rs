use pulldown_cmark::{Parser, Event, Tag, TagEnd, Options};

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
    Text(String),
}

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
                    _ => {}
                },
                Event::Text(text) => {
                    current_text.push_str(&text);
                    if !in_heading.is_some() && !in_emphasis && !in_strong && !in_code && !in_link {
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
}