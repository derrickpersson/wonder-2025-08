mod helpers;

// Re-export test helpers for convenience
pub(super) use helpers::{create_test_editor_minimal, TestableEditor};

// Test modules organized by functionality  
mod basic_editor;
mod selection;
mod mouse;
mod keyboard;