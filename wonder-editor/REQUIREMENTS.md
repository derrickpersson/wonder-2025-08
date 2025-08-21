# Wonder Editor Requirements

## Overview
A markdown editor inspired by Obsidian's editing experience, supporting the GitHub Flavored Markdown (GFM) specification with live preview capabilities.

## Core Features

### 1. Markdown Parsing & Rendering
- **Support GFM Spec**: Full compliance with GitHub Flavored Markdown
- **Token-Based Rendering**: 
  - Render all tokens before and after cursor as formatted preview
  - Show ONLY the current token containing the cursor as raw markdown
  - When text is selected, show entire selection as raw markdown
- **Smart Cursor Context**: Detect which token the cursor is within

### 2. File Operations
- **Open File**: Load existing markdown files from disk
- **Create New File**: Initialize blank documents
- **Save File**: Write content to disk (with save/save-as functionality)
- **Close File**: Properly handle file closure with unsaved changes warning
- **Single File Focus**: One file editing at a time (no tabs initially)
- **Supported Formats**: `.md`, `.markdown`, `.txt` files

### 3. Editor Shortcuts
- **Bold**: Cmd/Ctrl + B - Wraps selection with `**`, toggles if already bold, inserts `****` with cursor in middle if no selection
- **Italic**: Cmd/Ctrl + I - Wraps selection with `*`, toggles if already italic, inserts `**` with cursor in middle if no selection  
- **Code**: Cmd/Ctrl + ` - Wraps selection with backticks, toggles if already code
- **Link**: Cmd/Ctrl + K - Creates link with selected text as label
- **Heading**: Cmd/Ctrl + 1-6 - Converts current line to corresponding heading level
- **List**: Cmd/Ctrl + L - Converts line to list item
- **Task List**: Cmd/Ctrl + Shift + L - Converts line to task list item
- **Blockquote**: Cmd/Ctrl + > - Converts line to blockquote

## Implementation Subtasks

### Phase 1: Core Editor Infrastructure
- [ ] Implement basic text input and cursor management
- [ ] Add text selection support
- [ ] Implement undo/redo functionality
- [ ] Add basic keyboard navigation (arrow keys, home/end, etc.)

### Phase 2: Markdown Parser Integration
- [ ] Integrate a GFM-compliant markdown parser (e.g., pulldown-cmark)
- [ ] Create token position tracking system
- [ ] Implement cursor-to-token mapping
- [ ] Build AST representation of document

### Phase 3: Hybrid Rendering System
- [ ] Create preview renderer for parsed tokens
- [ ] Implement edit mode renderer for current line/block
- [ ] Build seamless transition between preview and edit modes
- [ ] Handle edge cases (multi-line blocks, nested elements)

### Phase 4: File Operations
- [ ] Implement file open dialog
- [ ] Add file save functionality
- [ ] Create new file workflow
- [ ] Add unsaved changes detection
- [ ] Implement file close with confirmation

### Phase 5: Markdown Elements Support
- [ ] Headers (H1-H6)
- [ ] Bold and Italic text
- [ ] Code blocks and inline code
- [ ] Lists (ordered and unordered)
- [ ] Task lists with checkboxes
- [ ] Links
- [ ] Images
- [ ] Blockquotes
- [ ] Tables
- [ ] Horizontal rules

### Phase 6: Keyboard Shortcuts
- [ ] Implement shortcut system
- [ ] Add formatting shortcuts
- [ ] Create navigation shortcuts
- [ ] Support customizable keybindings

### Phase 7: UI Polish
- [ ] Add toolbar with formatting buttons
- [ ] Implement status bar (cursor position, word count)
- [ ] Create settings/preferences panel
- [ ] Add find/replace functionality

## Technical Considerations

### Parser Library
- Use `pulldown-cmark` for GFM compliance
- Consider `comrak` as alternative for more extensions

### Rendering Strategy
- Use GPUI's immediate mode rendering
- Maintain virtual DOM-like structure for efficient updates
- Cache rendered elements when possible

### Performance Goals
- Handle files up to 1MB smoothly
- Instant preview updates (< 16ms)
- Responsive typing with no lag

## Testing Strategy
- Unit tests for each markdown element
- Integration tests for file operations
- Performance benchmarks for large files
- UI tests for keyboard shortcuts

## Future Enhancements (Post-MVP)
- Multiple file tabs
- File tree/explorer
- Plugin system
- Theme support
- Export to various formats (PDF, HTML)
- Live collaboration
- Syntax highlighting for code blocks
- Math equation support (LaTeX)
- Mermaid diagram support