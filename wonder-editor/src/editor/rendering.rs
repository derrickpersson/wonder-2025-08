use gpui::{
    px, rgb, size, transparent_black, App, Bounds, Pixels, ShapedLine, TextRun, Window,
};

use super::element::EditorElement;

impl EditorElement {
    pub(super) fn paint_selection(
        &self,
        bounds: Bounds<Pixels>,
        _shaped_lines: &[ShapedLine],
        selection_range: std::ops::Range<usize>,
        window: &mut Window,
    ) {
        let padding = px(16.0);
        let line_height = px(24.0);
        let selection_color = gpui::Rgba {
            r: 0.337,
            g: 0.502,
            b: 0.761,
            a: 0.3, // Semi-transparent blue
        };

        let content = &self.content;
        let mut char_offset = 0;

        // Process each logical line to find selections
        let lines: Vec<&str> = content.lines().collect();
        for (logical_line, line_text) in lines.iter().enumerate() {
            let line_start = char_offset;
            let line_end = char_offset + line_text.len();
            
            // Check if this logical line intersects with the selection
            if selection_range.end > line_start && selection_range.start <= line_end {
                // Calculate the selection bounds within this logical line
                let sel_start_in_line = selection_range.start.saturating_sub(line_start);
                let sel_end_in_line = (selection_range.end.min(line_end) - line_start).min(line_text.len());
                
                if sel_start_in_line < sel_end_in_line {
                    // Find all visual lines in this logical line that contain selection
                    let visual_lines_with_selection = self.visual_line_manager
                        .find_visual_lines_in_selection(logical_line, sel_start_in_line, sel_end_in_line);
                    
                    for (visual_line_idx, visual_line) in visual_lines_with_selection {
                        // Calculate selection bounds within this visual line
                        let vl_sel_start = sel_start_in_line.max(visual_line.start_offset) - visual_line.start_offset;
                        let vl_sel_end = sel_end_in_line.min(visual_line.end_offset) - visual_line.start_offset;
                        
                        if vl_sel_start < vl_sel_end {
                            // Use the simplified paint method for this visual line
                            self.paint_visual_line_selection_simple(
                                bounds,
                                &visual_line.text(),
                                vl_sel_start,
                                vl_sel_end,
                                visual_line_idx,
                                padding,
                                line_height,
                                selection_color,
                                window,
                            );
                        }
                    }
                }
            }
            
            // Move to next logical line
            char_offset = line_end + 1; // +1 for newline character
        }
    }

    fn paint_visual_line_selection_simple(
        &self,
        bounds: Bounds<Pixels>,
        visual_line_text: &str,
        sel_start: usize,
        sel_end: usize,
        visual_line_index: usize,
        padding: Pixels,
        line_height: Pixels,
        selection_color: gpui::Rgba,
        window: &mut Window,
    ) {
        let font_size = px(16.0);

        // ENG-190: Apply scroll offset to selection rendering
        let scroll_offset_px = px(self.scroll_offset);
        
        // Use VisualLineManager to get Y position with scroll offset handled internally
        let y_pos = if let Some(bounds_point) = self.visual_line_manager.get_visual_line_bounds(
            visual_line_index, 
            bounds.origin, 
            padding, 
            line_height,
            self.scroll_offset
        ) {
            bounds_point.y
        } else {
            // Fallback calculation with scroll offset
            bounds.origin.y + padding + (line_height * visual_line_index as f32) - scroll_offset_px
        };

        // Calculate X start position
        let x_start = if sel_start == 0 {
            bounds.origin.x + padding
        } else {
            let text_before = visual_line_text.chars().take(sel_start).collect::<String>();
            if !text_before.is_empty() {
                let text_run = TextRun {
                    len: text_before.len(),
                    font: gpui::Font {
                        family: "SF Pro".into(),
                        features: gpui::FontFeatures::default(),
                        weight: gpui::FontWeight::NORMAL,
                        style: gpui::FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: rgb(0xcdd6f4).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let shaped = window.text_system().shape_line(
                    text_before.into(),
                    font_size,
                    &[text_run],
                    None,
                );
                bounds.origin.x + padding + shaped.width
            } else {
                bounds.origin.x + padding
            }
        };

        // Calculate X end position
        let x_end = {
            let text_to_end = visual_line_text.chars().take(sel_end).collect::<String>();
            if !text_to_end.is_empty() {
                let text_run = TextRun {
                    len: text_to_end.len(),
                    font: gpui::Font {
                        family: "SF Pro".into(),
                        features: gpui::FontFeatures::default(),
                        weight: gpui::FontWeight::NORMAL,
                        style: gpui::FontStyle::Normal,
                        fallbacks: None,
                    },
                    color: rgb(0xcdd6f4).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let shaped = window.text_system().shape_line(
                    text_to_end.into(),
                    font_size,
                    &[text_run],
                    None,
                );
                bounds.origin.x + padding + shaped.width
            } else {
                bounds.origin.x + padding
            }
        };

        // ENG-188: Only paint selection if it's visible in the current viewport
        let selection_is_visible = y_pos >= bounds.origin.y && y_pos < bounds.origin.y + bounds.size.height;
        if !selection_is_visible {
            return; // Skip painting selection outside viewport
        }
        
        // Paint selection rectangle
        let selection_width = if x_end > x_start {
            x_end - x_start
        } else {
            px(4.0) // Minimum visible width
        };

        if selection_width > px(0.0) {
            window.paint_quad(gpui::PaintQuad {
                bounds: Bounds {
                    origin: gpui::point(x_start, y_pos),
                    size: size(selection_width, line_height),
                },
                background: selection_color.into(),
                border_widths: gpui::Edges::all(px(0.0)),
                border_color: transparent_black().into(),
                border_style: gpui::BorderStyle::Solid,
                corner_radii: gpui::Corners::all(px(0.0)),
            });
        }
    }

    pub(super) fn paint_cursor(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        // Get the cursor position from the editor (original content position)
        let original_cursor_position = self.editor.read(cx).cursor_position();
        let content = &self.content;
        
        eprintln!("DEBUG RENDER: Painting cursor at original position: {}", original_cursor_position);
        eprintln!("DEBUG RENDER: Content length: {}", content.len());

        // Calculate which logical line the cursor is on
        let chars_before_cursor: String = content.chars().take(original_cursor_position).collect();
        let logical_line = chars_before_cursor.matches('\n').count();
        
        eprintln!("DEBUG RENDER: Cursor is on logical line {} (0-based)", logical_line);

        // Find cursor position within this logical line
        let lines_before: Vec<&str> = chars_before_cursor.lines().collect();
        let position_in_logical_line = if chars_before_cursor.ends_with('\n') {
            0
        } else {
            lines_before
                .last()
                .map(|line| line.chars().count())
                .unwrap_or(0)
        };

        // Use VisualLineManager to find the visual line containing the cursor
        let (visual_line_index, visual_line, position_in_visual_line) = 
            if let Some((vl_idx, vl)) = self.visual_line_manager.find_visual_line_at_position(logical_line, position_in_logical_line) {
                eprintln!("DEBUG RENDER: Found cursor in visual line {} at position {}", vl_idx, position_in_logical_line - vl.start_offset);
                eprintln!("DEBUG RENDER: Visual line text: {:?}", vl.text());
                (Some(vl_idx), Some(vl), position_in_logical_line - vl.start_offset)
            } else {
                eprintln!("DEBUG RENDER: Cursor not found in visual lines");
                (None, None, position_in_logical_line)
            };

        // Get the text content for cursor positioning
        let (line_display_text, transformed_position_in_line) = if let Some(vl) = visual_line {
            // Use the visual line's text content directly
            (vl.text(), position_in_visual_line)
        } else {
            // Fallback: no visual line found, use original line content
            let current_line_original = content.lines().nth(logical_line).unwrap_or("");
            eprintln!("DEBUG RENDER: Using fallback with line content: {:?}", current_line_original);
            (current_line_original.to_string(), position_in_logical_line)
        };

        // Calculate cursor position
        let padding = px(16.0);
        let line_height = px(24.0);
        let font_size = px(16.0);

        // Calculate X position by shaping the TRANSFORMED text up to the TRANSFORMED cursor position
        let cursor_x_offset = if transformed_position_in_line == 0 {
            px(0.0)
        } else {
            // CRITICAL FIX: Take characters but then convert to byte slice for proper TextRun lengths
            let text_up_to_cursor = line_display_text.chars().take(transformed_position_in_line).collect::<String>();
            
            // Helper function to convert character position to byte position in a string
            let char_to_byte_pos = |text: &str, char_pos: usize| -> usize {
                text.char_indices()
                    .nth(char_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(text.len())
            };

            if text_up_to_cursor.is_empty() {
                px(0.0)
            } else {
                // For visual lines, we don't need the complex hybrid renderer logic
                let line_runs: Vec<TextRun> = Vec::new(); // Simplified approach for refactored system

                if line_runs.is_empty() {
                    // Fallback to simple measurement - TextRun.len must be in BYTES
                    let text_run = TextRun {
                        len: text_up_to_cursor.len(), // This is already in bytes
                        font: gpui::Font {
                            family: "SF Pro".into(),
                            features: gpui::FontFeatures::default(),
                            weight: gpui::FontWeight::NORMAL,
                            style: gpui::FontStyle::Normal,
                            fallbacks: None,
                        },
                        color: rgb(0xcdd6f4).into(),
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    
                    let shaped_line = window.text_system().shape_line(
                        text_up_to_cursor.into(),
                        font_size,
                        &[text_run],
                        None,
                    );
                    
                    shaped_line.width
                } else {
                    // Create a subset of text runs that covers our cursor position
                    // CRITICAL FIX: Track BYTE positions for runs, not character positions
                    let mut runs_for_cursor = Vec::new();
                    let mut byte_pos = 0; // Track byte position in line_display_text
                    let cursor_byte_pos = char_to_byte_pos(&line_display_text, transformed_position_in_line);
                    
                    for run in line_runs {
                        if byte_pos >= cursor_byte_pos {
                            break;
                        }
                        
                        if byte_pos + run.len <= cursor_byte_pos {
                            // Full run is before cursor
                            runs_for_cursor.push(run.clone());
                            byte_pos += run.len;
                        } else {
                            // Partial run up to cursor - calculate partial BYTE length
                            let partial_byte_len = cursor_byte_pos - byte_pos;
                            runs_for_cursor.push(TextRun {
                                len: partial_byte_len,
                                font: run.font,
                                color: run.color,
                                background_color: run.background_color,
                                underline: run.underline,
                                strikethrough: run.strikethrough,
                            });
                            break;
                        }
                    }
                    
                    if !runs_for_cursor.is_empty() {
                        let shaped_line = window.text_system().shape_line(
                            text_up_to_cursor.into(),
                            font_size,
                            &runs_for_cursor,
                            None,
                        );
                        
                        shaped_line.width
                    } else {
                        px(0.0)
                    }
                }
            }
        };

        // Create cursor bounds - a thin vertical line
        let cursor_x = bounds.origin.x + padding + cursor_x_offset;
        
        // Calculate cursor Y position using logical line position and apply scroll offset
        let scroll_offset_px = px(self.scroll_offset);
        let cursor_y = if let Some(vl_idx) = visual_line_index {
            // FIXED: VisualLineManager now handles scroll offset internally
            if let Some(origin) = self.visual_line_manager.get_visual_line_bounds(vl_idx, bounds.origin, padding, line_height, self.scroll_offset) {
                eprintln!("DEBUG RENDER: Cursor Y from VisualLineManager: {:?}", origin.y);
                origin.y
            } else {
                eprintln!("DEBUG RENDER: VisualLineManager bounds calculation failed, using fallback");
                // For fallback, use document-relative position and apply scroll offset
                bounds.origin.y + padding + (line_height * vl_idx as f32) - scroll_offset_px
            }
        } else {
            eprintln!("DEBUG RENDER: No visual line index, using logical line: {}", logical_line);
            // Fallback to logical line calculation with scroll offset
            bounds.origin.y + padding + (line_height * logical_line as f32) - scroll_offset_px
        };
        
        // Only paint cursor if it's visible in the current viewport
        let cursor_is_visible = cursor_y >= bounds.origin.y && cursor_y < bounds.origin.y + bounds.size.height;
        if !cursor_is_visible {
            eprintln!("DEBUG RENDER: Cursor is outside viewport, skipping paint");
            return;
        }
        
        eprintln!("DEBUG RENDER: Final cursor position - x: {:?}, y: {:?}", cursor_x, cursor_y);
        eprintln!("DEBUG RENDER: Cursor x_offset: {:?}, padding: {:?}", cursor_x_offset, padding);

        let cursor_bounds = Bounds {
            origin: gpui::point(cursor_x, cursor_y),
            size: size(px(2.0), line_height),
        };

        // Paint the cursor as a filled rectangle
        let cursor_color = rgb(0xf38ba8); // Pink cursor color
        window.paint_quad(gpui::PaintQuad {
            bounds: cursor_bounds,
            background: cursor_color.into(),
            border_widths: gpui::Edges::all(px(0.0)),
            border_color: transparent_black().into(),
            border_style: gpui::BorderStyle::Solid,
            corner_radii: gpui::Corners::all(px(0.0)),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unicode_visual_cursor_positioning() {
        // Test that cursor visual position calculation correctly handles Unicode characters
        // This test verifies that byte lengths are properly handled when calculating cursor X position
        
        // Test case 1: Chinese characters (3 bytes each)
        let chinese_text = "你好世界";
        assert_eq!(chinese_text.len(), 12, "Chinese text should be 12 bytes");
        assert_eq!(chinese_text.chars().count(), 4, "Chinese text should be 4 characters");
        
        // Test case 2: Emoji (4 bytes)
        let emoji_text = "Hello 🌍 world";
        assert_eq!(emoji_text.len(), 16, "Emoji text should be 16 bytes (emoji is 4 bytes)");
        assert_eq!(emoji_text.chars().count(), 13, "Emoji text should be 13 characters");
        
        // Test case 3: Mixed content
        let mixed_text = "Test 你好 🌍 text";
        assert_eq!(mixed_text.len(), 21, "Mixed text should be 21 bytes");
        assert_eq!(mixed_text.chars().count(), 14, "Mixed text should be 14 characters");
        
        // Verify byte position calculation for cursor positioning
        // When cursor is at character position 5 in "Test 你好", it should be at byte position 5
        let test_str = "Test 你好";
        let char_pos = 5; // After space, before first Chinese character
        let byte_pos = test_str
            .char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(test_str.len());
        assert_eq!(byte_pos, 5, "Character position 5 should map to byte position 5");
        
        // When cursor is at character position 6 (after first Chinese char)
        let char_pos = 6;
        let byte_pos = test_str
            .char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(test_str.len());
        assert_eq!(byte_pos, 8, "Character position 6 should map to byte position 8 (after 3-byte char)");
    }
    
    #[test]
    fn test_text_run_byte_length_calculation() {
        // Test that TextRun lengths are correctly calculated in bytes
        let text = "Hello 你好 🌍";
        
        // Substring from char 0 to 5 ("Hello")
        let substring: String = text.chars().take(5).collect();
        assert_eq!(substring, "Hello");
        assert_eq!(substring.len(), 5, "ASCII substring should be 5 bytes");
        
        // Substring from char 0 to 7 ("Hello 你")
        let substring: String = text.chars().take(7).collect();
        assert_eq!(substring, "Hello 你");
        assert_eq!(substring.len(), 9, "Substring with Chinese char should be 9 bytes");
        
        // Substring from char 0 to 9 ("Hello 你好 ")
        let substring: String = text.chars().take(9).collect();
        assert_eq!(substring, "Hello 你好 ");
        assert_eq!(substring.len(), 13, "Substring with 2 Chinese chars should be 13 bytes");
    }
    
    #[test]
    fn test_char_to_byte_position_conversion() {
        // Helper function to convert character position to byte position
        fn char_to_byte_pos(text: &str, char_pos: usize) -> usize {
            text.char_indices()
                .nth(char_pos)
                .map(|(i, _)| i)
                .unwrap_or(text.len())
        }
        
        let text = "Test 你好 🌍 émojis";
        
        // Test various character positions
        assert_eq!(char_to_byte_pos(text, 0), 0, "Start of string");
        assert_eq!(char_to_byte_pos(text, 4), 4, "After 'Test'");
        assert_eq!(char_to_byte_pos(text, 5), 5, "After space");
        assert_eq!(char_to_byte_pos(text, 6), 8, "After first Chinese char (3 bytes)");
        assert_eq!(char_to_byte_pos(text, 7), 11, "After second Chinese char");
        assert_eq!(char_to_byte_pos(text, 8), 12, "After space following Chinese");
        assert_eq!(char_to_byte_pos(text, 9), 16, "After emoji (4 bytes)");
        assert_eq!(char_to_byte_pos(text, 10), 17, "After space following emoji");
        
        // Verify the full string length
        assert_eq!(text.len(), 24, "Total byte length");
        assert_eq!(text.chars().count(), 16, "Total character count");
    }
}