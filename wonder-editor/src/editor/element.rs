use gpui::{
    px, rgb, size, App, Bounds, Element, ElementInputHandler, Entity,
    FocusHandle, LayoutId, Pixels, ShapedLine, TextRun, Window,
};
use crate::hybrid_renderer::HybridTextRenderer;
use crate::rendering::VisualLineManager;

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
    // Manage all visual lines for the document
    pub(super) visual_line_manager: VisualLineManager,
    // Scroll offset for viewport
    pub(super) scroll_offset: f32,
    // ENG-189: Store actual GPUI element bounds for viewport calculations
    pub(super) actual_bounds: Option<Bounds<Pixels>>,
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

        // Only clear visual lines if content has actually changed
        // TODO: Add proper version tracking in Phase 2

        // ENG-189: Get actual viewport information for visible line culling
        let line_height_f32 = 24.0; // px(24.0) as f32
        let scroll_offset = self.scroll_offset;
        let viewport_height = if let Some(bounds) = self.actual_bounds {
            bounds.size.height.0 // Use actual bounds from previous paint cycle
        } else {
            600.0 // Reasonable fallback for first layout before bounds are available
        };
        
        // Calculate which lines are potentially visible
        let first_visible_line = if scroll_offset > 0.0 {
            (scroll_offset / line_height_f32).floor() as usize
        } else {
            0
        };
        let lines_in_viewport = if viewport_height > 0.0 {
            (viewport_height / line_height_f32).ceil() as usize + 2 // +2 for buffer
        } else {
            lines.len() // Show all if viewport not set
        };
        let last_visible_line = (first_visible_line + lines_in_viewport).min(lines.len());

        for (logical_line_index, line) in lines.iter().enumerate() {
            // Skip lines outside viewport for performance
            if logical_line_index < first_visible_line || logical_line_index >= last_visible_line {
                // Still need to update offset for cursor calculations
                current_offset += line.len() + 1;
                continue;
            }

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

            // Use the hybrid renderer's line wrapping system for proper styling and measurement
            let visual_lines = self.hybrid_renderer.wrap_line(
                logical_line_index,
                *line,
                line_cursor_position,
                line_selection,
                0, // TODO: Pass actual document version
                window,
            );
            
            // Add visual lines to the manager
            self.visual_line_manager.add_visual_lines_for_logical(logical_line_index, visual_lines.clone());

            // Convert each visual line to a shaped line for GPUI
            for visual_line in visual_lines {
                let shaped_line = if !visual_line.segments.is_empty() {
                    // Use the visual line's styled segments
                    let combined_text: String = visual_line.segments.iter().map(|s| s.text.as_str()).collect();
                    let combined_runs: Vec<_> = visual_line.segments.iter().map(|s| s.text_run.clone()).collect();
                    
                    // Use the first segment's font size as primary (H1 will dominate if present)
                    let primary_font_size = visual_line.segments.first().map(|s| px(s.font_size)).unwrap_or(font_size);
                    
                    window.text_system().shape_line(
                        combined_text.into(),
                        primary_font_size,
                        &combined_runs,
                        None
                    )
                } else {
                    // Fallback for empty visual lines
                    let text_run = TextRun {
                        len: 1,
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
                    window.text_system().shape_line(" ".into(), font_size, &[text_run], None)
                };

                max_width = max_width.max(shaped_line.width);
                shaped_lines.push(shaped_line);
            }

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
        
        // ENG-185: We need ALL visual lines to know real height, but we only render visible ones
        // This is a fundamental limitation - we can't know the real height without rendering everything
        // For now, use logical line count, but the actual height will be updated from paint phase
        let total_document_lines = lines.len().max(1);

        let total_width = max_width + padding * 2.0;
        // Use total document lines for height - this ensures scrollable area exists
        // The actual height will be updated from paint phase with real Y positions
        let total_height = (line_height * total_document_lines as f32) + padding * 2.0;

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
        // ENG-189: Store actual GPUI bounds for future layout calculations
        self.actual_bounds = Some(bounds);
        
        // Update the editor with the actual element bounds and viewport
        self.editor.update(cx, |editor, _cx| {
            editor.update_element_bounds(bounds);
            editor.update_viewport_from_bounds(bounds);
            
            // Debug: Print the actual bounds we receive
            // eprintln!("🎯 ACTUAL ELEMENT BOUNDS RECEIVED:");
            // eprintln!("  Origin: ({:.1}, {:.1})px", bounds.origin.x.0, bounds.origin.y.0);
            // eprintln!("  Size: {:.1} x {:.1}px", bounds.size.width.0, bounds.size.height.0);
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
        // Apply scroll offset to the starting Y position
        let scroll_offset_px = px(self.scroll_offset);
        let text_origin = bounds.origin + gpui::point(padding, padding - scroll_offset_px);
        let mut actual_line_positions = Vec::new();

        // Paint selection first (behind text)
        if let Some(ref selection_range) = self.selection {
            self.paint_selection(bounds, shaped_lines, selection_range.clone(), window);
        }

        let line_height = px(24.0);
        for (shaped_line_index, shaped_line) in shaped_lines.iter_mut().enumerate() {
            // Calculate Y position for this line - paint it at the correct position
            // accounting for scroll offset
            let line_y = text_origin.y + px(shaped_line_index as f32 * line_height.0);
            
            // CRITICAL FIX: Capture Y position relative to document origin (scroll-agnostic)
            // This position should be independent of scroll offset for consistent coordinate mapping
            let document_relative_y = shaped_line_index as f32 * line_height.0;
            actual_line_positions.push(document_relative_y);
            
            // Paint the line at the calculated position
            shaped_line
                .paint(gpui::point(text_origin.x, line_y), line_height, window, cx)
                .unwrap_or_else(|_err| {
                    // eprintln!("Failed to paint text line: {:?}", err);
                });
        }

        // Update the visual line manager with Y positions
        self.visual_line_manager.update_y_positions(actual_line_positions.clone());
        
        // Update the editor with both line positions and visual line manager
        self.editor.update(cx, |editor, _cx| {
            editor.update_line_positions(actual_line_positions);
            // Pass the visual line manager to the editor for mouse coordinate conversion
            editor.update_visual_line_manager(self.visual_line_manager.clone());
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