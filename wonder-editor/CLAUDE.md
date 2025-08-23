# Wonder Editor - Project Conventions

## Project Overview
Wonder Editor is a hybrid markdown editor built using the GPUI framework (from Zed). The project features real-time preview/raw mode switching based on cursor position and text selection. Tasks are tracked using Linear in the "Markdown Editor" project.

## Development Methodology

### Test-Driven Development (TDD) - MANDATORY
We follow STRICT TDD principles with a Red-Green-Refactor cycle:

1. **🔴 RED**: Write a failing test FIRST
2. **🟢 GREEN**: Write minimal code to make the test pass  
3. **🔵 REFACTOR**: Clean up the code while keeping tests green

### TDD Rules - NO EXCEPTIONS
- **NEVER write production code without a failing test first**
- **NEVER write more test code than necessary to fail**
- **NEVER write more production code than necessary to pass**
- **ALL tests must pass before moving to the next cycle**
- **Refactor only when tests are green**

### TDD Cycle Enforcement
Each development task MUST follow this exact sequence:

1. **🔴 RED Phase** (30 seconds - 2 minutes):
   - Write the smallest possible failing test
   - Run the test and confirm it fails for the right reason
   - Do NOT write any production code yet

2. **🟢 GREEN Phase** (30 seconds - 5 minutes):
   - Write the minimal code to make the test pass
   - No more, no less - resist the urge to add extra features
   - Run all tests to ensure they pass

3. **🔵 REFACTOR Phase** (1-10 minutes):
   - Clean up code while keeping all tests green
   - Remove duplication, improve names, extract methods
   - Run tests after each refactor step
   - Apply SOLID principles and design patterns

### TDD Violations - IMMEDIATE STOP
If you find yourself:
- Writing production code before a test ❌ STOP
- Writing complex tests that test multiple things ❌ STOP  
- Making tests pass by changing the test ❌ STOP
- Skipping the refactor phase ❌ STOP
- Writing code "just in case" ❌ STOP

### TDD Examples

**✅ CORRECT TDD Example:**
```rust
// 1. RED: Write failing test
#[test]
fn test_insert_char_at_beginning() {
    let mut doc = TextDocument::new();
    doc.insert_char('a');
    assert_eq!(doc.content(), "a");
    assert_eq!(doc.cursor_position(), 1);
}
// Run test - it fails because insert_char doesn't exist

// 2. GREEN: Minimal implementation
impl TextDocument {
    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(0, ch);
        self.cursor_position = 1;
    }
}
// Run test - it passes

// 3. REFACTOR: Improve if needed
impl TextDocument {
    pub fn insert_char(&mut self, ch: char) {
        let position = self.cursor.position();
        self.content.insert(position, ch);
        self.cursor.move_right(self.content.len());
    }
}
// Run test - still passes
```

**❌ WRONG - Writing code without test:**
```rust
// This violates TDD - writing implementation first!
impl TextDocument {
    pub fn insert_char(&mut self, ch: char) {
        // Complex implementation without test
    }
}
```

### Code Quality Principles
- **Simplicity**: Write clean, simple code that is easy to understand
- **Small Functions**: Keep functions focused on a single responsibility
- **Clear Naming**: Use descriptive names for functions, variables, and modules
- **No Premature Optimization**: Focus on correctness first, optimize when needed

## Testing Commands
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check code without running
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Structure - Clean Architecture
```
wonder-editor/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Main application component (GPUI UI)
│   ├── editor.rs            # UI layer - Hybrid markdown editor component
│   ├── core/                # 📦 Domain layer (business logic)
│   │   ├── mod.rs           # Core module exports (TextDocument only)
│   │   ├── cursor.rs        # Cursor position management
│   │   ├── selection.rs     # Text selection state management
│   │   └── text_document.rs # Core text operations & selection logic
│   ├── input/               # 📦 Input layer (user interactions)
│   │   ├── mod.rs           # Input module exports
│   │   ├── input_event.rs   # Input event types & special keys
│   │   ├── commands.rs      # Command pattern for text operations
│   │   └── keyboard_handler.rs # Input processing & command dispatch
│   ├── hybrid_renderer.rs   # 🎨 Hybrid preview/raw mode rendering
│   └── markdown_parser.rs   # 📝 Markdown token parsing & positioning
├── Cargo.toml               # Dependencies and project metadata
└── CLAUDE.md                # This file - project conventions
```

### Architecture Layers (SOLID Compliant)
1. **UI Layer** (`editor.rs`, `app.rs`) - Pure UI components, GPUI rendering
2. **Input Layer** (`input/`) - Handles user interactions, converts to commands
3. **Domain Layer** (`core/`) - Business logic, text operations, core abstractions
4. **Rendering Layer** (`hybrid_renderer.rs`) - Preview/raw mode switching logic
5. **Parsing Layer** (`markdown_parser.rs`) - Markdown tokenization with positions
6. **Infrastructure** - GPUI framework, external dependencies

## Current Features & Components

### Core Features ✅
- **Hybrid Rendering**: Real-time switching between preview and raw markdown based on cursor position
- **Text Selection**: Full selection support with Shift+arrow keys and visual highlighting
- **Markdown Parsing**: Supports headings, bold, italic, code, links, lists, blockquotes
- **GPUI Integration**: Custom text rendering with mixed font styles and highlighting
- **Command System**: Extensible command pattern for all text operations

### Core Components (All Test-Driven - 81 Tests)
- **TextDocument** (18 tests) - Main text operations, cursor management, selection
- **HybridTextRenderer** (9 tests) - Preview/raw mode switching logic
- **KeyboardHandler** (5 tests) - Input event processing
- **EditorCommand** (13 tests) - Command pattern for operations including selection
- **MarkdownParser** (13 tests) - Markdown tokenization with positioning
- **Cursor** (5 tests) - Position management
- **Selection** (4 tests) - Selection state management
- **InputEvent** (2 tests) - Input event types
- **Editor Integration** (12 tests) - UI layer behavior and selection highlighting

## Linear Integration & Workflow

### Task Selection Process - TDD FIRST
1. **Always pick the next available task from Linear** in priority order
2. **Before starting any task, break it down** into TDD cycles
3. **Ask key questions** about test-first implementation approach  
4. **Plan the RED-GREEN-REFACTOR cycle** for each subtask

### TDD-First Task Planning
Before writing ANY code, plan your TDD cycles:

1. **Identify the smallest testable behavior**
2. **Write the test name first** (describes what you're testing)
3. **Plan the minimal implementation** that would make it pass
4. **Consider the refactoring opportunities** after green

### Pre-Task Breakdown Questions - TDD FOCUSED
Before working on any Linear ticket, ask yourself:

1. **What tests will drive this feature?** (TDD First!)
   - What's the first failing test I should write?
   - What's the smallest behavior I can test?
   - How can I break this into 5-10 micro TDD cycles?

2. **What are the core subtasks?** (After test planning)
   - What are the 3-5 main TDD cycles this task needs?
   - Which test should be written first?
   - Are there any test dependencies between subtasks?

3. **What's the RED-GREEN-REFACTOR plan?**
   - RED: What's the simplest failing test?
   - GREEN: What's the minimal code to make it pass?
   - REFACTOR: What improvements can be made while tests are green?

4. **What are the integration points?** (Test boundaries)
   - What interfaces need to be tested?
   - How will I test the integration without breaking TDD?
   - What might break and how will tests catch it?

### TDD Cycle Checklist
For each subtask, ensure:
- [ ] 🔴 Written failing test first
- [ ] 🔴 Test fails for the right reason  
- [ ] 🟢 Written minimal code to pass
- [ ] 🟢 All tests pass
- [ ] 🔵 Refactored while keeping tests green
- [ ] 🔵 Applied SOLID principles where appropriate

### Linear Workflow Steps - TDD MANDATORY
1. **Check Linear** for the next task from Linear in the "Markdown Editor" project
2. **Read the ticket** thoroughly, including acceptance criteria
3. **Break down the task** into TDD cycles using the questions above
4. **Create a TDD plan** with 5-10 specific RED-GREEN-REFACTOR cycles
5. **Start with first failing test** (RED phase) - NO EXCEPTIONS
6. **Make it pass minimally** (GREEN phase)
7. **Refactor while tests are green** (REFACTOR phase)
8. **Repeat TDD cycle** for next behavior
9. **Update Linear** with progress comments and any blockers discovered
10. **Mark ticket complete** in Linear only when all tests pass
11. **COMMIT CHANGES** immediately after task completion (NEW RULE)
12. **Move to next priority task** and repeat TDD process

### AUTO-COMMIT RULE - NEW REQUIREMENT ⚡
**MANDATORY**: After completing ANY task (Linear ticket, bug fix, feature implementation, or cleanup):

1. **IMMEDIATELY create a git commit** with descriptive message
2. **Use proper commit format** (see below)
3. **Include all relevant changes** in the commit
4. **Do NOT wait** for user instruction to commit
5. **Commit BEFORE** moving to the next task

This ensures:
- All work is properly tracked and versioned
- Changes are not lost between tasks
- Clear progress documentation
- Easy rollback if needed

### TDD Workflow Enforcement
**🚨 VIOLATION ALERT:** If you catch yourself:
- Writing implementation before test → STOP immediately
- Making a test pass by changing the test → STOP immediately  
- Skipping any of the 3 TDD phases → STOP immediately
- Writing multiple behaviors in one test → STOP immediately
- Completing a task without committing → STOP immediately (NEW)

### Checking Linear for Next Task
Use the Linear MCP server to:
- List issues in the "Markdown Editor" project
- Check issue status and priority
- Get the next available task (usually lowest numbered ENG-XX not yet started)
- Read full issue details including acceptance criteria

### Always continue working
- After you have completed your current Linear task. Always check Linear for the next task assigned to you, and continue with that task.
- If there are no further tasks assigned to you, see if there's another task you can pull in from the backlog and assign it to you.

### Commit Message Format
```
[ENG-XX] Brief description of change

- Specific change made
- Any important notes

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Example TDD Task Breakdown
For ticket "ENG-001: Visual Selection Highlighting":

**TDD Cycles (RED-GREEN-REFACTOR):**
1. 🔴 **Test**: Selection highlighting data flow → 🟢 **Code**: Basic highlighting method → 🔵 **Refactor**: Clean interface
2. 🔴 **Test**: Single-line selection bounds → 🟢 **Code**: Position calculation → 🔵 **Refactor**: Extract helpers
3. 🔴 **Test**: Text shaping integration → 🟢 **Code**: GPUI text measurement → 🔵 **Refactor**: Optimize performance
4. 🔴 **Test**: Selection color rendering → 🟢 **Code**: Paint quad with alpha → 🔵 **Refactor**: Color constants
5. 🔴 **Test**: Shift+arrow key integration → 🟢 **Code**: Event handling → 🔵 **Refactor**: Command pattern

**First Test Example:**
```rust
#[test]
fn test_selection_highlighting_data_flow() {
    let mut editor = new_with_content("Hello World".to_string());
    editor.document_mut().set_cursor_position(6);
    editor.handle_input_event(InputEvent::ShiftArrowRight);
    
    assert!(editor.document_mut().has_selection());
    let selection_range = editor.document_mut().selection_range().unwrap();
    assert_eq!(selection_range, (6, 7));
}
```

**TDD Cycle Planning:**
- Each cycle should take 5-15 minutes maximum
- If a cycle takes longer, break it down further
- Always end with green tests before moving to next cycle

## GPUI Framework Notes
- Using GPUI from the Zed repository for high-performance text editing
- Components implement the `Render` trait
- Use `Context` for component initialization and entity management
- Use `div()` and builder pattern for UI construction
- Custom `Element` trait for advanced text rendering and input handling
- `TextRun` system for mixed font styles within single text spans
- `EntityInputHandler` for text input integration

### GPUI Learning Resources
**IMPORTANT**: Always check `zed-main` codebase for implementation examples when working with GPUI:
- **Text Rendering**: Look at `editor/src/editor_element.rs` for advanced text rendering patterns
- **Input Handling**: Check `editor/src/editor.rs` for keyboard and mouse event handling
- **Selection Logic**: Study `text/src/selection.rs` for selection management patterns
- **UI Components**: Examine `ui/src/` for GPUI component patterns and styling
- **Element Implementation**: Review custom element implementations in editor components
- **Performance Patterns**: Look at how Zed optimizes text rendering and large document handling

Use Zed's proven patterns rather than inventing new approaches for GPUI integration.

## Git Workflow & Auto-Commit
- **ALWAYS commit** immediately after completing any task
- Commit messages should be clear and descriptive
- Reference Linear task IDs when applicable
- Keep commits focused and atomic
- Use the standardized commit message format above
- **NO exceptions** - every completed task gets committed

## Running the Application
```bash
# Build the application
cargo build

# Run the application
cargo run

# Build release version
cargo build --release
```

## Dependencies
- **gpui**: UI framework from Zed for high-performance text editing
- **anyhow**: Error handling
- **serde**: Serialization/deserialization
- **pulldown-cmark**: Markdown parsing with positioning support
- **env_logger**: Logging (dev dependency)

## Test Coverage Requirements
- **All new code MUST be test-driven (TDD)**
- **Current: 81 tests across all components**
- **Minimum 90% test coverage for core components**
- **Integration tests for UI components**
- **No untested code in core/ or input/ layers**

### Testing Architecture Layers
1. **Unit Tests**: Core domain logic (cursor, selection, text operations)
2. **Integration Tests**: Input handling and command execution  
3. **Component Tests**: UI layer behavior and rendering
4. **Selection Tests**: Visual highlighting and interaction behavior
5. **End-to-End Tests**: Full user interaction flows (future)

## Codebase Health Status ✅

### Recently Cleaned (2024)
- **Removed Legacy Components**: text_buffer.rs, preview_renderer.rs, mode_manager.rs (~1,433 lines)
- **Updated Module Structure**: Clean exports, removed dead imports
- **Fixed Compilation Warnings**: From 31 warnings to 12 (only future-functionality warnings remain)
- **Maintained Test Coverage**: 81/81 tests passing (100%)
- **Architecture**: Clean separation maintained across all layers

### Current Quality Metrics
- **Tests**: 81 tests, 100% passing
- **Compilation**: Clean with no errors
- **Warnings**: 12 (only related to future functionality, not dead code)
- **Architecture**: SOLID principles maintained throughout
- **Features**: Hybrid rendering and visual selection highlighting fully operational

## Future Development Priorities
As the project grows, we will add:
- Multi-line selection highlighting support
- Advanced markdown features (tables, footnotes, task lists)
- Plugin system for custom markdown extensions
- Performance optimizations for large documents
- Collaborative editing capabilities
- Advanced keyboard shortcuts and vim-style navigation

## Code Review Standards
- All code must pass TDD cycle review
- No code without comprehensive tests
- Clear, descriptive naming throughout
- SOLID principles applied consistently
- Performance considerations documented
- Security best practices followed (no secrets in code)