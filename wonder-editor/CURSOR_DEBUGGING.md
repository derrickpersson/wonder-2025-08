# Cursor Positioning Diagnostics

## Overview

This diagnostic system helps identify and debug cursor positioning issues during movement and selection operations.

## How to Use

### 1. Enable Diagnostics

Press **`Ctrl+Shift+D`** to toggle cursor diagnostics on/off.

When enabled, you'll see:
```
🔍 CURSOR DIAGNOSTICS ENABLED - Press Ctrl+Shift+D to toggle
═══════════════════════════════════════════════════════════
```

### 2. Run Manual Tests

1. Load the test document: `test_positioning.md`
2. Enable diagnostics with `Ctrl+Shift+D`
3. Perform these actions and watch the console output:

#### Test Cases to Perform:

**Mouse Clicks:**
- Click on different words at various positions
- Click after long lines to test accuracy
- Click on headings (should account for larger font sizes)
- Click on special characters and unicode text

**Keyboard Navigation:**
- Use arrow keys to move cursor
- Use Shift+arrow keys to create selections
- Try word navigation (Ctrl+arrow keys if supported)
- Test line navigation (Home/End if supported)

**Text Selection:**
- Create selections with mouse drag
- Extend selections with Shift+click
- Test multi-line selections

### 3. Read the Diagnostic Output

The console will show detailed logs for each operation:

#### Mouse Click Logs:
```
🖱️ MOUSE CLICK POSITIONING
  Screen: (123.4, 56.7) pixels
  Calculated Point: row=2, col=5
  Calculated Offset: 45
  Actual Cursor: 45
  Line content: "This is the clicked line"
  Click position:      ^
```

#### Cursor Movement Logs:
```
📍 CURSOR MOVEMENT: Keyboard: ArrowRight
  Old: offset=45, point=(2, 5)
  New: offset=46, point=(2, 6)
  Delta: offset=+1, row=+0, col=+1
  Context: "is the clicked line"
           ^→
```

#### Selection Change Logs:
```
🔲 SELECTION CHANGE: Keyboard: shift+ArrowRight
  Range: 45 to 47 (2 chars)
  Cursor at: 47
  Selected: "li"
```

#### Line Height Calculations:
```
📏 LINE HEIGHT CALCULATION
  Line 0: "# This is a heading"
  Height: 32.0px
  Heading level: 1
```

#### Coordinate Conversions:
```
🔄 COORDINATE CONVERSION: Screen to offset
  Input point: (2, 5)
  Output offset: 45
```

### 4. Identify Issues

Look for these warning signs in the logs:

**⚠️ MISMATCH Warnings:**
```
  ⚠️ MISMATCH: Expected 45 but got 47 (diff: +2)
```

**Inconsistent Conversions:**
- Point-to-offset doesn't match expected result
- Screen coordinates don't map to correct text position
- Large deltas in cursor movement for simple operations

**Wrong Line Heights:**
- Headings not showing increased height
- Fixed heights when content should vary

## Expected Behavior

### Accurate Mouse Positioning
- Clicking on any character should position cursor exactly at that character
- No drift after long lines
- Proper handling of variable font sizes (headings)

### Consistent Coordinate Conversion
- Point ↔ offset conversions should be symmetric
- Screen coordinates should map accurately to text positions

### Proper Selection Behavior
- Selections should start/end exactly where clicked
- Keyboard selection extensions should be precise
- No unexpected jumps during selection operations

## Reporting Issues

When you find positioning issues:

1. **Capture the moment**: Press `Ctrl+Shift+D` right when the issue occurs
2. **Screenshot the diagnostic report** that appears
3. **Copy the console logs** showing the problematic operation
4. **Note the specific action** that caused the issue
5. **Include the test content** that triggered the problem

## Common Issues to Test

Based on the reported bug, specifically test:

1. **"Default" → "Platform" Jump:**
   - Click on the word "default" 
   - Cursor should NOT jump to "platform" on a different line
   - Logs should show accurate coordinate conversion

2. **Long Line Accuracy:**
   - Type or load a very long line
   - Click at various positions along the line
   - Accuracy should not degrade toward the end

3. **Mixed Content:**
   - Documents with headings and regular text
   - Variable line heights should be calculated correctly
   - Mouse positioning should account for font size differences

## Diagnostic Report

The diagnostic report (shown when enabling diagnostics) provides:
- Document statistics (lines, characters)
- Current cursor position (both offset and point)
- Selection information
- Current line content and cursor position within it
- Tips for further debugging

This comprehensive diagnostic system will help identify exactly where the cursor positioning logic fails and provide detailed information for fixing the issues.