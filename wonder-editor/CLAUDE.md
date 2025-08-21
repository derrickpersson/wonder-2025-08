# Wonder Editor - Project Conventions

## Project Overview
Wonder Editor is a markdown editor built using the GPUI framework (from Zed). The project is tracked using Linear, with tasks in the "Markdown Editor" project.

## Development Methodology

### Test-Driven Development (TDD)
We follow strict TDD principles with a Red-Green-Refactor cycle:

1. **Red**: Write a failing test first
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Clean up the code while keeping tests green

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

## Project Structure
```
wonder-editor/
├── src/
│   ├── main.rs         # Application entry point
│   ├── app.rs          # Main application component
│   ├── editor.rs       # Markdown editor component
│   └── tests/          # Integration tests
├── Cargo.toml          # Dependencies and project metadata
└── CLAUDE.md           # This file - project conventions
```

## Linear Integration & Workflow

### Task Selection Process
1. **Always pick the next available task from Linear** in priority order
2. **Before starting any task, break it down** into implementable subtasks
3. **Ask key questions** about implementation approach
4. **Plan the TDD cycle** for each subtask

### Pre-Task Breakdown Questions
Before working on any Linear ticket, ask yourself:

1. **What are the core subtasks?**
   - What are the 3-5 main components this task needs?
   - Which subtask should be implemented first?
   - Are there any dependencies between subtasks?

2. **What tests need to be written?**
   - What behavior should be tested first (Red phase)?
   - What edge cases need to be covered?
   - How can we make the tests as simple as possible?

3. **What's the minimal implementation?**
   - What's the simplest code that could make the first test pass?
   - What can we defer until later iterations?
   - How do we avoid over-engineering?

4. **What are the integration points?**
   - How does this connect to existing code?
   - What interfaces need to be defined?
   - What might break when we add this feature?

### Linear Workflow Steps
1. **Check Linear** for the next task from Linear in the "Markdown Editor" project
2. **Read the ticket** thoroughly, including acceptance criteria
3. **Break down the task** using the questions above
4. **Create a mini-plan** with 3-5 specific subtasks
5. **Start TDD cycle** with the first subtask
6. **Update Linear** with progress comments and any blockers discovered
7. **Move to next subtask** until ticket is complete
8. **Mark ticket complete** in Linear and move to next priority task

### Checking Linear for Next Task
Use the Linear MCP server to:
- List issues in the "Markdown Editor" project
- Check issue status and priority
- Get the next available task (usually lowest numbered ENG-XX not yet started)
- Read full issue details including acceptance criteria

### Commit Message Format
```
[ENG-XX] Brief description of change

- Specific change made
- Any important notes

Implements: [Linear ticket URL]
```

### Example Task Breakdown
For ticket "WE-001: Text Buffer Foundation":

**Subtasks:**
1. Create TextBuffer struct with basic fields
2. Implement insert_char method with tests
3. Implement delete_char method with tests  
4. Add cursor position tracking
5. Handle edge cases (empty buffer, boundaries)

**First Test:** Insert character at position 0 in empty buffer

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
- **env_logger**: Logging (dev dependency)

## Future Conventions
As the project grows, we will add:
- Component testing patterns
- State management patterns
- Keyboard shortcut conventions
- File organization for larger features
- Performance testing guidelines