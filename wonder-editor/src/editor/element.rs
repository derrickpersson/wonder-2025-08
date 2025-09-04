use gpui::{
    px, rgb, size, App, Bounds, Element, ElementInputHandler, Entity,
    FocusHandle, LayoutId, Pixels, ShapedLine, TextRun, Window,
};
use crate::hybrid_renderer::HybridTextRenderer;

use super::MarkdownEditor;

// Custom element that handles text layout and input registration during paint phase
pub(super) struct EditorElement {
    pub(super) editor: Entity<MarkdownEditor>,
    pub(super) content: String,
    pub(super) focused: bool,
    pub(super) focus_handle: FocusHandle,
    pub(super) cursor_position: usize,
    pub(super) selection: Option<std::ops::Range<usize>>,
    pub(super) hybrid_renderer: HybridTextRenderer,
}

impl Element for EditorElement {
    type RequestLayoutState = Vec<ShapedLine>;
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let text_to_display = if self.content.is_empty() {
            "Start typing your markdown content...".to_string()
        } else {
            self.content.clone()
        };

        // Split text by newlines and handle each line separately
        let lines: Vec<&str> = text_to_display.lines().collect();
        let font_size = px(16.0);

        let mut shaped_lines = Vec::new();
        let mut max_width = px(0.0);
        let mut current_offset = 0;

        for line in lines {
            // Prepare line content for rendering - avoid string conversion for non-empty lines

            // Calculate cursor position for this line
            let line_cursor_position = if self.cursor_position >= current_offset
                && self.cursor_position <= current_offset + line.len()
            {
                self.cursor_position - current_offset
            } else {
                usize::MAX // Cursor not in this line
            };

            // Calculate selection for this line
            let line_selection = self.selection.as_ref().and_then(|sel| {
                let line_start = current_offset;
                let line_end = current_offset + line.len();
                if sel.end > line_start && sel.start < line_end {
                    let adjusted_start = sel.start.saturating_sub(line_start);
                    let adjusted_end = (sel.end - line_start).min(line.len());
                    Some(adjusted_start..adjusted_end)
                } else {
                    None
                }
            });

            // Get both the transformed content and the styled segments (NEW APPROACH)
            // Use RopeSlice directly for efficiency - no string conversion for non-empty lines
            let (display_text, styled_segments, line_runs) = if line.is_empty() {
                // For empty lines, use a space string
                let empty_line = " ";
                (
                    self.hybrid_renderer.get_display_content(empty_line, line_cursor_position, line_selection.clone()),
                    self.hybrid_renderer.generate_styled_text_segments(empty_line, line_cursor_position, line_selection.clone()),
                    self.hybrid_renderer.generate_mixed_text_runs(empty_line, line_cursor_position, line_selection.clone())
                )
            } else {
                // For non-empty lines, use RopeSlice directly (no string conversion!)
                (
                    self.hybrid_renderer.get_display_content(line, line_cursor_position, line_selection.clone()),
                    self.hybrid_renderer.generate_styled_text_segments(line, line_cursor_position, line_selection.clone()),
                    self.hybrid_renderer.generate_mixed_text_runs(line, line_cursor_position, line_selection)
                )
            };

            // Use the display text (transformed) for shaping, not the original line text
            let text_to_shape = if display_text.is_empty() { " ".to_string() } else { display_text };
            
            // If no hybrid runs, use fallback styling based on display text length
            let text_runs = if line_runs.is_empty() {
                vec![TextRun {
                    len: text_to_shape.len(),
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
                }]
            } else {
                line_runs
            };

            // COMPLETE FONT SIZE INTEGRATION - Multi-segment text shaping
            let shaped_line = if !styled_segments.is_empty() {
                // Combine all segment text and text runs, but KEEP different font sizes
                // TODO: GPUI limitation - shape_line only accepts one font_size parameter
                // This is the core challenge: GPUI doesn't support mixed font sizes in a single call
                
                // For now, we can choose the approach:
                // Option 1: Use the font size of the first segment
                // Option 2: Use a weighted average font size  
                // Option 3: Shape each segment separately (complex layout integration needed)
                
                let combined_text: String = styled_segments.iter().map(|s| s.text.as_str()).collect();
                let combined_runs: Vec<_> = styled_segments.iter().map(|s| s.text_run.clone()).collect();
                
                // Use the first segment's font size as primary (H1 will dominate if present)
                let primary_font_size = styled_segments.first().map(|s| px(s.font_size)).unwrap_or(font_size);
                
                window.text_system().shape_line(
                    combined_text.into(),
                    primary_font_size,
                    &combined_runs,
                    None
                )
            } else {
                // Fallback to current single-font-size approach
                window.text_system().shape_line(text_to_shape.into(), font_size, &text_runs, None)
            };

            max_width = max_width.max(shaped_line.width);
            shaped_lines.push(shaped_line);

            // Update offset for next line (include newline character)
            current_offset += line.len() + 1;
        }

        // Handle case where text ends with newline (add empty line)
        if text_to_display.ends_with('\n') {
            let text_run = TextRun {
                len: 1,
                font: gpui::Font {
                    family: "system-ui".into(),
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

            let shaped_line =
                window
                    .text_system()
                    .shape_line(" ".into(), font_size, &[text_run], None);
            shaped_lines.push(shaped_line);
        }

        // Calculate the size we need including padding
        let line_height = px(24.0);
        let padding = px(16.0);
        let num_lines = shaped_lines.len().max(1);

        let total_width = max_width + padding * 2.0;
        let total_height = (line_height * num_lines as f32) + padding * 2.0;

        // Create layout with our calculated size
        let layout_id = window.request_layout(
            gpui::Style {
                size: size(total_width.into(), total_height.into()).into(),
                ..Default::default()
            },
            [],
            _cx,
        );

        (layout_id, shaped_lines)
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        shaped_lines: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Update the editor with the actual element bounds
        self.editor.update(cx, |editor, _cx| {
            editor.update_element_bounds(bounds);
            
            // Debug: Print the actual bounds we receive
            eprintln!("🎯 ACTUAL ELEMENT BOUNDS RECEIVED:");
            eprintln!("  Origin: ({:.1}, {:.1})px", bounds.origin.x.0, bounds.origin.y.0);
            eprintln!("  Size: {:.1} x {:.1}px", bounds.size.width.0, bounds.size.height.0);
        });

        // ALWAYS register input handler so we can receive text input
        window.handle_input(
            &self.focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        // Paint background
        let background_color = rgb(0x11111b);
        let border_color = if self.focused {
            rgb(0x89b4fa)
        } else {
            rgb(0x313244)
        };

        window.paint_quad(gpui::PaintQuad {
            bounds,
            background: background_color.into(),
            border_widths: gpui::Edges::all(px(1.0)),
            border_color: border_color.into(),
            border_style: gpui::BorderStyle::Solid,
            corner_radii: gpui::Corners::all(px(4.0)),
        });

        // Paint all text lines and capture actual line positions
        let padding = px(16.0);
        let mut text_origin = bounds.origin + gpui::point(padding, padding);
        let mut actual_line_positions = Vec::new();

        // Paint selection first (behind text)
        if let Some(ref selection_range) = self.selection {
            self.paint_selection(bounds, shaped_lines, selection_range.clone(), window);
        }

        let line_height = px(24.0);
        for shaped_line in shaped_lines.iter_mut() {
            // Capture the actual Y position for this line (relative to content area)
            let relative_y = text_origin.y - bounds.origin.y - padding;
            actual_line_positions.push(relative_y.0);
            
            shaped_line
                .paint(text_origin, line_height, window, cx)
                .unwrap_or_else(|err| {
                    eprintln!("Failed to paint text line: {:?}", err);
                });

            // Move to next line
            text_origin.y += line_height;
        }

        // Update the editor with actual line positions
        self.editor.update(cx, |editor, _cx| {
            editor.update_line_positions(actual_line_positions);
        });

        // Paint cursor if focused
        if self.focused {
            self.paint_cursor(bounds, window, cx);
        }
    }
}


impl gpui::IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}