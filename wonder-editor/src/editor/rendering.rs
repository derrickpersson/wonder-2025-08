use gpui::{
    px, rgb, size, transparent_black, App, Bounds, Pixels, ShapedLine, TextRun, Window,
};
use crate::hybrid_renderer::HybridTextRenderer;

use super::element::EditorElement;

impl EditorElement {
    pub(super) fn paint_selection(
        &self,
        bounds: Bounds<Pixels>,
        shaped_lines: &[ShapedLine],
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
        let mut y_offset = padding;
        let font_size = px(16.0);

        // Process each line with its corresponding shaped line
        let lines: Vec<&str> = content.lines().collect();
        for (line_index, line_text) in lines.iter().enumerate() {
            let line_start = char_offset;
            let line_end = char_offset + line_text.len();
            
            // Check if this line intersects with the selection
            if selection_range.end > line_start && selection_range.start <= line_end {
                // Calculate the selection bounds within this line (original coordinates)
                let sel_start_in_line = selection_range.start.saturating_sub(line_start);
                let sel_end_in_line = (selection_range.end.min(line_end) - line_start).min(line_text.len());
                
                if sel_start_in_line < sel_end_in_line && line_index < shaped_lines.len() {
                    // Calculate cursor position for this line
                    let line_cursor_pos = if self.cursor_position >= line_start && self.cursor_position <= line_end {
                        self.cursor_position - line_start
                    } else {
                        usize::MAX
                    };

                    // Calculate line selection
                    let line_selection = Some(sel_start_in_line..sel_end_in_line);

                    // Get the transformed content and text runs for this line
                    let line_display_text = self.hybrid_renderer.get_display_content(*line_text, line_cursor_pos, line_selection.clone());
                    let line_runs = self.hybrid_renderer.generate_mixed_text_runs(*line_text, line_cursor_pos, line_selection.clone());

                    // Map selection positions from original to transformed coordinates
                    let transformed_start = self.hybrid_renderer.map_cursor_position(*line_text, sel_start_in_line, line_selection.clone());
                    let transformed_end = self.hybrid_renderer.map_cursor_position(*line_text, sel_end_in_line, line_selection.clone());

                    // Calculate x positions using transformed content
                    let x_start = if transformed_start == 0 {
                        padding
                    } else {
                        let text_to_sel_start = line_display_text.chars().take(transformed_start).collect::<String>();
                        if !text_to_sel_start.is_empty() {
                            if line_runs.is_empty() {
                                // Fallback to simple measurement
                                let text_run = TextRun {
                                    len: text_to_sel_start.len(),
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
                                    text_to_sel_start.into(),
                                    font_size,
                                    &[text_run],
                                    None,
                                );
                                padding + shaped.width
                            } else {
                                // Use the proper text runs for accurate measurement
                                let mut runs_for_start = Vec::new();
                                let mut run_pos = 0;
                                
                                for run in &line_runs {
                                    if run_pos >= transformed_start {
                                        break;
                                    }
                                    
                                    if run_pos + run.len <= transformed_start {
                                        runs_for_start.push(run.clone());
                                        run_pos += run.len;
                                    } else {
                                        let partial_len = transformed_start - run_pos;
                                        runs_for_start.push(TextRun {
                                            len: partial_len,
                                            font: run.font.clone(),
                                            color: run.color,
                                            background_color: run.background_color,
                                            underline: run.underline.clone(),
                                            strikethrough: run.strikethrough.clone(),
                                        });
                                        break;
                                    }
                                }
                                
                                if !runs_for_start.is_empty() {
                                    let shaped = window.text_system().shape_line(
                                        text_to_sel_start.into(),
                                        font_size,
                                        &runs_for_start,
                                        None,
                                    );
                                    padding + shaped.width
                                } else {
                                    padding
                                }
                            }
                        } else {
                            padding
                        }
                    };
                    
                    // Calculate x_end using transformed content
                    let x_end = {
                        let text_to_sel_end = line_display_text.chars().take(transformed_end).collect::<String>();
                        if !text_to_sel_end.is_empty() {
                            if line_runs.is_empty() {
                                // Fallback to simple measurement
                                let text_run = TextRun {
                                    len: text_to_sel_end.len(),
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
                                    text_to_sel_end.into(),
                                    font_size,
                                    &[text_run],
                                    None,
                                );
                                padding + shaped.width
                            } else {
                                // Use the proper text runs for accurate measurement
                                let mut runs_for_end = Vec::new();
                                let mut run_pos = 0;
                                
                                for run in &line_runs {
                                    if run_pos >= transformed_end {
                                        break;
                                    }
                                    
                                    if run_pos + run.len <= transformed_end {
                                        runs_for_end.push(run.clone());
                                        run_pos += run.len;
                                    } else {
                                        let partial_len = transformed_end - run_pos;
                                        runs_for_end.push(TextRun {
                                            len: partial_len,
                                            font: run.font.clone(),
                                            color: run.color,
                                            background_color: run.background_color,
                                            underline: run.underline.clone(),
                                            strikethrough: run.strikethrough.clone(),
                                        });
                                        break;
                                    }
                                }
                                
                                if !runs_for_end.is_empty() {
                                    let shaped = window.text_system().shape_line(
                                        text_to_sel_end.into(),
                                        font_size,
                                        &runs_for_end,
                                        None,
                                    );
                                    padding + shaped.width
                                } else {
                                    padding
                                }
                            }
                        } else {
                            padding
                        }
                    };
                    
                    // Paint selection rectangle for this line
                    // For empty lines, show a minimum width selection to indicate the line is selected
                    let selection_width = if x_end > x_start {
                        x_end - x_start
                    } else {
                        // Empty line: show a small visual indicator (about 4 pixels wide)
                        px(4.0)
                    };
                    
                    if selection_width > px(0.0) {
                        window.paint_quad(gpui::PaintQuad {
                            bounds: Bounds {
                                origin: bounds.origin + gpui::point(x_start, y_offset),
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
            }
            
            // Move to next line
            char_offset = line_end + 1; // +1 for newline character
            y_offset += line_height;
        }
    }

    pub(super) fn paint_cursor(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        // Get the cursor position from the editor (original content position)
        let original_cursor_position = self.editor.read(cx).cursor_position();
        let content = &self.content;
        
        eprintln!("DEBUG RENDER: Painting cursor at original position: {}", original_cursor_position);
        eprintln!("DEBUG RENDER: Content length: {}", content.len());
        
        // Map cursor position to transformed content coordinates
        let transformed_cursor_position = self.hybrid_renderer.map_cursor_position(
            content.as_str(), 
            original_cursor_position, 
            self.selection.clone()
        );
        
        eprintln!("DEBUG RENDER: Transformed cursor position: {}", transformed_cursor_position);

        // Calculate which line the cursor is on based on ORIGINAL content (for line counting)
        let chars_before_cursor: String = content.chars().take(original_cursor_position).collect();
        let line_number = chars_before_cursor.matches('\n').count();
        
        eprintln!("DEBUG RENDER: Cursor is on line {} (0-based)", line_number);

        // Get the actual line content from original text
        let current_line_original = content.lines().nth(line_number).unwrap_or("");
        eprintln!("DEBUG RENDER: Line content: {:?}", current_line_original);
        
        // Find cursor position within this specific line (original coordinates)
        let lines_before: Vec<&str> = chars_before_cursor.lines().collect();
        let original_position_in_line = if chars_before_cursor.ends_with('\n') {
            0
        } else {
            lines_before
                .last()
                .map(|line| line.chars().count())
                .unwrap_or(0)
        };

        // Calculate the selection for this line
        let line_start_offset = content.lines().take(line_number).map(|l| l.len() + 1).sum::<usize>(); // +1 for newlines
        let line_cursor_pos = if original_cursor_position >= line_start_offset && original_cursor_position <= line_start_offset + current_line_original.len() {
            original_cursor_position - line_start_offset
        } else {
            usize::MAX
        };
        
        let line_selection = self.selection.as_ref().and_then(|sel| {
            let line_end = line_start_offset + current_line_original.len();
            if sel.end > line_start_offset && sel.start <= line_end {
                let adjusted_start = sel.start.saturating_sub(line_start_offset);
                let adjusted_end = (sel.end - line_start_offset).min(current_line_original.len());
                Some(adjusted_start..adjusted_end)
            } else {
                None
            }
        });

        // Get the transformed content for this line and map cursor position within the line
        let line_display_text = self.hybrid_renderer.get_display_content(current_line_original, line_cursor_pos, line_selection.clone());
        let transformed_position_in_line = self.hybrid_renderer.map_cursor_position(current_line_original, original_position_in_line, line_selection.clone());

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
                // Generate the actual text runs for this line so we measure correctly
                let line_runs = self.hybrid_renderer.generate_mixed_text_runs(
                    current_line_original, 
                    line_cursor_pos, 
                    line_selection
                );

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
        let cursor_y = bounds.origin.y + padding + (line_height * line_number as f32);
        
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