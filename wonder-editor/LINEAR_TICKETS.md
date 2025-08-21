# Wonder Editor - Linear Tickets

## Epic: Phase 1 - Core Text Editor

### WE-001: Text Buffer Foundation
**Title**: Implement basic text buffer with cursor management
**Description**: Create the core TextBuffer struct that handles text content and cursor position tracking.
**Acceptance Criteria**:
- [ ] TextBuffer can store and modify text content
- [ ] Cursor position is tracked accurately
- [ ] Insert character at cursor position
- [ ] Delete character with backspace
- [ ] Handle cursor at document boundaries
**Labels**: Core, Foundation
**Estimate**: 2 points

### WE-002: Cursor Navigation
**Title**: Implement cursor movement with arrow keys
**Description**: Add keyboard navigation for cursor movement in all directions.
**Acceptance Criteria**:
- [ ] Arrow keys move cursor up/down/left/right
- [ ] Handle line boundaries correctly
- [ ] Cursor position updates accurately
- [ ] No crashes on edge cases (empty document, end of lines)
**Labels**: Core, Navigation
**Estimate**: 3 points

### WE-003: Advanced Navigation
**Title**: Implement Home/End and word-boundary navigation
**Description**: Add advanced keyboard navigation shortcuts.
**Acceptance Criteria**:
- [ ] Home/End keys work correctly
- [ ] Cmd+Left/Right for word boundaries
- [ ] Cmd+Up/Down for document boundaries
- [ ] Page Up/Down navigation
**Labels**: Core, Navigation
**Estimate**: 2 points

### WE-004: Text Selection
**Title**: Implement text selection with keyboard
**Description**: Add text selection capabilities using Shift+navigation keys.
**Acceptance Criteria**:
- [ ] Shift+arrows select text
- [ ] Cmd+A selects all text
- [ ] Selection range is tracked accurately
- [ ] Visual selection highlighting works
**Labels**: Core, Selection
**Estimate**: 3 points

### WE-005: Selection Operations
**Title**: Implement operations on selected text
**Description**: Handle typing and deletion with active text selection.
**Acceptance Criteria**:
- [ ] Typing replaces selected text
- [ ] Backspace/Delete removes selected text
- [ ] Selection is cleared after operations
- [ ] Cursor positioned correctly after operations
**Labels**: Core, Selection
**Estimate**: 2 points

## Epic: Phase 2 - Markdown Parser Integration

### WE-006: Markdown Parser Setup
**Title**: Integrate pulldown-cmark parser
**Description**: Set up basic markdown parsing functionality using pulldown-cmark.
**Acceptance Criteria**:
- [ ] Parse markdown to events/tokens
- [ ] Handle headers, bold, italic, lists
- [ ] Parse code blocks and links
- [ ] Handle malformed markdown gracefully
**Labels**: Parser, Foundation
**Estimate**: 3 points

### WE-007: Token Position Tracking
**Title**: Map tokens to document positions
**Description**: Create system to track which tokens exist at which character positions.
**Acceptance Criteria**:
- [ ] ParsedToken struct with range information
- [ ] Map cursor position to containing token
- [ ] Handle nested tokens correctly
- [ ] Track token boundaries accurately
**Labels**: Parser, Core
**Estimate**: 5 points

### WE-008: Cursor-Token Mapping
**Title**: Determine which token contains the cursor
**Description**: Implement logic to identify the current token based on cursor position.
**Acceptance Criteria**:
- [ ] Identify token containing cursor
- [ ] Handle cursor at token boundaries
- [ ] Support nested token detection
- [ ] Update mapping on cursor movement
**Labels**: Parser, Core
**Estimate**: 3 points

### WE-009: Incremental Parsing
**Title**: Implement efficient re-parsing on text changes
**Description**: Optimize parsing to only re-parse affected document sections.
**Acceptance Criteria**:
- [ ] Parse in blocks/chunks rather than full document
- [ ] Re-parse only affected sections on changes
- [ ] Maintain token positions after edits
- [ ] Cache parsing results where possible
**Labels**: Parser, Performance
**Estimate**: 8 points

## Epic: Phase 3 - Hybrid Rendering System

### WE-010: Preview Renderer Foundation
**Title**: Create basic preview rendering for markdown tokens
**Description**: Implement rendering system that displays formatted markdown.
**Acceptance Criteria**:
- [ ] Render headers with proper typography
- [ ] Render bold/italic text styling
- [ ] Render lists with bullets/numbers
- [ ] Use GPUI styling system
**Labels**: Rendering, UI
**Estimate**: 5 points

### WE-011: Preview Renderer - Advanced Elements
**Title**: Extend preview renderer for links and code blocks
**Description**: Add rendering support for more complex markdown elements.
**Acceptance Criteria**:
- [ ] Render links as styled, clickable elements
- [ ] Render code blocks with monospace font
- [ ] Render blockquotes with styling
- [ ] Handle inline code styling
**Labels**: Rendering, UI
**Estimate**: 3 points

### WE-012: Raw Mode Renderer
**Title**: Implement raw markdown display mode
**Description**: Create renderer that shows markdown syntax characters.
**Acceptance Criteria**:
- [ ] Display markdown syntax characters
- [ ] Preserve exact spacing and formatting
- [ ] Handle cursor visibility in raw mode
- [ ] Maintain consistent font/styling
**Labels**: Rendering, UI
**Estimate**: 2 points

### WE-013: Mode Switching Logic
**Title**: Implement hybrid preview/raw mode switching
**Description**: Create logic to switch between preview and raw modes based on cursor position.
**Acceptance Criteria**:
- [ ] Switch to raw when cursor enters token
- [ ] Switch to preview when cursor leaves token
- [ ] Handle selection spanning multiple tokens
- [ ] Debounce rapid cursor movements
**Labels**: Core, Rendering
**Estimate**: 5 points

### WE-014: Nested Token Handling
**Title**: Handle nested markdown tokens in hybrid mode
**Description**: Implement correct behavior for nested tokens like bold within italic.
**Acceptance Criteria**:
- [ ] Show entire outer token as raw when cursor inside
- [ ] Handle deeply nested structures
- [ ] Maintain performance with complex nesting
- [ ] Correct visual transitions
**Labels**: Rendering, Core
**Estimate**: 3 points

## Epic: Phase 4 - File Operations

### WE-015: File Loading
**Title**: Implement file open functionality
**Description**: Add ability to load markdown files from disk.
**Acceptance Criteria**:
- [ ] Open .md, .markdown, .txt files
- [ ] Handle file read errors gracefully
- [ ] Support different text encodings
- [ ] Load content into text buffer
**Labels**: File Operations
**Estimate**: 3 points

### WE-016: File Saving
**Title**: Implement file save functionality
**Description**: Add ability to save current content to disk.
**Acceptance Criteria**:
- [ ] Save current content to file
- [ ] Handle save errors (permissions, disk space)
- [ ] Save As functionality
- [ ] Update window title with filename
**Labels**: File Operations
**Estimate**: 2 points

### WE-017: Unsaved Changes Detection
**Title**: Track and warn about unsaved changes
**Description**: Implement dirty state tracking and user warnings.
**Acceptance Criteria**:
- [ ] Track document dirty state on changes
- [ ] Prompt before losing changes
- [ ] Visual indicator of unsaved changes
- [ ] Handle file switching with unsaved changes
**Labels**: File Operations, UX
**Estimate**: 3 points

### WE-018: File Menu Integration
**Title**: Integrate file operations with macOS menu
**Description**: Wire up file operations to native macOS menu items.
**Acceptance Criteria**:
- [ ] New, Open, Save, Save As menu items
- [ ] Keyboard shortcuts (Cmd+N, Cmd+O, Cmd+S)
- [ ] Recent files submenu
- [ ] Proper menu state management
**Labels**: File Operations, UI
**Estimate**: 3 points

## Epic: Phase 5 - Basic Formatting

### WE-019: Bold/Italic Shortcuts
**Title**: Implement Cmd+B and Cmd+I formatting shortcuts
**Description**: Add keyboard shortcuts for basic text formatting.
**Acceptance Criteria**:
- [ ] Cmd+B wraps selection with **
- [ ] Cmd+I wraps selection with *
- [ ] Toggle existing formatting when already applied
- [ ] Insert markers when no selection
**Labels**: Formatting, Shortcuts
**Estimate**: 3 points

### WE-020: Format Toggle Logic
**Title**: Implement smart formatting toggle behavior
**Description**: Handle toggling of existing formatting and mixed selections.
**Acceptance Criteria**:
- [ ] Detect existing formatting in selection
- [ ] Remove formatting when toggling off
- [ ] Handle partial selections intelligently
- [ ] Apply formatting to entire selection consistently
**Labels**: Formatting, Core
**Estimate**: 5 points

### WE-021: List Auto-formatting
**Title**: Implement automatic list continuation
**Description**: Add smart behavior for working with markdown lists.
**Acceptance Criteria**:
- [ ] Enter creates new list item
- [ ] Auto-increment numbered lists
- [ ] Exit list on empty item + Enter
- [ ] Maintain list indentation levels
**Labels**: Lists, Formatting
**Estimate**: 4 points

### WE-022: List Indentation
**Title**: Implement Tab/Shift+Tab for list indentation
**Description**: Add keyboard shortcuts for list nesting.
**Acceptance Criteria**:
- [ ] Tab indents list items
- [ ] Shift+Tab unindents list items
- [ ] Handle nested list structures
- [ ] Maintain proper markdown formatting
**Labels**: Lists, Shortcuts
**Estimate**: 3 points

## Epic: MVP Testing & Polish

### WE-023: Integration Testing
**Title**: Create comprehensive integration tests
**Description**: Ensure all components work together correctly.
**Acceptance Criteria**:
- [ ] Full editing workflow tests
- [ ] File operations integration tests
- [ ] Rendering mode switching tests
- [ ] Performance benchmarks for large files
**Labels**: Testing, Quality
**Estimate**: 5 points

### WE-024: Error Handling & Edge Cases
**Title**: Robust error handling and edge case management
**Description**: Ensure application handles errors and edge cases gracefully.
**Acceptance Criteria**:
- [ ] Handle malformed markdown gracefully
- [ ] Recover from file operation errors
- [ ] Manage memory efficiently with large files
- [ ] Prevent crashes on invalid input
**Labels**: Quality, Robustness
**Estimate**: 3 points

### WE-025: Performance Optimization
**Title**: Optimize performance for target file sizes
**Description**: Ensure smooth performance with 1MB files and responsive typing.
**Acceptance Criteria**:
- [ ] Handle 1MB files smoothly
- [ ] Preview updates under 16ms
- [ ] Responsive typing with no lag
- [ ] Efficient memory usage
**Labels**: Performance, Optimization
**Estimate**: 5 points

## Future Enhancements (Post-MVP)

### WE-026: Interactive Checkboxes
**Title**: Implement clickable task list checkboxes
**Description**: Make task list checkboxes interactive with animations.

### WE-027: Clickable Links
**Title**: Implement Cmd+Click link following
**Description**: Add ability to follow links with keyboard modifier.

### WE-028: Undo/Redo System
**Title**: Implement comprehensive undo/redo functionality
**Description**: Add intelligent undo/redo with 200 levels.

### WE-029: Find/Replace
**Title**: Implement find and replace functionality
**Description**: Add search and replace with regex support.

### WE-030: Table Editing
**Title**: Implement enhanced table editing features
**Description**: Add table navigation and editing shortcuts.

---

## Estimation Legend
- 1 point: ~2-4 hours
- 2 points: ~4-8 hours  
- 3 points: ~1 day
- 5 points: ~2-3 days
- 8 points: ~1 week

## Priority Labels
- **P0**: Critical for MVP
- **P1**: Important for MVP
- **P2**: Nice to have for MVP
- **P3**: Post-MVP

## Component Labels
- **Core**: Core editor functionality
- **Parser**: Markdown parsing
- **Rendering**: Display and UI
- **File Operations**: File I/O
- **Formatting**: Text formatting
- **Testing**: Quality assurance
- **Performance**: Speed optimization