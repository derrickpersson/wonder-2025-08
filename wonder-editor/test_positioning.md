# Cursor Positioning Test Document

This document is designed to test cursor positioning accuracy.

## Test Cases

### 1. Short Lines
Line 1
Line 2
Line 3

### 2. Long Lines
This is a very long line that should help us test whether cursor positioning becomes less accurate after long lines. The positioning should remain accurate regardless of line length.

### 3. Mixed Content
Short.
Another very long line with lots of text to test positioning accuracy after long content. Does the cursor position correctly here?
Short again.

### 4. Special Characters
Testing with special chars: !@#$%^&*()_+-=[]{}|;':",./<>?
Testing with unicode: 你好世界 🌍 émojis included

### 5. Markdown Elements
**Bold text** and *italic text* and `inline code` all in one line.
[Links should work](http://example.com) and positioning should be accurate.

## Testing Instructions

1. Press `Ctrl+Shift+D` to enable cursor diagnostics
2. Click at various positions in the document
3. Use arrow keys to move cursor
4. Create selections with Shift+Arrow keys
5. Check console output for diagnostic logs

The diagnostics will show:
- Screen coordinates of mouse clicks
- Calculated text positions (row, column)
- Offset conversions
- Line height calculations
- Any positioning mismatches

## Expected Behavior

When you click on any character, the cursor should appear exactly at that position.
The logs should show no MISMATCH warnings.

## Known Issues to Test

1. Click on "default" - cursor should not jump to "platform"
2. Click after long lines - positioning should remain accurate
3. Click on headings - should account for larger font sizes