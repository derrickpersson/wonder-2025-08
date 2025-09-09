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

## Critical Implementation Principles

### ALWAYS Use Actual GPUI Rendering Values - NEVER Estimate
**CRITICAL**: When working with text layout, visual lines, cursor positioning, or any rendering-related functionality:

- **✅ ALWAYS** use actual values from GPUI's text rendering system
- **✅ ALWAYS** get real font metrics, character widths, and line positions from GPUI
- **✅ ALWAYS** use the actual visual lines generated during the GPUI layout phase
- **❌ NEVER** estimate character widths (e.g., assuming 8px per character)
- **❌ NEVER** estimate line wrapping positions based on character counts
- **❌ NEVER** approximate visual line boundaries without actual rendering data

**Why This Matters:**
- Text rendering is complex - fonts have variable character widths, kerning, ligatures, etc.
- Different characters have vastly different widths (e.g., 'i' vs 'W' vs '😀')
- Line wrapping depends on actual pixel measurements, not character counts
- Estimates will ALWAYS be wrong and lead to broken cursor positioning, selection, and navigation

**Correct Approach:**
```rust
// ✅ CORRECT: Use actual GPUI measurements
let shaped_line = window.text_system().shape_line(text, font_size, &runs, wrap_width);
let actual_line_width = shaped_line.width;
let actual_visual_lines = line_wrapper.wrap_line(logical_line, text, cursor, selection, window);

// ❌ WRONG: Estimating values
let estimated_chars_per_line = wrap_width / 8.0; // NEVER DO THIS
let estimated_line_breaks = text.len() / 80; // NEVER DO THIS
```

**Implementation Strategy:**
1. During the GPUI layout phase (`request_layout`), generate and store actual visual lines
2. During the paint phase, use those stored visual lines for cursor/selection rendering
3. For testing, either mock the GPUI window properly or test at the integration level
4. If GPUI values aren't available yet, defer the operation rather than estimating

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

# Run performance benchmarks
cargo run --bin benchmark

# Quick performance validation
cargo run --bin benchmark -- --quick
```

## Performance Testing Standards - MANDATORY ⚡

### Performance Requirements (ENG-148 Validated)
**ALL features must meet these performance standards:**

- ✅ **<10ms response time** for any single edit operation
- ✅ **<10ms response time** for any cursor/selection operation  
- ✅ **No UI freezing** during any operation (including 10MB+ documents)
- ✅ **Minimum throughput targets**:
  - Cursor positioning: >1M ops/sec
  - Character insertion: >250 ops/sec (10MB docs), >100K ops/sec (1KB docs)
  - Word navigation: >25K ops/sec
  - Selection operations: >250 ops/sec (10MB docs)

### Performance Testing Workflow - MANDATORY
**BEFORE every feature commit:**

1. **🔴 RED Phase - Performance Test First:**
   ```rust
   #[test] 
   fn test_new_feature_performance_large_document() {
       let mut doc = create_large_test_document(1024 * 1024); // 1MB
       let start = std::time::Instant::now();
       
       // Execute new feature operation
       doc.your_new_feature();
       
       let duration = start.elapsed();
       assert!(duration.as_millis() < 10, "Operation took {}ms, exceeds 10ms limit", duration.as_millis());
   }
   ```

2. **🟢 GREEN Phase - Meet Performance Target:**
   - Implement feature to pass performance test
   - Use O(log n) algorithms where possible (Rope benefits)
   - Avoid full document scans

3. **🔵 REFACTOR Phase - Optimize:**
   - Profile with `cargo run --bin benchmark` if needed
   - Apply Rope-specific optimizations
   - Maintain performance targets

### Performance Regression Prevention
**MANDATORY checks before ANY commit:**

```bash
# 1. Run full benchmark suite (must complete in <30 seconds)
cargo run --bin benchmark

# 2. Verify all operations under 10ms target
# Look for "✅ Within 10ms performance target" messages

# 3. Run all tests (performance tests included)
cargo test

# 4. If ANY performance regression found:
#    - STOP development
#    - Fix performance issue FIRST
#    - Re-run benchmarks to verify fix
#    - THEN commit
```

### Performance Testing Commands
```bash
# Quick validation (during development)
cargo run --bin benchmark -- --quick

# Comprehensive benchmark (before commits)
cargo run --bin benchmark

# Performance regression testing
cargo run --bin benchmark > current_results.txt
# Compare with baseline performance

# Profile specific operations
cargo test test_cursor_performance_on_large_document -- --nocapture
```

### Current Performance Baseline (ENG-148)
**These are MINIMUM acceptable performance levels:**

| Operation | 1KB | 100KB | 1MB | 10MB | Target |
|-----------|-----|-------|-----|------|--------|
| Cursor positioning | 6M ops/sec | 6.6M ops/sec | 8M ops/sec | 8.4M ops/sec | >1M ops/sec ✅ |
| Single char insert | 130K ops/sec | 34K ops/sec | 4K ops/sec | 293 ops/sec | >250 ops/sec ✅ |
| Word navigation | 127K ops/sec | 32K ops/sec | 38K ops/sec | 50K ops/sec | >25K ops/sec ✅ |
| Selection operations | 665K ops/sec | 50K ops/sec | 4K ops/sec | 264 ops/sec | >250 ops/sec ✅ |
| **Worst case latency** | 0.1ms | 0.1ms | 0.3ms | **4.3ms** | **<10ms** ✅ |

### Performance Testing Guidelines

**🚨 IMMEDIATE STOP CONDITIONS:**
- Any operation takes >10ms on documents ≤10MB
- Throughput drops below minimum targets
- UI becomes unresponsive during any operation
- Full benchmark suite takes >60 seconds to complete

**⚡ Performance Optimization Priorities:**
1. **Cursor/Selection**: Must be instant (<1ms typical)
2. **Text Editing**: Must be responsive (<5ms typical)  
3. **Rendering**: Must not block UI (<16ms frame budget)
4. **Large Documents**: Must remain usable (10MB+ documents)

**🔧 Performance Debugging Tools:**
- `cargo run --bin benchmark` - Comprehensive performance analysis
- `cargo test test_cursor_performance_on_large_document` - Specific operation testing
- Built-in benchmark analysis with throughput metrics
- Rope-specific performance characteristics validation

### Performance Review Checklist
Before any commit, verify:

- [ ] All benchmark tests pass with current performance standards
- [ ] No operation exceeds 10ms response time limit
- [ ] Throughput meets minimum targets across all document sizes
- [ ] UI remains responsive during all operations
- [ ] Performance baseline maintained or improved
- [ ] Any performance-critical code includes performance tests

## Project Structure - Clean Architecture
```
wonder-editor/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library crate root for benchmarks
│   ├── app.rs               # Main application component (GPUI UI)
│   ├── benchmarks.rs        # ⚡ Performance benchmark suite (ENG-148)
│   ├── bin/
│   │   └── benchmark.rs     # 📊 Benchmark runner binary
│   ├── editor/              # 📦 UI layer - Modular editor components
│   │   ├── mod.rs           # Main MarkdownEditor struct and coordination
│   │   ├── element.rs       # GPUI Element implementation for custom rendering
│   │   ├── rendering.rs     # Editor-specific rendering logic
│   │   ├── keyboard.rs      # Keyboard event handling and integration
│   │   ├── mouse.rs         # Mouse event handling and positioning
│   │   └── tests/           # Editor integration tests
│   ├── core/                # 📦 Domain layer (business logic)
│   │   ├── mod.rs           # Core module exports and coordinate mapping
│   │   ├── cursor.rs        # Cursor position management
│   │   ├── selection.rs     # Text selection state management
│   │   ├── text_document.rs # 🚀 Rope-based text operations & selection (O(log n))
│   │   ├── point.rs         # Point-based coordinate system
│   │   └── coordinate_mapping.rs # Unified coordinate mapping system (ENG-173)
│   ├── input/               # 📦 Input layer (user interactions)
│   │   ├── mod.rs           # Input module exports
│   │   ├── actions.rs       # Action types and command patterns
│   │   ├── keymap.rs        # Keyboard mapping configuration
│   │   └── router.rs        # Input event routing and dispatch
│   ├── rendering/           # 🎨 Modular rendering system (refactored)
│   │   ├── mod.rs           # Rendering module exports and organization
│   │   ├── style_context.rs # Theme-aware styling system (ENG-165)
│   │   ├── typography.rs    # Typography hierarchy and sizing
│   │   ├── token_mode.rs    # Token render mode determination
│   │   ├── coordinate_mapping.rs # Coordinate system for rendering
│   │   ├── text_runs.rs     # GPUI TextRun generation
│   │   ├── text_content.rs  # Text content processing
│   │   ├── layout.rs        # Layout management and positioning
│   │   └── tests/           # Rendering module tests
│   ├── hybrid_renderer.rs   # 🎨 Main hybrid preview/raw mode renderer
│   └── markdown_parser.rs   # 📝 Markdown token parsing & positioning
├── Cargo.toml               # Dependencies and project metadata (includes ropey)
└── CLAUDE.md                # This file - project conventions
```

### Architecture Layers (SOLID Compliant)
1. **UI Layer** (`editor/`, `app.rs`) - Modular UI components with GPUI rendering and custom elements
2. **Input Layer** (`input/`) - Action-based input routing and command dispatch system
3. **Domain Layer** (`core/`) - Business logic with rope-based text operations and unified coordinate system
4. **Rendering Layer** (`rendering/`, `hybrid_renderer.rs`) - Modular theme-aware rendering with typography hierarchy
5. **Parsing Layer** (`markdown_parser.rs`) - Markdown tokenization with accurate positioning
6. **Performance Layer** (`benchmarks.rs`, `bin/benchmark.rs`) - Comprehensive performance testing & validation
7. **Infrastructure** - GPUI framework, ropey, coordinate mapping, external dependencies

## How the System Works 🔄

### Overview
Wonder Editor is a hybrid markdown editor that dynamically switches between raw markdown editing and rich text preview based on cursor position and selection state. The system uses a rope-based text data structure for O(log n) performance and a modular rendering architecture for optimal responsiveness.

### Core Data Flow

#### 1. **Text Storage & Management** (`core/text_document.rs`)
```
User Input → InputRouter → TextDocument (Rope) → Cursor/Selection Update
```
- **Rope Data Structure**: Text is stored using the `ropey` crate for efficient large document handling
- **O(log n) Operations**: Insert, delete, and positioning operations scale logarithmically
- **Cursor Management**: Point-based coordinate system tracks cursor position across line/column boundaries
- **Selection State**: Range-based selection with visual highlighting support

#### 2. **Input Processing Pipeline** (`input/`)
```
Keyboard/Mouse Events → InputRouter → EditorAction → TextDocument → UI Update
```
- **Action System**: All user interactions converted to structured EditorAction commands
- **Input Router**: Centralized dispatch system that routes events to appropriate handlers
- **Command Pattern**: Extensible action system supporting undo/redo and complex operations

#### 3. **Hybrid Rendering Pipeline** (`rendering/`, `hybrid_renderer.rs`)
```
TextDocument → MarkdownParser → TokenRenderMode → StyledTextSegments → GPUI TextRuns
```

**Step 3a: Parsing & Tokenization**
- **Markdown Parser**: Converts raw text to positioned tokens (headings, bold, italic, etc.)
- **Position Tracking**: Each token maintains byte offset ranges in the original document
- **Token Classification**: Identifies markdown syntax vs content for rendering decisions

**Step 3b: Render Mode Determination** (`rendering/token_mode.rs`)
- **Cursor-Based Switching**: Tokens switch between preview/raw mode based on cursor proximity
- **Selection Awareness**: Selected text always renders in raw mode for editing
- **Smart Boundaries**: Smooth transitions between preview and raw rendering

**Step 3c: Styling & Typography** (`rendering/style_context.rs`, `rendering/typography.rs`)
- **Theme System**: Context-aware styling with dark/light theme support
- **Typography Hierarchy**: Heading levels, code blocks, and text sizing
- **GPUI Integration**: Styled segments converted to GPUI TextRuns for rendering

#### 4. **Coordinate System** (`core/coordinate_mapping.rs`, `rendering/coordinate_mapping.rs`)
```
Byte Offsets ↔ Point (Line/Column) ↔ Screen Coordinates ↔ Mouse Position
```
- **Unified Mapping**: Consistent coordinate translation between text, layout, and screen space
- **Rope Integration**: Efficient line/column calculations using rope data structure
- **Mouse Positioning**: Accurate click-to-cursor positioning with typography awareness
- **Selection Bounds**: Precise visual selection highlighting across hybrid rendering modes

#### 5. **GPUI Integration** (`editor/element.rs`, `editor/rendering.rs`)
```
TextRuns → GPUI Element → Window Rendering → User Interface
```
- **Custom Element**: Implements GPUI Element trait for advanced text rendering control
- **Focus Management**: Integrates with GPUI focus system for keyboard input handling
- **Event Delegation**: Mouse and keyboard events processed through GPUI's event system
- **Performance Optimization**: Efficient text shaping and rendering with minimal redraws

### Key System Interactions

#### **Edit Operation Flow**
1. User types character → Keyboard event captured
2. InputRouter converts to InsertChar action
3. TextDocument updates rope structure (O(log n))
4. Cursor position updated using Point system
5. MarkdownParser re-parses affected region
6. HybridRenderer determines new render modes
7. GPUI rerenders affected text runs
8. Screen updates with new content

#### **Mode Switching Flow**
1. Cursor moves → Position change detected
2. HybridRenderer evaluates all token positions
3. Tokens near cursor switch to raw mode
4. Tokens far from cursor switch to preview mode
5. StyleContext applies appropriate themes
6. TextRunGenerator creates new styled segments
7. GPUI updates display with smooth transition

#### **Mouse Click Flow**
1. Mouse click → GPUI screen coordinates
2. CoordinateMapper converts to text position
3. Typography system accounts for font sizing
4. Point system determines line/column
5. TextDocument updates cursor position
6. UI rerenders with new cursor placement

### Performance Characteristics

- **Text Operations**: O(log n) complexity for documents up to 10MB+
- **Rendering**: Incremental updates only for changed regions
- **Coordinate Mapping**: Cached calculations for repeated operations
- **Memory Usage**: Rope structure provides efficient memory utilization
- **Response Time**: <10ms for all interactive operations

### Module Responsibilities

- **`core/`**: Pure business logic, no UI dependencies
- **`input/`**: User interaction abstraction, converts events to actions  
- **`rendering/`**: Display logic, no business logic dependencies
- **`editor/`**: GPUI integration, coordinates between layers
- **`hybrid_renderer.rs`**: High-level rendering orchestration

This architecture ensures clean separation of concerns, testability, and maintainability while delivering high-performance text editing with smooth hybrid preview/raw mode transitions.

## Current Features & Components

### Core Features ✅
- **Hybrid Rendering**: Real-time switching between preview and raw markdown based on cursor position
- **Text Selection**: Full selection support with Shift+arrow keys and visual highlighting
- **Rope-based Text Operations**: O(log n) performance for large documents (ENG-144 to ENG-147)
- **Performance Benchmarking**: Comprehensive performance testing infrastructure (ENG-148)
- **Unified Coordinate System**: Point-based positioning with accurate mouse-to-text mapping (ENG-173)
- **Typography Hierarchy**: Multi-level heading system with proper font sizing (ENG-168)
- **Theme-Aware Styling**: Context-aware styling system with dark/light theme support (ENG-165)
- **Modular Rendering Architecture**: Separated concerns with dedicated rendering modules
- **Markdown Parsing**: Supports headings, bold, italic, code, links, lists, blockquotes with emoji support (ENG-163)
- **Highlight Text Support**: Advanced text highlighting capabilities (ENG-155)
- **GPUI Integration**: Custom Element implementation with advanced text rendering control
- **Action-based Input System**: Extensible command pattern with centralized input routing
- **Zero-copy Rendering**: Efficient RopeSlice integration for optimal memory usage
- **Mouse Positioning Accuracy**: Precise click-to-cursor positioning with typography awareness

### Core Components (All Test-Driven - 150+ Tests)
- **TextDocument** (21 tests) - Rope-based text operations, cursor management, selection (O(log n))
- **PerformanceBenchmark** (4 tests) - Performance testing infrastructure and validation (ENG-148)
- **HybridTextRenderer** (9 tests) - Preview/raw mode switching logic (rope optimized)
- **Coordinate System** (10+ tests) - Point-based positioning and unified coordinate mapping (ENG-173)
- **Rendering Modules** (15+ tests) - Modular rendering system with typography and styling
- **Input System** (8+ tests) - Action-based input routing and command dispatch
- **MarkdownParser** (15+ tests) - Enhanced markdown tokenization with emoji and highlight support
- **Cursor & Selection** (9 tests) - Position management and selection state with Point integration
- **Mouse Positioning** (6+ tests) - Accurate mouse-to-text coordinate mapping
- **Editor Integration** (15+ tests) - UI layer behavior with custom Element implementation
- **Typography System** (5+ tests) - Hierarchical font sizing and theme-aware styling
- **Application** (1 test) - Main application component testing

**Recent Architecture Improvements:**
- Modular rendering system with separated concerns
- Point-based coordinate system for accurate positioning
- Theme-aware styling with context management
- Enhanced mouse positioning accuracy
- Improved typography hierarchy system

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
8. **FOR INPUT/KEYBOARD FEATURES**: Follow INTEGRATION_CHECKLIST.md COMPLETELY
9. **PERFORMANCE VALIDATION** - Run `cargo run --bin benchmark -- --quick` during development
10. **Repeat TDD cycle** for next behavior
11. **Update Linear** with progress comments and any blockers discovered
12. **PRE-COMMIT PERFORMANCE CHECK** - Run `cargo run --bin benchmark` (must pass)
13. **Mark ticket complete** in Linear only when all tests pass AND performance targets met
14. **COMMIT CHANGES** immediately after task completion (NEW RULE)
15. **Move to next priority task** and repeat TDD process

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
- Completing a task without committing → STOP immediately
- Committing without running performance benchmark → STOP immediately (NEW)
- Any operation taking >10ms → STOP immediately (NEW)
- Performance regression detected → STOP immediately (NEW)

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
- **ropey**: High-performance rope data structure for text editing (O(log n) operations)
- **anyhow**: Error handling
- **serde**: Serialization/deserialization
- **pulldown-cmark**: Markdown parsing with positioning support
- **env_logger**: Logging (dev dependency)

## Test Coverage Requirements
- **All new code MUST be test-driven (TDD)**
- **Current: 154 tests across all components** (updated after ENG-148)
- **Minimum 90% test coverage for core components**
- **Performance tests for all new features** (MANDATORY)
- **Integration tests for UI components**
- **No untested code in core/ or input/ layers**
- **MANDATORY: Full integration tests for input features (see INTEGRATION_CHECKLIST.md)**
- **MANDATORY: Performance benchmarking before every commit**

### Testing Architecture Layers
1. **Unit Tests**: Core domain logic (cursor, selection, text operations)
2. **Performance Tests**: Benchmark suite validating <10ms response times (ENG-148)
3. **Integration Tests**: Input handling and command execution  
4. **Component Tests**: UI layer behavior and rendering
5. **Selection Tests**: Visual highlighting and interaction behavior
6. **Rope Tests**: O(log n) algorithm correctness and performance
7. **End-to-End Tests**: Full user interaction flows (future)

## Codebase Health Status ✅

### Recently Completed (2025) - Architecture & Coordinate System Success
- **Modular Architecture Complete**: Refactored rendering system into focused, single-responsibility modules
- **Unified Coordinate System**: Point-based positioning with accurate mouse-to-text mapping (ENG-173)
- **Typography Hierarchy**: Multi-level heading system with proper font sizing (ENG-168)
- **Theme-Aware Styling**: Context-aware styling system with dark/light theme support (ENG-165)
- **Mouse Positioning Accuracy**: Comprehensive fixes for click-to-cursor positioning accuracy
- **Rope Migration Complete**: String → Rope data structure (ENG-144 to ENG-148)
- **Performance Optimizations**: O(n) → O(log n) for all text operations
- **Zero-copy Rendering**: RopeSlice integration with hybrid renderer
- **Enhanced Markdown Support**: Emoji parsing (ENG-163) and highlight text support (ENG-155)
- **Comprehensive Benchmarking**: 28 performance operations validated <10ms
- **Architecture Enhanced**: Added performance testing layer + modular rendering system
- **Test Coverage Enhanced**: 150+ tests with expanded coordinate system and rendering coverage

### Current Quality Metrics  
- **Tests**: 150+ tests, 100% passing ✅
- **Performance**: All operations <10ms (8.4M ops/sec peak) ✅  
- **Compilation**: Clean with no errors ✅
- **Architecture**: SOLID principles + modular rendering + unified coordinates ✅
- **Features**: Advanced hybrid rendering with accurate mouse positioning ✅
- **Coordinate System**: Point-based positioning with sub-pixel accuracy ✅
- **Rendering**: Modular, theme-aware system with typography hierarchy ✅
- **Scalability**: Handles 10MB+ documents with <5ms response time ✅

## Future Development Priorities
As the project grows, we will add:
- **Collaborative editing capabilities** (ENG-149 - CRDTs with rope foundation)
- Multi-line selection highlighting support
- Advanced markdown features (tables, footnotes, task lists)
- Plugin system for custom markdown extensions
- Real-time collaborative synchronization
- Advanced keyboard shortcuts and vim-style navigation
- Performance profiling and optimization tools

## Code Review Standards
- All code must pass TDD cycle review
- No code without comprehensive tests
- **All new features must pass performance benchmarks** (NEW)
- **<10ms response time requirement** for all operations (NEW)
- Clear, descriptive naming throughout
- SOLID principles applied consistently
- Rope-optimized algorithms preferred (O(log n) vs O(n))
- Performance considerations documented and tested
- Security best practices followed (no secrets in code)