# Integration Checklist for New Features

## ✅ Required Steps for ANY New Keyboard/Input Feature

When adding any new keyboard shortcut or input handling feature, you MUST complete ALL of these steps:

### 1. Input Layer (Bottom-Up)
- [ ] Add new variant to `InputEvent` enum in `src/input/input_event.rs`
- [ ] Add new variant to `SpecialKey` enum (if applicable) in `src/input/input_event.rs`
- [ ] Add conversion in `From<SpecialKey> for InputEvent` implementation

### 2. Command Layer
- [ ] Add new variant to `EditorCommand` enum in `src/input/commands.rs`
- [ ] Implement command execution in `impl CommandExecutor for TextDocument`
- [ ] Add mapping in `KeyboardHandler::map_input_to_command()`

### 3. Business Logic Layer
- [ ] Implement the actual logic methods in `TextDocument`
- [ ] Write TDD tests for the core logic FIRST (following TDD principles)
- [ ] Ensure all edge cases are handled

### 4. UI Integration Layer (CRITICAL - OFTEN MISSED!)
- [ ] Add GPUI action to `actions!` macro in `src/editor.rs`
- [ ] Add mapping in `MarkdownEditor::action_to_input_event()`
- [ ] Add mapping in `TestableEditor::action_to_input_event()` (for tests)
- [ ] Ensure `handle_editor_action()` properly routes the action

### 5. Testing - ALL LEVELS REQUIRED
- [ ] **Unit Tests**: Core logic in TextDocument
- [ ] **Integration Tests**: KeyboardHandler tests
- [ ] **Action Tests**: Test GPUI action → InputEvent conversion
- [ ] **End-to-End Tests**: Test complete flow from action to result

### 6. Documentation
- [ ] Update this checklist if new patterns emerge
- [ ] Document any special behavior in CLAUDE.md
- [ ] Add comments for non-obvious mappings

## 🚨 Common Mistakes to Avoid

1. **Forgetting GPUI Actions**: The most common mistake - implementing everything but forgetting to add the GPUI action bindings
2. **Incomplete Test Coverage**: Only testing the core logic but not the integration points
3. **Mismatched Mappings**: Having different behavior in TestableEditor vs MarkdownEditor
4. **Missing Edge Cases**: Not handling boundary conditions in navigation

## 📋 Feature Implementation Template

Use this template when implementing a new input feature:

```markdown
## Feature: [Feature Name]

### Checklist
- [ ] InputEvent variant added
- [ ] SpecialKey variant added (if needed)
- [ ] EditorCommand variant added
- [ ] Command execution implemented
- [ ] KeyboardHandler mapping added
- [ ] TextDocument methods implemented
- [ ] GPUI action added to actions! macro
- [ ] MarkdownEditor::action_to_input_event mapping added
- [ ] TestableEditor::action_to_input_event mapping added
- [ ] Unit tests written (TDD)
- [ ] Integration tests written
- [ ] Action flow tests written
- [ ] Manual testing completed

### Test Coverage
- Unit Tests: ___/___
- Integration Tests: ___/___
- Action Tests: ___/___
```

## 🔍 Verification Commands

Run these commands to verify complete integration:

```bash
# Check all tests pass
cargo test

# Check for missing action mappings
grep -n "InputEvent::" src/input/input_event.rs | wc -l
grep -n "EditorCommand::" src/input/commands.rs | wc -l
grep -n "Editor" src/editor.rs | grep -c "actions!"

# Verify test coverage
cargo test 2>&1 | grep -E "test result|running"
```

## 🎯 Integration Test Pattern

Always include an integration test that verifies the complete flow:

```rust
#[test]
fn test_[feature]_complete_integration() {
    let mut editor = new_with_buffer();
    
    // Setup initial state
    editor.handle_char_input('t');
    editor.handle_char_input('e');
    editor.handle_char_input('s');
    editor.handle_char_input('t');
    
    // Test the action (this verifies GPUI integration)
    editor.handle_editor_action(&EditorNewAction {});
    
    // Verify the result
    assert_eq!(editor.some_property(), expected_value);
}
```

## 🚀 Pre-Commit Checklist

Before committing any input/keyboard feature:

1. Run `cargo test` - ALL tests must pass
2. Run `cargo check` - No compilation errors
3. Manually verify the feature works in the running application
4. Check this checklist is complete
5. Update test count in commit message

## 📊 Tracking Integration Completeness

For each feature, track these metrics:
- Input Events Added: ___
- Commands Added: ___
- Actions Added: ___
- Tests Added: ___
- Total Test Count: ___ (should increase)

Remember: **A feature is NOT complete until it works end-to-end from keyboard to screen!**