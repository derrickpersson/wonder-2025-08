# Wonder Editor MVP Implementation Plan

## Phase 1: Core Text Editor (Week 1)

### 1.1 Basic Text Input & Cursor Management
**TDD Approach**: Red → Green → Refactor

#### Test Cases:
- [ ] Insert character at cursor position
- [ ] Delete character with backspace
- [ ] Move cursor with arrow keys
- [ ] Handle cursor at beginning/end of document
- [ ] Track cursor position accurately

#### Implementation:
```rust
// src/text_buffer.rs
struct TextBuffer {
    content: String,
    cursor: usize,
}

impl TextBuffer {
    fn insert_char(&mut self, ch: char)
    fn delete_char(&mut self)
    fn move_cursor(&mut self, direction: Direction)
    fn get_cursor_position(&self) -> usize
}
```

### 1.2 Text Selection
#### Test Cases:
- [ ] Select text with Shift+arrows
- [ ] Select all with Cmd+A
- [ ] Replace selected text when typing
- [ ] Delete selected text with backspace
- [ ] Track selection range

### 1.3 Basic Keyboard Navigation
#### Test Cases:
- [ ] Home/End keys
- [ ] Page Up/Down
- [ ] Cmd+Left/Right (word boundaries)
- [ ] Cmd+Up/Down (document boundaries)

## Phase 2: Markdown Parser Integration (Week 2)

### 2.1 Parser Setup
#### Dependencies:
```toml
[dependencies]
pulldown-cmark = "0.10"
```

#### Test Cases:
- [ ] Parse simple markdown (headers, bold, italic)
- [ ] Generate token positions
- [ ] Map cursor position to tokens
- [ ] Handle malformed markdown gracefully

### 2.2 Token Position Tracking
#### Test Cases:
- [ ] Identify which token contains cursor
- [ ] Track token boundaries (start/end positions)
- [ ] Handle nested tokens correctly
- [ ] Update tokens on text changes

#### Implementation:
```rust
// src/markdown_parser.rs
struct ParsedToken {
    kind: TokenKind,
    range: Range<usize>,
    content: String,
}

struct DocumentTokens {
    tokens: Vec<ParsedToken>,
    cursor_token: Option<usize>,
}
```

### 2.3 Incremental Parsing
#### Test Cases:
- [ ] Re-parse only affected blocks on change
- [ ] Maintain token positions after edits
- [ ] Handle insertions/deletions efficiently
- [ ] Cache parsing results appropriately

## Phase 3: Hybrid Rendering System (Week 3)

### 3.1 Preview Renderer
#### Test Cases:
- [ ] Render headers with proper styling
- [ ] Render bold/italic text
- [ ] Render lists with bullets/numbers
- [ ] Render links as clickable elements
- [ ] Render code blocks with monospace font

#### Implementation:
```rust
// src/renderer/preview.rs
impl PreviewRenderer {
    fn render_token(&self, token: &ParsedToken) -> Element
    fn render_header(&self, level: u8, text: &str) -> Element
    fn render_emphasis(&self, text: &str) -> Element
    fn render_list_item(&self, text: &str) -> Element
}
```

### 3.2 Raw Mode Renderer
#### Test Cases:
- [ ] Show markdown syntax characters
- [ ] Preserve exact spacing and formatting
- [ ] Handle cursor visibility in raw mode
- [ ] Switch seamlessly between modes

### 3.3 Mode Switching Logic
#### Test Cases:
- [ ] Switch to raw when cursor enters token
- [ ] Switch to preview when cursor leaves token
- [ ] Handle selection spanning multiple tokens
- [ ] Debounce rapid cursor movements

## Phase 4: File Operations (Week 4)

### 4.1 File Loading
#### Test Cases:
- [ ] Open .md, .markdown, .txt files
- [ ] Handle file read errors gracefully
- [ ] Load large files efficiently
- [ ] Detect file encoding

#### Implementation:
```rust
// src/file_manager.rs
impl FileManager {
    fn open_file(&mut self, path: PathBuf) -> Result<String>
    fn save_file(&mut self, path: PathBuf, content: &str) -> Result<()>
    fn create_new_file(&mut self) -> Result<()>
}
```

### 4.2 File Saving
#### Test Cases:
- [ ] Save current content to file
- [ ] Handle save errors (permissions, disk space)
- [ ] Save As functionality
- [ ] Track unsaved changes

### 4.3 Unsaved Changes Detection
#### Test Cases:
- [ ] Track document dirty state
- [ ] Prompt before losing changes
- [ ] Handle file conflicts on save

## Phase 5: Basic Formatting (Week 5)

### 5.1 Bold/Italic Shortcuts
#### Test Cases:
- [ ] Cmd+B wraps selection with **
- [ ] Cmd+I wraps selection with *
- [ ] Toggle existing formatting
- [ ] Insert markers when no selection
- [ ] Handle partial selections

#### Implementation:
```rust
// src/formatting.rs
impl Formatter {
    fn apply_bold(&mut self, buffer: &mut TextBuffer)
    fn apply_italic(&mut self, buffer: &mut TextBuffer)
    fn toggle_formatting(&mut self, markers: &str)
}
```

### 5.2 List Handling
#### Test Cases:
- [ ] Enter creates new list item
- [ ] Auto-increment numbered lists
- [ ] Exit list on empty item + Enter
- [ ] Tab/Shift+Tab for indentation

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_char_at_cursor() {
        let mut buffer = TextBuffer::new("hello");
        buffer.set_cursor(2);
        buffer.insert_char('x');
        assert_eq!(buffer.content(), "hexllo");
        assert_eq!(buffer.cursor(), 3);
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
#[test]
fn test_bold_formatting_workflow() {
    let mut editor = MarkdownEditor::new();
    editor.insert_text("hello world");
    editor.select_range(0..5);
    editor.apply_bold();
    assert_eq!(editor.content(), "**hello** world");
}
```

### Performance Benchmarks
```rust
#[cfg(test)]
mod benchmarks {
    #[test]
    fn bench_large_document_parsing() {
        // Test 1MB document parsing under 16ms
    }
}
```

## Development Workflow

### TDD Cycle for Each Feature:
1. **Red**: Write failing test first
2. **Green**: Implement minimal code to pass
3. **Refactor**: Clean up while keeping tests green
4. **Commit**: Small, focused commits

### Example TDD Session:
```rust
// 1. RED - Write failing test
#[test]
fn test_cursor_movement_right() {
    let mut buffer = TextBuffer::new("hello");
    buffer.set_cursor(0);
    buffer.move_cursor_right();
    assert_eq!(buffer.cursor(), 1); // FAILS - not implemented
}

// 2. GREEN - Minimal implementation
impl TextBuffer {
    fn move_cursor_right(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor += 1;
        }
    }
}

// 3. REFACTOR - Improve code quality
impl TextBuffer {
    fn move_cursor_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.content.len());
    }
}
```

## Milestones

### Week 1 Milestone: Basic Editor
- Can type and edit text
- Cursor navigation works
- Text selection functional

### Week 2 Milestone: Parser Integration  
- Markdown parsing working
- Token identification complete
- Cursor-to-token mapping functional

### Week 3 Milestone: Hybrid Rendering
- Preview mode renders basic markdown
- Raw mode shows syntax
- Mode switching operational

### Week 4 Milestone: File Operations
- Can open/save files
- Unsaved changes detection
- File format support

### Week 5 Milestone: MVP Complete
- Basic formatting shortcuts
- List handling
- Ready for user testing

## Next Steps After MVP
1. Interactive elements (checkboxes, links)
2. Undo/redo system
3. Performance optimizations
4. Advanced formatting features