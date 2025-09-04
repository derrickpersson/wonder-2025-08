//! Cursor positioning diagnostics module
//! 
//! This module provides comprehensive logging and debugging tools to diagnose
//! cursor positioning issues during movement and selection operations.

use crate::core::{Point as TextPoint, CoordinateConversion, RopeCoordinateMapper};
use gpui::{Pixels, Point};
use ropey::Rope;
use std::sync::atomic::{AtomicBool, Ordering};

static DIAGNOSTICS_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable or disable cursor diagnostics
pub fn set_diagnostics_enabled(enabled: bool) {
    DIAGNOSTICS_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        eprintln!("🔍 CURSOR DIAGNOSTICS ENABLED - Press Ctrl+Shift+D to toggle");
        eprintln!("═══════════════════════════════════════════════════════════");
    } else {
        eprintln!("🔕 CURSOR DIAGNOSTICS DISABLED");
    }
}

/// Check if diagnostics are enabled
pub fn diagnostics_enabled() -> bool {
    DIAGNOSTICS_ENABLED.load(Ordering::Relaxed)
}

/// Log cursor movement details
pub fn log_cursor_movement(
    operation: &str,
    old_position: usize,
    new_position: usize,
    old_point: TextPoint,
    new_point: TextPoint,
    content_sample: &str,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n📍 CURSOR MOVEMENT: {}", operation);
    eprintln!("  Old: offset={}, point=({}, {})", old_position, old_point.row, old_point.column);
    eprintln!("  New: offset={}, point=({}, {})", new_position, new_point.row, new_point.column);
    eprintln!("  Delta: offset={:+}, row={:+}, col={:+}", 
        new_position as i32 - old_position as i32,
        new_point.row as i32 - old_point.row as i32,
        new_point.column as i32 - old_point.column as i32
    );
    
    // Show context around cursor
    let start = old_position.saturating_sub(20);
    let end = (new_position + 20).min(content_sample.len());
    if let Some(context) = content_sample.get(start..end) {
        let cursor_offset = old_position - start;
        let new_cursor_offset = new_position - start;
        
        eprintln!("  Context: \"{}\"", context.replace('\n', "⏎"));
        eprintln!("           {}^{}", 
            " ".repeat(cursor_offset + 1),
            if cursor_offset != new_cursor_offset {
                format!("→{}", " ".repeat((new_cursor_offset as i32 - cursor_offset as i32 - 1).abs() as usize))
            } else {
                String::new()
            }
        );
    }
}

/// Log selection state changes
pub fn log_selection_change(
    operation: &str,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    cursor_position: usize,
    content: &str,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n🔲 SELECTION CHANGE: {}", operation);
    
    if let (Some(start), Some(end)) = (selection_start, selection_end) {
        eprintln!("  Range: {} to {} ({} chars)", start, end, end - start);
        eprintln!("  Cursor at: {}", cursor_position);
        
        // Show selected text (truncated if too long)
        if let Some(selected) = content.get(start..end) {
            let display = if selected.len() > 50 {
                format!("{}...", &selected[..50])
            } else {
                selected.to_string()
            };
            eprintln!("  Selected: \"{}\"", display.replace('\n', "⏎"));
        }
    } else {
        eprintln!("  No selection (cursor at {})", cursor_position);
    }
}

/// Log mouse click positioning details
pub fn log_mouse_click(
    screen_point: Point<Pixels>,
    calculated_text_point: TextPoint,
    calculated_offset: usize,
    actual_cursor_position: usize,
    line_content: &str,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n🖱️ MOUSE CLICK POSITIONING");
    eprintln!("  Screen: ({:.1}, {:.1}) pixels", screen_point.x.0, screen_point.y.0);
    eprintln!("  Calculated Point: row={}, col={}", calculated_text_point.row, calculated_text_point.column);
    eprintln!("  Calculated Offset: {}", calculated_offset);
    eprintln!("  Actual Cursor: {}", actual_cursor_position);
    
    if calculated_offset != actual_cursor_position {
        eprintln!("  ⚠️ MISMATCH: Expected {} but got {} (diff: {:+})", 
            calculated_offset, 
            actual_cursor_position,
            actual_cursor_position as i32 - calculated_offset as i32
        );
    }
    
    eprintln!("  Line content: \"{}\"", line_content.replace('\n', "⏎"));
    if calculated_text_point.column as usize <= line_content.len() {
        eprintln!("  Click position: {}^", " ".repeat(calculated_text_point.column as usize + 16));
    }
}

/// Log coordinate conversion details
pub fn log_coordinate_conversion(
    operation: &str,
    input_offset: Option<usize>,
    input_point: Option<TextPoint>,
    output_offset: Option<usize>,
    output_point: Option<TextPoint>,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n🔄 COORDINATE CONVERSION: {}", operation);
    
    if let Some(offset) = input_offset {
        eprintln!("  Input offset: {}", offset);
    }
    if let Some(point) = input_point {
        eprintln!("  Input point: ({}, {})", point.row, point.column);
    }
    if let Some(offset) = output_offset {
        eprintln!("  Output offset: {}", offset);
    }
    if let Some(point) = output_point {
        eprintln!("  Output point: ({}, {})", point.row, point.column);
    }
}

/// Log line height calculation details
pub fn log_line_height_calculation(
    line_index: usize,
    line_content: &str,
    calculated_height: f32,
    is_heading: bool,
    heading_level: Option<u32>,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n📏 LINE HEIGHT CALCULATION");
    eprintln!("  Line {}: \"{}\"", line_index, 
        if line_content.len() > 40 {
            format!("{}...", &line_content[..40])
        } else {
            line_content.to_string()
        }.replace('\n', "⏎")
    );
    eprintln!("  Height: {:.1}px", calculated_height);
    if is_heading {
        eprintln!("  Heading level: {}", heading_level.unwrap_or(0));
    }
}

/// Log keyboard input and its effect on cursor
pub fn log_keyboard_input(
    key: &str,
    modifiers: &str,
    old_position: usize,
    new_position: usize,
    selection_active: bool,
) {
    if !diagnostics_enabled() { return; }
    
    eprintln!("\n⌨️ KEYBOARD INPUT");
    eprintln!("  Key: {} {}", modifiers, key);
    eprintln!("  Position: {} → {} (moved {})", 
        old_position, 
        new_position,
        if new_position >= old_position {
            format!("+{}", new_position - old_position)
        } else {
            format!("-{}", old_position - new_position)
        }
    );
    eprintln!("  Selection: {}", if selection_active { "active" } else { "none" });
}

/// Comprehensive diagnostic report for current cursor state
pub fn generate_diagnostic_report(
    cursor_offset: usize,
    cursor_point: TextPoint,
    selection: Option<(usize, usize)>,
    content: &str,
    viewport_height: f32,
) -> String {
    let mut report = String::new();
    
    report.push_str("\n╔══════════════════════════════════════════════════════╗\n");
    report.push_str("║         CURSOR POSITIONING DIAGNOSTIC REPORT          ║\n");
    report.push_str("╚══════════════════════════════════════════════════════╝\n\n");
    
    // Document stats
    let rope = Rope::from_str(content);
    let total_lines = rope.len_lines();
    let total_chars = rope.len_chars();
    
    report.push_str(&format!("📄 DOCUMENT STATS\n"));
    report.push_str(&format!("  Total lines: {}\n", total_lines));
    report.push_str(&format!("  Total chars: {}\n", total_chars));
    report.push_str(&format!("  Viewport height: {:.1}px\n\n", viewport_height));
    
    // Cursor position
    report.push_str(&format!("📍 CURSOR POSITION\n"));
    report.push_str(&format!("  Offset: {} / {}\n", cursor_offset, total_chars));
    report.push_str(&format!("  Point: row={}, col={}\n", cursor_point.row, cursor_point.column));
    report.push_str(&format!("  Progress: {:.1}%\n\n", (cursor_offset as f32 / total_chars as f32) * 100.0));
    
    // Selection info
    report.push_str(&format!("🔲 SELECTION\n"));
    if let Some((start, end)) = selection {
        report.push_str(&format!("  Active: {} to {} ({} chars)\n", start, end, end - start));
        report.push_str(&format!("  Direction: {}\n", if cursor_offset == end { "forward" } else { "backward" }));
    } else {
        report.push_str("  None\n");
    }
    report.push_str("\n");
    
    // Line context
    if cursor_point.row < total_lines as u32 {
        let line_start = rope.line_to_char(cursor_point.row as usize);
        let line_end = if (cursor_point.row as usize + 1) < total_lines {
            rope.line_to_char(cursor_point.row as usize + 1)
        } else {
            total_chars
        };
        
        if let Some(line_content) = content.get(line_start..line_end) {
            report.push_str(&format!("📝 CURRENT LINE ({})\n", cursor_point.row));
            report.push_str(&format!("  Content: \"{}\"\n", line_content.replace('\n', "⏎")));
            report.push_str(&format!("  Length: {} chars\n", line_content.len()));
            report.push_str(&format!("  Cursor at column: {}\n", cursor_point.column));
            
            // Visual cursor position
            if (cursor_point.column as usize) <= line_content.len() {
                report.push_str(&format!("  Position: {}{}\n", 
                    " ".repeat(cursor_point.column as usize + 11),
                    "^"
                ));
            }
        }
    }
    
    report.push_str("\n💡 DIAGNOSTIC TIPS:\n");
    report.push_str("  • Press Ctrl+Shift+D to toggle diagnostics\n");
    report.push_str("  • Check console for detailed operation logs\n");
    report.push_str("  • Screenshot this report when issues occur\n");
    
    report
}

/// Log position flow through click handler to identify corruption point  
pub fn log_position_flow(stage: &str, position: usize, description: &str) {
    if !diagnostics_enabled() { return; }
    eprintln!("🔍 POSITION FLOW: {} => position={} ({})", stage, position, description);
}

/// Test helper: Verify cursor position consistency
pub fn verify_cursor_consistency(
    offset: usize,
    point: TextPoint,
    content: &str,
) -> Result<(), String> {
    let rope = Rope::from_str(content);
    let mapper = RopeCoordinateMapper::new(rope);
    
    // Convert offset to point and verify
    let calculated_point = mapper.offset_to_point(offset);
    if calculated_point != point {
        return Err(format!(
            "Point mismatch: expected ({}, {}), got ({}, {}) for offset {}",
            point.row, point.column,
            calculated_point.row, calculated_point.column,
            offset
        ));
    }
    
    // Convert point to offset and verify
    let calculated_offset = mapper.point_to_offset(point);
    if calculated_offset != offset {
        return Err(format!(
            "Offset mismatch: expected {}, got {} for point ({}, {})",
            offset, calculated_offset,
            point.row, point.column
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_diagnostics_toggle() {
        set_diagnostics_enabled(true);
        assert!(diagnostics_enabled());
        
        set_diagnostics_enabled(false);
        assert!(!diagnostics_enabled());
    }
    
    #[test]
    fn test_cursor_consistency_verification() {
        let content = "hello\nworld\ntest";
        
        // Test valid positions
        assert!(verify_cursor_consistency(0, TextPoint::new(0, 0), content).is_ok());
        assert!(verify_cursor_consistency(5, TextPoint::new(0, 5), content).is_ok());
        assert!(verify_cursor_consistency(6, TextPoint::new(1, 0), content).is_ok());
        assert!(verify_cursor_consistency(11, TextPoint::new(1, 5), content).is_ok());
        
        // Test invalid position
        assert!(verify_cursor_consistency(5, TextPoint::new(1, 0), content).is_err());
    }
    
    #[test]
    fn test_diagnostic_report_generation() {
        let content = "hello\nworld";
        let report = generate_diagnostic_report(
            6,
            TextPoint::new(1, 0),
            Some((0, 5)),
            content,
            500.0
        );
        
        assert!(report.contains("CURSOR POSITIONING DIAGNOSTIC REPORT"));
        assert!(report.contains("Total lines: 2"));
        assert!(report.contains("Total chars: 11"));
        assert!(report.contains("row=1, col=0"));
        assert!(report.contains("Active: 0 to 5"));
    }
}