# Wonder Editor - Project Conventions

## Project Overview
Wonder Editor is a markdown editor built using the GPUI framework (from Zed). The project is tracked using Linear, with tasks in the "Markdown Editor" project.

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
│   ├── app.rs               # Main application component
│   ├── editor.rs            # UI layer - Markdown editor component
│   ├── core/                # 📦 Domain layer (business logic)
│   │   ├── mod.rs           # Core module exports
│   │   ├── cursor.rs        # Cursor management abstraction
│   │   ├── selection.rs     # Text selection abstraction
│   │   └── text_document.rs # Core text operations
│   ├── input/               # 📦 Input layer (user interactions)
│   │   ├── mod.rs           # Input module exports
│   │   ├── input_event.rs   # Input event types
│   │   ├── commands.rs      # Command pattern for operations
│   │   └── keyboard_handler.rs # Input processing logic
│   ├── text_buffer.rs       # 🚫 Legacy - being phased out
│   └── markdown_parser.rs   # 📦 Will move to core/parsing/
├── Cargo.toml               # Dependencies and project metadata
└── CLAUDE.md                # This file - project conventions
```

### Architecture Layers (SOLID Compliant)
1. **UI Layer** (`editor.rs`, `app.rs`) - Pure UI components, no business logic
2. **Input Layer** (`input/`) - Handles user interactions, converts to commands
3. **Domain Layer** (`core/`) - Business logic, text operations, core abstractions
4. **Infrastructure** - GPUI framework, external dependencies

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
11. **Move to next priority task** and repeat TDD process

### TDD Workflow Enforcement
**🚨 VIOLATION ALERT:** If you catch yourself:
- Writing implementation before test → STOP immediately
- Making a test pass by changing the test → STOP immediately  
- Skipping any of the 3 TDD phases → STOP immediately
- Writing multiple behaviors in one test → STOP immediately

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

Implements: [Linear ticket URL]
```

### Example TDD Task Breakdown
For ticket "WE-001: Text Buffer Foundation":

**TDD Cycles (RED-GREEN-REFACTOR):**
1. 🔴 **Test**: Create empty TextDocument → 🟢 **Code**: Basic struct → 🔵 **Refactor**: Clean interface
2. 🔴 **Test**: Insert char in empty buffer → 🟢 **Code**: Basic insert → 🔵 **Refactor**: Position tracking
3. 🔴 **Test**: Insert char at specific position → 🟢 **Code**: Position-aware insert → 🔵 **Refactor**: Error handling
4. 🔴 **Test**: Delete char from buffer → 🟢 **Code**: Basic delete → 🔵 **Refactor**: Boundary checks
5. 🔴 **Test**: Handle cursor at boundaries → 🟢 **Code**: Boundary logic → 🔵 **Refactor**: Extract helpers

**First Test Example:**
```rust
#[test]
fn test_create_empty_text_document() {
    let doc = TextDocument::new();
    assert_eq!(doc.content(), "");
    assert_eq!(doc.cursor_position(), 0);
}
```

**TDD Cycle Planning:**
- Each cycle should take 5-15 minutes maximum
- If a cycle takes longer, break it down further
- Always end with green tests before moving to next cycle

## GPUI Framework Notes
- Using GPUI from the Zed repository
- Components implement the `Render` trait
- Use `Context` for component initialization
- Use `div()` and builder pattern for UI construction

## Git Workflow
- Commit messages should be clear and descriptive
- Reference Linear task IDs when applicable
- Keep commits focused and atomic

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
- **gpui**: UI framework from Zed
- **anyhow**: Error handling
- **serde**: Serialization/deserialization
- **pulldown-cmark**: Markdown parsing
- **env_logger**: Logging (dev dependency)

## Current Architecture Status
The project now follows SOLID principles with clean architecture:

### Core Components (All Test-Driven)
- **TextDocument** (66 tests) - Main text operations
- **Cursor** (4 tests) - Cursor position management  
- **Selection** (4 tests) - Text selection abstraction
- **KeyboardHandler** (5 tests) - Input event processing
- **EditorCommand** (4 tests) - Command pattern for operations

### Test Coverage Requirements
- **All new code MUST be test-driven (TDD)**
- **Minimum 90% test coverage for core components**
- **Integration tests for UI components**
- **No untested code in core/ or input/ layers**

### Testing Architecture Layers
1. **Unit Tests**: Core domain logic (cursor, selection, text operations)
2. **Integration Tests**: Input handling and command execution
3. **Component Tests**: UI layer behavior (when possible with GPUI)
4. **End-to-End Tests**: Full user interaction flows (future)

## Future Conventions
As the project grows, we will add:
- Component testing patterns
- State management patterns
- Keyboard shortcut conventions
- File organization for larger features
- Performance testing guidelines
