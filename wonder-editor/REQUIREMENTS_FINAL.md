# Wonder Editor - Final Requirements Specification

## Overview
A markdown editor inspired by Obsidian's editing experience, supporting GitHub Flavored Markdown (GFM) with token-based hybrid preview capabilities.

## Core Behavior

### Token-Based Hybrid Rendering
- **Preview Mode**: Render all tokens as formatted content by default
- **Edit Mode**: Show raw markdown ONLY for:
  - The current token containing the cursor
  - Entire selection when text is selected
  - For nested tokens (e.g., `**bold _italic_ text**`): show entire outer token as raw
  - For long paragraphs: optimize to show in preview mode unless actively editing
- **Transitions**: 
  - Switch to raw mode immediately on keystroke
  - Switch back to preview when cursor leaves token
  - Debounce preview updates to prevent flickering
- **Special Cases**:
  - Multi-line tokens (code blocks): show entire block as raw when cursor inside
  - Block vs inline tokens: can preview more of block tokens, less of inline tokens

### Interactive Elements
- **Task Lists**: 
  - Clicking checkbox toggles state ([ ] ↔ [x])
  - Cursor stays at current position
  - Animate checkbox transitions
- **Links**: 
  - Require Cmd/Ctrl+Click to follow
  - Show URL tooltip on hover
  - External links open in default browser
  - Internal links deferred for later
- **Images**:
  - Support both local paths and URLs
  - Display at original size with maximum size limits
  - Show alt text for broken images
  - Lazy loading for performance
  - Retry mechanism for failed loads

## File Operations
- **Supported Formats**: `.md`, `.markdown`, `.txt`
- **New File**: Create blank documents
- **Open File**: Load from disk with unsaved changes prompt
- **Save/Save As**: Save As switches to editing new file
- **Recent Files**: Remember last 10 files
- **Single File**: One file at a time (no tabs)

## Editor Features

### Text Editing
- **Undo/Redo**: 
  - Group intelligently (word boundaries)
  - 200 undo levels
  - Formatting changes as separate actions
  - Reset history on file switch
- **Selections**: Apply formatting to entire selection
- **Lists**:
  - Enter creates next list item
  - Auto-increment numbered lists
  - Empty item + Enter exits list
  - Tab/Shift+Tab for indentation

### Keyboard Shortcuts
- **Bold**: Cmd/Ctrl+B - wrap with `**`, toggle if already bold, insert `****` if no selection
- **Italic**: Cmd/Ctrl+I - wrap with `*`, toggle if already italic, insert `**` if no selection
- **Code**: Cmd/Ctrl+` - wrap with backticks, toggle existing
- **Headings**: Cmd/Ctrl+1-6 - convert line(s) to heading, replace existing level
- **Lists**: Cmd/Ctrl+L - convert to list item
- **Task Lists**: Cmd/Ctrl+Shift+L - convert to task item
- **Blockquote**: Cmd/Ctrl+> - convert to blockquote
- **Links**: Cmd/Ctrl+K - create link from selection

### Advanced Features (Post-MVP)
- **Tables**:
  - Always show in preview mode
  - Tab/Shift+Tab navigation between cells
  - Tab in last cell creates new row
  - Shift+Tab in first cell stays put
  - Enhanced editing with row/column operations
  - Tab in list within cell indents list (not cell navigation)
- **Find/Replace**:
  - Highlight current match differently from others
  - Work on raw markdown text
  - Regex support
  - Replace as single undoable action
- **Code Blocks**:
  - Auto-detect language if not specified
  - Syntax highlighting for popular languages (JS, TS, Python, Ruby, Rust, etc.)
  - No line numbers
- **Vim Mode**:
  - Basic navigation (hjkl)
  - Optional in settings

## Technical Architecture

### Parsing System
- **Parser**: Use `pulldown-cmark` for GFM compliance
- **Incremental Parsing**: Parse in chunks/blocks
- **Optimization**: Prioritize visible content, background parse rest
- **Cache Management**: Research optimal invalidation strategies

### Performance
- **Target**: 1MB files smoothly
- **Rendering**: Visible area + buffer only
- **Updates**: < 16ms preview updates
- **Images**: Lazy loading

### Settings
- **Storage**: Global config file
- **Options**: Theme, vim mode, font size
- **Updates**: Apply immediately on save

### UI/UX
- **Interface**: Minimal, no toolbar initially
- **Menu**: Use macOS native menu bar
- **Status Bar**: None initially
- **Views**: No split view, hybrid only

## MVP Scope
**Phase 1 - Core Editor**:
1. Basic text editing with cursor management
2. Token-based preview rendering system
3. File operations (new, open, save)
4. Core GFM parsing integration

**Phase 2 - Essential Features**:
1. Basic formatting shortcuts (bold, italic)
2. List handling and navigation
3. Undo/redo system
4. Interactive elements (checkboxes, links)

**Later Phases**:
- Tables, find/replace, advanced shortcuts
- Code syntax highlighting
- Vim mode, settings, recent files
- Performance optimizations

## Error Handling
- **Malformed Markdown**: Render what's parseable, show rest as raw
- **Unsupported Features**: Fall back to raw text
- **Missing Images**: Show alt text with retry mechanism
- **Parse Errors**: Continue rendering, no error display to user